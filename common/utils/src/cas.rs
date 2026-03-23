use chacha20poly1305::aead::Aead;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::PathBuf;

const ENCRYPTION_MAGIC: &[u8; 4] = b"ECAS";
const NONCE_LEN: usize = 24;

#[derive(Debug, thiserror::Error)]
pub enum CasError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("invalid hash: {0}")]
    InvalidHash(String),
    #[error("encryption error")]
    Encryption,
    #[error("decryption error")]
    Decryption,
    #[error("encrypted blob but no key configured")]
    MissingKey,
}

pub struct FsCas {
    root: PathBuf,
    key: Option<[u8; 32]>,
    cipher: Option<XChaCha20Poly1305>,
}

impl std::fmt::Debug for FsCas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FsCas")
            .field("root", &self.root)
            .field("encrypted", &self.key.is_some())
            .finish()
    }
}

impl Clone for FsCas {
    fn clone(&self) -> Self {
        match &self.key {
            Some(k) => Self::with_key(self.root.clone(), k),
            None => Self::new(self.root.clone()),
        }
    }
}

impl FsCas {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            key: None,
            cipher: None,
        }
    }

    pub fn with_key(root: impl Into<PathBuf>, key: &[u8; 32]) -> Self {
        Self {
            root: root.into(),
            key: Some(*key),
            cipher: Some(XChaCha20Poly1305::new(key.into())),
        }
    }

    pub fn with_key_file(
        root: impl Into<PathBuf>,
        key_file: &std::path::Path,
    ) -> Result<Self, CasError> {
        let key_bytes = fs::read(key_file)?;
        if key_bytes.len() < 32 {
            return Err(CasError::Encryption);
        }
        let key: [u8; 32] = key_bytes[..32]
            .try_into()
            .map_err(|_| CasError::Encryption)?;
        Ok(Self::with_key(root, &key))
    }

    pub fn put(&self, bytes: &[u8]) -> Result<String, CasError> {
        let hash = sha256_hex(bytes);
        let final_path = self.path_for_hash(&hash)?;

        if final_path.exists() {
            return Ok(hash);
        }

        let on_disk = self.maybe_encrypt(bytes)?;

        if let Some(parent) = final_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let parent = final_path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "No parent directory for CAS path")
        })?;

        let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
        io::Write::write_all(&mut tmp, &on_disk)?;

        match tmp.persist(&final_path) {
            Ok(_) => Ok(hash),
            Err(_e) if final_path.exists() => Ok(hash),
            Err(e) => Err(CasError::Io(e.error)),
        }
    }

    pub fn get(&self, hash: &str) -> Result<Vec<u8>, CasError> {
        let p = self.path_for_hash(hash)?;
        let raw = fs::read(p)?;
        self.maybe_decrypt(&raw)
    }

    pub fn contains(&self, hash: &str) -> Result<bool, CasError> {
        let p = self.path_for_hash(hash)?;
        Ok(p.exists())
    }

    pub fn remove(&self, hash: &str) -> Result<(), CasError> {
        let p = self.path_for_hash(hash)?;
        if p.exists() {
            fs::remove_file(p)?;
        }
        Ok(())
    }

    fn maybe_encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CasError> {
        let cipher = match &self.cipher {
            Some(c) => c,
            None => return Ok(plaintext.to_vec()),
        };

        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = chacha20poly1305::XNonce::from(nonce_bytes);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| CasError::Encryption)?;

        let mut out = Vec::with_capacity(ENCRYPTION_MAGIC.len() + NONCE_LEN + ciphertext.len());
        out.extend_from_slice(ENCRYPTION_MAGIC);
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    fn maybe_decrypt(&self, data: &[u8]) -> Result<Vec<u8>, CasError> {
        if !data.starts_with(ENCRYPTION_MAGIC) {
            return Ok(data.to_vec());
        }

        let cipher = match &self.cipher {
            Some(c) => c,
            None => return Err(CasError::MissingKey),
        };

        let header_len = ENCRYPTION_MAGIC.len() + NONCE_LEN;
        if data.len() < header_len {
            return Err(CasError::Decryption);
        }

        let nonce_bytes: [u8; NONCE_LEN] = data[ENCRYPTION_MAGIC.len()..header_len]
            .try_into()
            .map_err(|_| CasError::Decryption)?;
        let nonce = chacha20poly1305::XNonce::from(nonce_bytes);
        let ciphertext = &data[header_len..];

        cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|_| CasError::Decryption)
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
    fn encrypted_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let key = [42u8; 32];
        let cas = FsCas::with_key(dir.path(), &key);

        let blob = b"secret data";
        let hash = cas.put(blob).unwrap();
        let retrieved = cas.get(&hash).unwrap();
        assert_eq!(retrieved, blob);
    }

    #[test]
    fn ciphertext_differs_from_plaintext() {
        let dir = tempfile::tempdir().unwrap();
        let key = [42u8; 32];
        let cas = FsCas::with_key(dir.path(), &key);

        let blob = b"secret data that should be encrypted";
        let hash = cas.put(blob).unwrap();

        let on_disk_path = dir.path().join(&hash[0..2]).join(&hash[2..]);
        let raw = std::fs::read(on_disk_path).unwrap();
        assert_ne!(raw, blob);
        assert!(raw.starts_with(b"ECAS"));
    }

    #[test]
    fn backward_compat_reads_unencrypted() {
        let dir = tempfile::tempdir().unwrap();
        let plain_cas = FsCas::new(dir.path());
        let blob = b"old plaintext blob";
        let hash = plain_cas.put(blob).unwrap();

        let key = [42u8; 32];
        let encrypted_cas = FsCas::with_key(dir.path(), &key);
        let retrieved = encrypted_cas.get(&hash).unwrap();
        assert_eq!(retrieved, blob);
    }

    #[test]
    fn encrypted_blob_without_key_errors() {
        let dir = tempfile::tempdir().unwrap();
        let key = [42u8; 32];
        let cas_enc = FsCas::with_key(dir.path(), &key);

        let blob = b"secret";
        let hash = cas_enc.put(blob).unwrap();

        let cas_plain = FsCas::new(dir.path());
        let err = cas_plain.get(&hash).unwrap_err();
        assert!(matches!(err, CasError::MissingKey));
    }

    #[test]
    fn with_key_file_works() {
        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join("test.key");
        std::fs::write(&key_path, [7u8; 32]).unwrap();

        let cas = FsCas::with_key_file(dir.path().join("store"), &key_path).unwrap();
        let hash = cas.put(b"via key file").unwrap();
        assert_eq!(cas.get(&hash).unwrap(), b"via key file");
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
