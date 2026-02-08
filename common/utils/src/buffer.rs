use sha2::{Digest, Sha256};
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

const MAGIC: u32 = 0x4C4C4F47; // "LLOG" in ASCII
const HEADER_SIZE: u64 = 8; // MAGIC (4) + LEN (4)
const CHECKSUM_SIZE: u64 = 32; // SHA256

/// A disk-backed byte-oriented buffer (Write-Ahead Log) that supports appending raw bytes,
/// peeking at chunks of items, and committing the read offset.
///
/// Layout:
/// - `wal.log`: Sequence of [magic: u32][len: u32][data: bytes][checksum: 32 bytes]
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

        let mut hasher = Sha256::new();
        hasher.update(MAGIC.to_le_bytes());
        hasher.update(len.to_le_bytes());
        hasher.update(data);
        let checksum = hasher.finalize();

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_path())
            .await?;

        file.write_u32_le(MAGIC).await?;
        file.write_u32_le(len).await?;
        file.write_all(data).await?;
        file.write_all(&checksum).await?;
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
            if current_offset + HEADER_SIZE > file_len {
                break;
            }

            // Read MAGIC
            let magic = match file.read_u32_le().await {
                Ok(m) => m,
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            };

            if magic != MAGIC {
                return Err(BufferError::CorruptData(format!(
                    "Invalid magic at offset {}: expected {:08X}, got {:08X}",
                    current_offset, MAGIC, magic
                )));
            }

            // Read length (u32)
            let len = match file.read_u32_le().await {
                Ok(l) => l as u64,
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return Err(BufferError::CorruptData(format!(
                        "Truncated header at offset {}",
                        current_offset
                    )));
                }
                Err(e) => return Err(e.into()),
            };

            if current_offset + HEADER_SIZE + len + CHECKSUM_SIZE > file_len {
                // Potential torn write at the end of file
                break;
            }

            // Read data
            let mut data_buf = vec![0u8; len as usize];
            match file.read_exact(&mut data_buf).await {
                Ok(_) => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(e) => return Err(e.into()),
            }

            // Read checksum
            let mut stored_checksum = [0u8; 32];
            match file.read_exact(&mut stored_checksum).await {
                Ok(_) => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(e) => return Err(e.into()),
            }

            // Verify checksum
            let mut hasher = Sha256::new();
            hasher.update(MAGIC.to_le_bytes());
            hasher.update((len as u32).to_le_bytes());
            hasher.update(&data_buf);
            let calculated_checksum = hasher.finalize();

            if calculated_checksum.as_slice() != stored_checksum {
                return Err(BufferError::CorruptData(format!(
                    "Checksum mismatch at offset {}",
                    current_offset
                )));
            }

            items.push(data_buf);

            // MAGIC (4) + len (4) + data len + checksum (32)
            current_offset += HEADER_SIZE + len + CHECKSUM_SIZE;
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
#[allow(clippy::unwrap_used)]
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

        assert_eq!(buffer.get_uncommitted_size().await.unwrap(), 90); // (4+4+5+32) + (4+4+5+32)

        // Peek
        let (next, items) = buffer.peek_chunk(1).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], b"hello");
        assert_eq!(next, 45);

        let (next, items) = buffer.peek_chunk(10).await.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], b"hello");
        assert_eq!(items[1], b"world");
        assert_eq!(next, 90);

        // Commit first item
        buffer.commit_offset(45).await.unwrap();
        assert_eq!(buffer.get_committed_offset().await.unwrap(), 45);
        assert_eq!(buffer.get_uncommitted_size().await.unwrap(), 45);

        // Peek again
        let (next, items) = buffer.peek_chunk(10).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], b"world");
        assert_eq!(next, 90);
    }

    #[tokio::test]
    async fn test_disk_buffer_corruption() {
        let tmp = tempdir().unwrap();
        let buffer = DiskBuffer::new(tmp.path()).unwrap();

        buffer.append(b"valid_data").await.unwrap();

        // Corrupt the file
        let path = buffer.log_path();
        let mut file = OpenOptions::new().write(true).open(&path).await.unwrap();

        // Header size (8) + data len (10) + checksum (32) = 50 bytes total
        // Offset 10 is inside the data. Let's flip a bit in the data.
        file.seek(SeekFrom::Start(10)).await.unwrap();
        file.write_all(b"X").await.unwrap();
        file.flush().await.unwrap();

        // Attempt to peek
        let result = buffer.peek_chunk(1).await;
        assert!(
            matches!(result, Err(BufferError::CorruptData(ref msg)) if msg.contains("Checksum mismatch")),
            "Expected CorruptData error with Checksum mismatch, got {:?}",
            result
        );
    }
}
