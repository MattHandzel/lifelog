use serde::{de::DeserializeOwned, Serialize};
use std::io::SeekFrom;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

#[derive(Debug, thiserror::Error)]
pub enum BufferError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization Error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("Corrupt Data: {0}")]
    CorruptData(String),
}

/// A disk-backed buffer (Write-Ahead Log) that supports appending items,
/// peeking at chunks of items, and committing the read offset.
///
/// Layout:
/// - `wal.log`: Sequence of [len: u32][data: bytes]
/// - `wal.cursor`: [offset: u64] (The byte offset in wal.log where the next read should start)
#[derive(Debug)]
pub struct DiskBuffer<T> {
    directory: PathBuf,
    _marker: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned + Send + Sync> DiskBuffer<T> {
    pub fn new(directory: impl AsRef<Path>) -> Result<Self, BufferError> {
        let dir = directory.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            directory: dir,
            _marker: PhantomData,
        })
    }

    fn log_path(&self) -> PathBuf {
        self.directory.join("wal.log")
    }

    fn cursor_path(&self) -> PathBuf {
        self.directory.join("wal.cursor")
    }

    /// Append a single item to the log.
    pub async fn append(&self, item: &T) -> Result<(), BufferError> {
        let serialized = bincode::serialize(item)?;
        let len = serialized.len() as u32;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_path())
            .await?;

        file.write_u32_le(len).await?;
        file.write_all(&serialized).await?;
        file.flush().await?;

        Ok(())
    }

    /// Read the current committed offset from the cursor file.
    /// Returns 0 if the cursor file does not exist.
    pub async fn get_committed_offset(&self) -> Result<u64, BufferError> {
        let path = self.cursor_path();
        if !path.exists() {
            return Ok(0);
        }

        let mut file = File::open(path).await?;
        let len = file.metadata().await?.len();
        if len < 8 {
            // Invalid cursor file? reset to 0
            return Ok(0);
        }

        let mut buf = [0u8; 8];
        file.read_exact(&mut buf).await?;
        Ok(u64::from_le_bytes(buf))
    }

    /// Returns the approximate size in bytes of uncommitted data (log size - cursor offset).
    pub async fn get_uncommitted_size(&self) -> Result<u64, BufferError> {
        let read_offset = self.get_committed_offset().await?;

        let path = self.log_path();
        if !path.exists() {
            return Ok(0);
        }

        match tokio::fs::metadata(path).await {
            Ok(metadata) => {
                let total_size = metadata.len();
                if total_size >= read_offset {
                    Ok(total_size - read_offset)
                } else {
                    Ok(0)
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(0),
            Err(e) => Err(e.into()),
        }
    }

    /// Peek at the next chunk of items starting from the committed offset.
    /// Returns `(next_offset, items)`.
    /// `next_offset` should be passed to `commit_offset` once the items are processed.
    pub async fn peek_chunk(&self, max_items: usize) -> Result<(u64, Vec<T>), BufferError> {
        let start_offset = self.get_committed_offset().await?;
        let mut current_offset = start_offset;
        let mut items = Vec::new();

        let path = self.log_path();
        if !path.exists() {
            return Ok((start_offset, items));
        }

        let mut file = File::open(path).await?;
        let file_len = file.metadata().await?.len();

        if start_offset >= file_len {
            return Ok((start_offset, items));
        }

        file.seek(SeekFrom::Start(start_offset)).await?;

        for _ in 0..max_items {
            if current_offset >= file_len {
                break;
            }

            // Read length (u32)
            let len = match file.read_u32_le().await {
                Ok(l) => l as u64,
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            };

            // Read data
            let mut data_buf = vec![0u8; len as usize];
            match file.read_exact(&mut data_buf).await {
                Ok(_) => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Incomplete record at end of file?
                    break;
                }
                Err(e) => return Err(e.into()),
            }

            let item: T = bincode::deserialize(&data_buf)?;
            items.push(item);

            // 4 bytes for len + data len
            current_offset += 4 + len;
        }

        Ok((current_offset, items))
    }

    /// Commit the offset, marking items before this offset as processed.
    pub async fn commit_offset(&self, offset: u64) -> Result<(), BufferError> {
        let path = self.cursor_path();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true) // Overwrite
            .open(path)
            .await?;

        file.write_all(&offset.to_le_bytes()).await?;
        file.flush().await?;
        Ok(())
    }

    // TODO: Add compaction logic (rewrite log from offset to end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestItem {
        id: u32,
        data: String,
    }

    #[tokio::test]
    async fn test_disk_buffer() {
        let dir = tempdir().unwrap();
        let buffer = DiskBuffer::<TestItem>::new(dir.path()).unwrap();

        // 1. Append items
        let item1 = TestItem {
            id: 1,
            data: "hello".to_string(),
        };
        let item2 = TestItem {
            id: 2,
            data: "world".to_string(),
        };

        buffer.append(&item1).await.unwrap();
        buffer.append(&item2).await.unwrap();

        // 2. Peek
        let (offset, items) = buffer.peek_chunk(10).await.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], item1);
        assert_eq!(items[1], item2);

        // 3. Commit
        buffer.commit_offset(offset).await.unwrap();

        // 4. Verify committed offset
        let committed = buffer.get_committed_offset().await.unwrap();
        assert_eq!(committed, offset);

        // 5. Peek again (should be empty)
        let (_, items_empty) = buffer.peek_chunk(10).await.unwrap();
        assert!(items_empty.is_empty());

        // 6. Uncommitted size
        let size = buffer.get_uncommitted_size().await.unwrap();
        assert_eq!(size, 0);

        // 7. Append more
        let item3 = TestItem {
            id: 3,
            data: "more".to_string(),
        };
        buffer.append(&item3).await.unwrap();

        let size_new = buffer.get_uncommitted_size().await.unwrap();
        assert!(size_new > 0);

        let (_, items_new) = buffer.peek_chunk(10).await.unwrap();
        assert_eq!(items_new.len(), 1);
        assert_eq!(items_new[0], item3);
    }
}
