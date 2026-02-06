use crate::cas::FsCas;
use crate::chunk::{ChunkError, ChunkOffsetValidator, OffsetPolicy};
use async_trait::async_trait;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum IngestError {
    #[error("chunk validation error: {0}")]
    Validation(#[from] ChunkError),
    #[error("backend error: {0}")]
    Backend(String),
}

/// Interface for the backend storage and indexing system.
#[async_trait]
pub trait IngestBackend {
    /// Persist chunk metadata. Should be idempotent based on (collector, stream, session, offset).
    async fn persist_metadata(
        &self,
        collector_id: &str,
        stream_id: &str,
        session_id: u64,
        offset: u64,
        length: u64,
        hash: &str,
    ) -> Result<(), String>;

    /// Check if the chunk at the given offset is fully indexed and queryable.
    async fn is_indexed(
        &self,
        collector_id: &str,
        stream_id: &str,
        session_id: u64,
        offset: u64,
    ) -> bool;
}

pub struct ChunkIngester<B: IngestBackend> {
    validator: ChunkOffsetValidator,
    cas: FsCas,
    backend: B,
    collector_id: String,
    stream_id: String,
    session_id: u64,
}

impl<B: IngestBackend> ChunkIngester<B> {
    pub fn new(
        backend: B,
        cas: FsCas,
        collector_id: String,
        stream_id: String,
        session_id: u64,
        start_offset: u64,
    ) -> Self {
        Self {
            validator: ChunkOffsetValidator::new(start_offset),
            cas,
            backend,
            collector_id,
            stream_id,
            session_id,
        }
    }

    /// Applies a chunk: validates, stores in CAS, and persists metadata.
    /// Returns the next expected offset.
    pub async fn apply_chunk(
        &mut self,
        offset: u64,
        bytes: &[u8],
        hash: &str,
    ) -> Result<u64, IngestError> {
        // UT-040: Validate offset and hash
        let next_offset = self.validator.validate_chunk(
            offset,
            bytes,
            hash,
            OffsetPolicy::Resume {
                allow_offset: offset,
            },
        )?;

        // UT-050: Store in CAS (deduplicated by FsCas)
        self.cas
            .put(bytes)
            .map_err(|e| IngestError::Backend(e.to_string()))?;

        // UT-041: Idempotent metadata persistence
        self.backend
            .persist_metadata(
                &self.collector_id,
                &self.stream_id,
                self.session_id,
                offset,
                bytes.len() as u64,
                hash,
            )
            .await
            .map_err(IngestError::Backend)?;

        Ok(next_offset)
    }

    /// Checks if the chunk at the given offset is indexed.
    pub async fn is_chunk_indexed(&self, offset: u64) -> bool {
        self.backend
            .is_indexed(&self.collector_id, &self.stream_id, self.session_id, offset)
            .await
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::bool_assert_comparison
)]
mod tests {
    use super::*;
    use crate::cas::sha256_hex;
    use std::collections::HashSet;
    use std::sync::Mutex;

    struct MockBackend {
        persisted: Mutex<HashSet<(String, String, u64, u64)>>,
        indexed: Mutex<HashSet<(String, String, u64, u64)>>,
    }

    impl MockBackend {
        fn new() -> Self {
            Self {
                persisted: Mutex::new(HashSet::new()),
                indexed: Mutex::new(HashSet::new()),
            }
        }
    }

    #[async_trait]
    impl IngestBackend for &MockBackend {
        async fn persist_metadata(
            &self,
            collector_id: &str,
            stream_id: &str,
            session_id: u64,
            offset: u64,
            _length: u64,
            _hash: &str,
        ) -> Result<(), String> {
            let mut p = self.persisted.lock().unwrap();
            p.insert((
                collector_id.to_string(),
                stream_id.to_string(),
                session_id,
                offset,
            ));
            Ok(())
        }

        async fn is_indexed(
            &self,
            collector_id: &str,
            stream_id: &str,
            session_id: u64,
            offset: u64,
        ) -> bool {
            let i = self.indexed.lock().unwrap();
            i.contains(&(
                collector_id.to_string(),
                stream_id.to_string(),
                session_id,
                offset,
            ))
        }
    }

    #[tokio::test]
    async fn ut_041_idempotent_chunk_apply() {
        let dir = tempfile::tempdir().unwrap();
        let cas = FsCas::new(dir.path());
        let backend = MockBackend::new();
        let mut ingester = ChunkIngester::new(&backend, cas, "c1".into(), "s1".into(), 123, 0);

        let data = b"chunk1";
        let hash = sha256_hex(data);

        // First apply
        ingester.apply_chunk(0, data, &hash).await.unwrap();
        assert_eq!(backend.persisted.lock().unwrap().len(), 1);

        // Second apply (same chunk)
        // Resume policy allows same offset
        ingester.apply_chunk(0, data, &hash).await.unwrap();

        // Assert: still only one record in "database" (mock)
        assert_eq!(backend.persisted.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn ut_042_durable_ack_gate() {
        let dir = tempfile::tempdir().unwrap();
        let cas = FsCas::new(dir.path());
        let backend = MockBackend::new();
        let mut ingester = ChunkIngester::new(&backend, cas, "c1".into(), "s1".into(), 123, 0);

        let data = b"chunk1";
        let hash = sha256_hex(data);
        ingester.apply_chunk(0, data, &hash).await.unwrap();

        // Before indexing: should be false
        assert_eq!(ingester.is_chunk_indexed(0).await, false);

        // After indexing (mock backend update)
        // Note: we insert '0' because that is the chunk start offset
        backend
            .indexed
            .lock()
            .unwrap()
            .insert(("c1".into(), "s1".into(), 123, 0));
        assert_eq!(ingester.is_chunk_indexed(0).await, true);
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(64))]

            #[test]
            fn prop_ingester_offset_tracking(
                chunks in prop::collection::vec(prop::collection::vec(1u8..=255, 1..=256), 1..=10),
            ) {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    let dir = tempfile::tempdir().unwrap();
                    let cas = FsCas::new(dir.path());
                    let backend = MockBackend::new();
                    let mut ingester = ChunkIngester::new(
                        &backend, cas, "prop-c".into(), "prop-s".into(), 42, 0,
                    );

                    let mut expected_offset = 0u64;
                    for chunk in &chunks {
                        let hash = sha256_hex(chunk);
                        let next = ingester.apply_chunk(expected_offset, chunk, &hash).await.unwrap();
                        expected_offset += chunk.len() as u64;
                        prop_assert_eq!(next, expected_offset);
                    }

                    // All chunks should be persisted
                    let persisted = backend.persisted.lock().unwrap();
                    prop_assert_eq!(persisted.len(), chunks.len());
                    Ok(())
                })?;
            }
        }
    }

    #[tokio::test]
    async fn test_ingester_backend_error_propagated() {
        use std::sync::atomic::{AtomicBool, Ordering};

        struct FailBackend {
            should_fail: AtomicBool,
        }

        #[async_trait]
        impl IngestBackend for &FailBackend {
            async fn persist_metadata(
                &self,
                _collector_id: &str,
                _stream_id: &str,
                _session_id: u64,
                _offset: u64,
                _length: u64,
                _hash: &str,
            ) -> Result<(), String> {
                if self.should_fail.load(Ordering::Relaxed) {
                    Err("simulated backend failure".into())
                } else {
                    Ok(())
                }
            }

            async fn is_indexed(
                &self, _: &str, _: &str, _: u64, _: u64,
            ) -> bool {
                false
            }
        }

        let dir = tempfile::tempdir().unwrap();
        let cas = FsCas::new(dir.path());
        let backend = FailBackend {
            should_fail: AtomicBool::new(true),
        };
        let mut ingester = ChunkIngester::new(&backend, cas, "c".into(), "s".into(), 1, 0);

        let data = b"test-data";
        let hash = sha256_hex(data);
        let err = ingester.apply_chunk(0, data, &hash).await.unwrap_err();
        assert!(matches!(err, IngestError::Backend(_)));
    }
}
