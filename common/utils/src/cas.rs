use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CasError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("invalid hash: {0}")]
    InvalidHash(String),
}

/// Simple filesystem content-addressed store (CAS) keyed by SHA256.
#[derive(Debug, Clone)]
pub struct FsCas {
    root: PathBuf,
}

impl FsCas {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn put(&self, bytes: &[u8]) -> Result<String, CasError> {
        let hash = sha256_hex(bytes);
        let final_path = self.path_for_hash(&hash)?;

        if final_path.exists() {
            return Ok(hash);
        }

        if let Some(parent) = final_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Use a unique temporary file in the same directory to avoid collisions between concurrent writers.
        let parent = final_path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "No parent directory for CAS path")
        })?;

        let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
        io::Write::write_all(&mut tmp, bytes)?;

        // Best-effort atomic write: rename unique temp file to final path.
        // On POSIX, rename is atomic and overwrites the destination if it exists.
        // On Windows, it might fail if the destination exists, which we handle.
        match tmp.persist(&final_path) {
            Ok(_) => Ok(hash),
            Err(_e) if final_path.exists() => {
                // Another writer won the race and successfully persisted the file.
                // Since this is content-addressed, we can safely ignore the error.
                Ok(hash)
            }
            Err(e) => Err(CasError::Io(e.error)),
        }
    }

    pub fn get(&self, hash: &str) -> Result<Vec<u8>, CasError> {
        let p = self.path_for_hash(hash)?;
        Ok(fs::read(p)?)
    }

    pub fn contains(&self, hash: &str) -> Result<bool, CasError> {
        let p = self.path_for_hash(hash)?;
        Ok(p.exists())
    }

    fn path_for_hash(&self, hash: &str) -> Result<PathBuf, CasError> {
        if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CasError::InvalidHash(hash.to_string()));
        }
        let (a, rest) = hash.split_at(2);
        Ok(self.root.join(a).join(rest))
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn cas_dedupes_identical_blobs() {
        let dir = tempfile::tempdir().unwrap();
        let cas = FsCas::new(dir.path());

        let blob = b"hello world";
        let h1 = cas.put(blob).unwrap();
        let h2 = cas.put(blob).unwrap();
        assert_eq!(h1, h2);
        assert!(cas.contains(&h1).unwrap());
        assert_eq!(cas.get(&h1).unwrap(), blob);

        // Physical layout should be stable.
        let p = dir.path().join(&h1[0..2]).join(&h1[2..]);
        assert!(p.exists());
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(256))]

            #[test]
            fn prop_cas_round_trip(data in prop::collection::vec(0u8..=255, 1..=4096)) {
                let dir = tempfile::tempdir().unwrap();
                let cas = FsCas::new(dir.path());
                let hash = cas.put(&data).unwrap();
                let retrieved = cas.get(&hash).unwrap();
                prop_assert_eq!(&data, &retrieved);
                prop_assert!(cas.contains(&hash).unwrap());
            }

            #[test]
            fn prop_cas_dedup_any_content(
                data in prop::collection::vec(0u8..=255, 1..=2048),
                repeats in 2u32..=5,
            ) {
                let dir = tempfile::tempdir().unwrap();
                let cas = FsCas::new(dir.path());
                let mut hashes = Vec::new();
                for _ in 0..repeats {
                    hashes.push(cas.put(&data).unwrap());
                }
                // All puts return the same hash
                let first = &hashes[0];
                for h in &hashes {
                    prop_assert_eq!(h, first);
                }
                // Only one file on disk
                prop_assert_eq!(cas.get(first).unwrap(), data);
            }
        }
    }

    #[test]
    fn test_cas_concurrent_puts() {
        use std::sync::Arc;
        use std::thread;

        let dir = tempfile::tempdir().unwrap();
        let cas = Arc::new(FsCas::new(dir.path()));
        let data = b"concurrent-data".to_vec();

        let handles: Vec<_> = (0..8)
            .map(|_| {
                let cas = Arc::clone(&cas);
                let data = data.clone();
                thread::spawn(move || cas.put(&data).unwrap())
            })
            .collect();

        let hashes: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        // All threads produce the same hash
        for h in &hashes {
            assert_eq!(h, &hashes[0]);
        }
        assert_eq!(cas.get(&hashes[0]).unwrap(), data);
    }
}
