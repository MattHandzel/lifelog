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

        // Best-effort atomic write: write temp file then rename.
        let tmp_path = final_path.with_extension("tmp");
        fs::write(&tmp_path, bytes)?;
        match fs::rename(&tmp_path, &final_path) {
            Ok(()) => Ok(hash),
            Err(_e) if final_path.exists() => {
                // Another writer won the race; clean up temp.
                let _ = fs::remove_file(&tmp_path);
                Ok(hash)
            }
            Err(e) => Err(CasError::Io(e)),
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
}
