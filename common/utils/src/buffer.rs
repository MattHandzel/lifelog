use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

#[derive(Debug, thiserror::Error)]
pub enum BufferError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Corrupt Data: {0}")]
    CorruptData(String),
}

/// A disk-backed byte-oriented buffer (Write-Ahead Log) that supports appending raw bytes,
/// peeking at chunks of items, and committing the read offset.
///
/// Layout:
/// - `wal.log`: Sequence of [len: u32][data: bytes]
/// - `wal.cursor`: [offset: u64] (The byte offset in wal.log where the next read should start)
#[derive(Debug)]
pub struct DiskBuffer {
    directory: PathBuf,
}

impl DiskBuffer {
    pub fn new(directory: impl AsRef<Path>) -> Result<Self, BufferError> {
        let dir = directory.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;
        Ok(Self { directory: dir })
    }

    fn log_path(&self) -> PathBuf {
        self.directory.join("wal.log")
    }

    fn cursor_path(&self) -> PathBuf {
        self.directory.join("wal.cursor")
    }

    /// Append raw bytes to the log.
    pub async fn append(&self, data: &[u8]) -> Result<(), BufferError> {
        let len = data.len() as u32;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_path())
            .await?;

        file.write_u32_le(len).await?;
        file.write_all(data).await?;
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

    /// Peek at the next chunk of raw items starting from the committed offset.
    /// Returns `(next_offset, items)`.
    pub async fn peek_chunk(&self, max_items: usize) -> Result<(u64, Vec<Vec<u8>>), BufferError> {
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
                    break;
                }
                Err(e) => return Err(e.into()),
            }

            items.push(data_buf);

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
            .truncate(true)
            .open(path)
            .await?;

        file.write_all(&offset.to_le_bytes()).await?;
        file.flush().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_disk_buffer_basic() {
        let tmp = tempdir().unwrap();
        let buffer = DiskBuffer::new(tmp.path()).unwrap();

        // Initially empty
        assert_eq!(buffer.get_committed_offset().await.unwrap(), 0);
        let (next, items) = buffer.peek_chunk(10).await.unwrap();
        assert_eq!(next, 0);
        assert!(items.is_empty());

        // Append items
        buffer.append(b"hello").await.unwrap();
        buffer.append(b"world").await.unwrap();

        assert_eq!(buffer.get_uncommitted_size().await.unwrap(), 18); // (4+5) + (4+5)

        // Peek
        let (next, items) = buffer.peek_chunk(1).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], b"hello");
        assert_eq!(next, 9);

        let (next, items) = buffer.peek_chunk(10).await.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], b"hello");
        assert_eq!(items[1], b"world");
        assert_eq!(next, 18);

        // Commit first item
        buffer.commit_offset(9).await.unwrap();
        assert_eq!(buffer.get_committed_offset().await.unwrap(), 9);
        assert_eq!(buffer.get_uncommitted_size().await.unwrap(), 9);

        // Peek again
        let (next, items) = buffer.peek_chunk(10).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], b"world");
        assert_eq!(next, 18);
    }
}
