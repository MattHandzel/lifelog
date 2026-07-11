use memchr::memmem;
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
const MAGIC_BYTES: [u8; 4] = MAGIC.to_le_bytes();
const HEADER_SIZE: u64 = 8; // MAGIC (4) + LEN (4)
const CHECKSUM_SIZE: u64 = 32; // SHA256
// Upper bound on a single WAL entry's payload size, used to reject
// implausible LEN values during corruption recovery.
const MAX_ENTRY_SIZE: u64 = 128 * 1024 * 1024;
// Read window for scanning forward looking for the next valid frame.
const SCAN_CHUNK_SIZE: usize = 4 * 1024 * 1024;

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
        file.sync_all().await?;

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
                // Frame header is corrupt. Scan forward for the next byte
                // sequence that begins a valid, checksum-verifying entry, and
                // resume from there. The data between current_offset and the
                // resync point is lost.
                match scan_for_next_valid_entry(&mut file, current_offset + 1, file_len).await? {
                    Some(resync_offset) => {
                        eprintln!(
                            "[WAL] Invalid magic at offset {}: got {:08X}; resynced to offset {} ({} bytes skipped)",
                            current_offset,
                            magic,
                            resync_offset,
                            resync_offset - current_offset
                        );
                        current_offset = resync_offset;
                        file.seek(SeekFrom::Start(current_offset)).await?;
                        continue;
                    }
                    None => {
                        return Err(BufferError::CorruptData(format!(
                            "Invalid magic at offset {}: expected {:08X}, got {:08X} (no valid entry found ahead)",
                            current_offset, MAGIC, magic
                        )));
                    }
                }
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

            // Implausibly large LEN means the frame is corrupt even though
            // MAGIC happened to match. Treat the same as bad magic: resync.
            if len > MAX_ENTRY_SIZE {
                match scan_for_next_valid_entry(&mut file, current_offset + 1, file_len).await? {
                    Some(resync_offset) => {
                        eprintln!(
                            "[WAL] Implausible LEN {} at offset {}; resynced to offset {} ({} bytes skipped)",
                            len,
                            current_offset,
                            resync_offset,
                            resync_offset - current_offset
                        );
                        current_offset = resync_offset;
                        file.seek(SeekFrom::Start(current_offset)).await?;
                        continue;
                    }
                    None => {
                        return Err(BufferError::CorruptData(format!(
                            "Implausible LEN {} at offset {} (no valid entry found ahead)",
                            len, current_offset
                        )));
                    }
                }
            }

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

            let entry_size = HEADER_SIZE + len + CHECKSUM_SIZE;
            if calculated_checksum.as_slice() != stored_checksum {
                eprintln!(
                    "[WAL] Checksum mismatch at offset {}; skipping corrupted entry ({} bytes)",
                    current_offset, entry_size
                );
                current_offset += entry_size;
                file.seek(SeekFrom::Start(current_offset)).await?;
                continue;
            }

            items.push(data_buf);
            current_offset += entry_size;
        }

        Ok((current_offset, items))
    }

    /// Compact the WAL by removing already-committed data from the front.
    /// Call periodically to prevent unbounded WAL growth.
    pub async fn compact(&self) -> Result<(), BufferError> {
        let committed = self.get_committed_offset().await?;
        if committed == 0 {
            return Ok(());
        }

        let log = self.log_path();
        if !log.exists() {
            return Ok(());
        }

        let file_len = tokio::fs::metadata(&log).await?.len();
        if committed >= file_len {
            tokio::fs::write(&log, &[]).await?;
            self.write_cursor(0).await?;
            return Ok(());
        }

        let remaining = file_len - committed;
        let tmp_log = log.with_extension("compact");

        {
            let mut src = File::open(&log).await?;
            src.seek(SeekFrom::Start(committed)).await?;
            let mut dst = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&tmp_log)
                .await?;
            tokio::io::copy(&mut src, &mut dst).await?;
            dst.sync_all().await?;
        }

        self.write_cursor(0).await?;
        tokio::fs::rename(&tmp_log, &log).await?;

        eprintln!(
            "[WAL] Compacted: removed {} bytes, {} bytes remaining",
            committed, remaining
        );

        Ok(())
    }

    async fn write_cursor(&self, offset: u64) -> Result<(), BufferError> {
        let path = self.cursor_path();
        let tmp_path = path.with_extension("tmp");

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)
            .await?;

        file.write_all(&offset.to_le_bytes()).await?;
        file.sync_all().await?;
        drop(file);

        tokio::fs::rename(&tmp_path, &path).await?;
        Ok(())
    }

    pub async fn commit_offset(&self, offset: u64) -> Result<(), BufferError> {
        self.write_cursor(offset).await?;

        const COMPACT_THRESHOLD: u64 = 100 * 1024 * 1024;
        let log = self.log_path();
        if let Ok(meta) = tokio::fs::metadata(&log).await {
            if meta.len() > COMPACT_THRESHOLD && offset > 0 {
                if let Err(e) = self.compact().await {
                    eprintln!("[WAL] Auto-compact failed: {e}");
                }
            }
        }

        Ok(())
    }
}

/// Verify that a complete, checksum-valid entry begins at `offset`. Returns
/// `Some(end_offset)` when the frame parses and the SHA-256 matches; `None`
/// when anything looks wrong (bad magic, implausible len, EOF before frame
/// end, checksum mismatch). Only I/O errors propagate.
///
/// The file's cursor position is not preserved across this call.
async fn try_parse_entry_at(
    file: &mut File,
    offset: u64,
    file_len: u64,
) -> Result<Option<u64>, BufferError> {
    if offset + HEADER_SIZE > file_len {
        return Ok(None);
    }
    file.seek(SeekFrom::Start(offset)).await?;

    let magic = match file.read_u32_le().await {
        Ok(m) => m,
        Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    if magic != MAGIC {
        return Ok(None);
    }

    let len = match file.read_u32_le().await {
        Ok(l) => l as u64,
        Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    if len > MAX_ENTRY_SIZE {
        return Ok(None);
    }
    if offset + HEADER_SIZE + len + CHECKSUM_SIZE > file_len {
        return Ok(None);
    }

    let mut data = vec![0u8; len as usize];
    match file.read_exact(&mut data).await {
        Ok(_) => {}
        Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    }

    let mut stored_checksum = [0u8; CHECKSUM_SIZE as usize];
    match file.read_exact(&mut stored_checksum).await {
        Ok(_) => {}
        Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    }

    let mut hasher = Sha256::new();
    hasher.update(MAGIC.to_le_bytes());
    hasher.update((len as u32).to_le_bytes());
    hasher.update(&data);
    if hasher.finalize().as_slice() != stored_checksum {
        return Ok(None);
    }

    Ok(Some(offset + HEADER_SIZE + len + CHECKSUM_SIZE))
}

/// Scan forward from `start_offset` looking for the next byte position where a
/// valid, checksum-verifying entry begins. Reads in `SCAN_CHUNK_SIZE`-byte
/// windows with overlap, using `memchr::memmem` to locate the magic byte
/// sequence quickly. Returns `None` if no valid entry exists between
/// `start_offset` and EOF.
async fn scan_for_next_valid_entry(
    file: &mut File,
    start_offset: u64,
    file_len: u64,
) -> Result<Option<u64>, BufferError> {
    let finder = memmem::Finder::new(&MAGIC_BYTES);
    let mut buf = vec![0u8; SCAN_CHUNK_SIZE];
    let mut search_offset = start_offset;

    while search_offset + HEADER_SIZE <= file_len {
        let to_read = ((file_len - search_offset) as usize).min(buf.len());

        file.seek(SeekFrom::Start(search_offset)).await?;
        let mut filled = 0usize;
        while filled < to_read {
            match file.read(&mut buf[filled..to_read]).await {
                Ok(0) => break,
                Ok(n) => filled += n,
                Err(e) => return Err(e.into()),
            }
        }
        if filled < MAGIC_BYTES.len() {
            break;
        }

        let mut local = 0usize;
        while local + MAGIC_BYTES.len() <= filled {
            match finder.find(&buf[local..filled]) {
                Some(idx) => {
                    let candidate = search_offset + (local + idx) as u64;
                    if try_parse_entry_at(file, candidate, file_len).await?.is_some() {
                        return Ok(Some(candidate));
                    }
                    local += idx + 1;
                }
                None => break,
            }
        }

        // Advance past this window, keeping a small overlap so magic bytes that
        // straddle the boundary are still found in the next iteration.
        let advance = (filled as u64).saturating_sub(MAGIC_BYTES.len() as u64 - 1);
        if advance == 0 {
            break;
        }
        search_offset += advance;
    }

    Ok(None)
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

        // Corrupted entry should be skipped, returning empty items
        let (_, items) = buffer.peek_chunk(1).await.unwrap();
        assert!(items.is_empty(), "Corrupted entry should be skipped");
    }

    /// Simulates the production failure: a region of zero bytes in the middle
    /// of the WAL between two valid entries (e.g. a torn write or partially-
    /// allocated file). Before the resync patch this returned `CorruptData`
    /// and the upload cycle aborted. Now we expect the reader to scan past
    /// the zero region and recover `after`.
    #[tokio::test]
    async fn test_resync_past_invalid_magic_region() {
        let tmp = tempdir().unwrap();
        let buffer = DiskBuffer::new(tmp.path()).unwrap();

        buffer.append(b"before").await.unwrap();
        let good_offset_before = tokio::fs::metadata(buffer.log_path())
            .await
            .unwrap()
            .len();

        // Inject ~9 KB of zeros directly into the WAL — simulates a region
        // with no valid magic bytes.
        let zero_run = vec![0u8; 9_000];
        {
            let mut f = OpenOptions::new()
                .append(true)
                .open(buffer.log_path())
                .await
                .unwrap();
            f.write_all(&zero_run).await.unwrap();
            f.flush().await.unwrap();
        }

        buffer.append(b"after").await.unwrap();

        // Resync should skip the zero region and return both records.
        let (next, items) = buffer.peek_chunk(10).await.unwrap();
        assert_eq!(
            items,
            vec![b"before".to_vec(), b"after".to_vec()],
            "expected reader to recover the post-corruption entry"
        );
        // `next` should be at EOF.
        let file_len = tokio::fs::metadata(buffer.log_path())
            .await
            .unwrap()
            .len();
        assert_eq!(next, file_len);
        assert!(good_offset_before < file_len);
    }

    /// Garbage bytes that happen to contain the magic sequence somewhere
    /// without forming a valid entry there must not be treated as a frame.
    /// The reader must keep scanning and recover the genuine post-garbage
    /// entry.
    #[tokio::test]
    async fn test_resync_skips_false_magic_in_garbage() {
        let tmp = tempdir().unwrap();
        let buffer = DiskBuffer::new(tmp.path()).unwrap();

        buffer.append(b"before").await.unwrap();

        // Construct a payload that contains MAGIC_BYTES followed by garbage,
        // appended directly to the WAL (not via `append`, so no checksum).
        let mut garbage = Vec::new();
        garbage.extend_from_slice(&[0xFFu8; 200]);
        garbage.extend_from_slice(&MAGIC_BYTES); // false positive
        garbage.extend_from_slice(&[0xAAu8; 500]); // random following bytes
        {
            let mut f = OpenOptions::new()
                .append(true)
                .open(buffer.log_path())
                .await
                .unwrap();
            f.write_all(&garbage).await.unwrap();
            f.flush().await.unwrap();
        }

        buffer.append(b"after").await.unwrap();

        let (_, items) = buffer.peek_chunk(10).await.unwrap();
        assert_eq!(
            items,
            vec![b"before".to_vec(), b"after".to_vec()],
            "false-positive magic in garbage must not be parsed as a frame"
        );
    }
}
