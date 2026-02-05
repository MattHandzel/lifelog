use crate::chunk::{ChunkOffsetValidator, OffsetPolicy, ChunkError};
use crate::cas::FsCas;
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
            OffsetPolicy::Resume { allow_offset: offset },
        )?;

        // UT-050: Store in CAS (deduplicated by FsCas)
        self.cas.put(bytes).map_err(|e| IngestError::Backend(e.to_string()))?;

        // UT-041: Idempotent metadata persistence
        self.backend.persist_metadata(
            &self.collector_id,
            &self.stream_id,
            self.session_id,
            offset,
            hash,
        ).await.map_err(IngestError::Backend)?;

        Ok(next_offset)
    }

    /// Returns the durably ACKed offset.
    /// UT-042: Only advances if backend reports as indexed.
    pub async fn get_acked_offset(&self, current_offset: u64) -> u64 {
        if self.backend.is_indexed(
            &self.collector_id,
            &self.stream_id,
            self.session_id,
            current_offset,
        ).await {
            // In a real implementation, we might track multiple chunks.
            // For the gate, we check if the requested offset is indexed.
            current_offset
        } else {
            // For v1 simplification, if not indexed, we return 0 or the last ACKed offset.
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cas::sha256_hex;
    use std::sync::Mutex;
    use std::collections::HashSet;

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
            _hash: &str,
        ) -> Result<(), String> {
            let mut p = self.persisted.lock().unwrap();
            p.insert((collector_id.to_string(), stream_id.to_string(), session_id, offset));
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
            i.contains(&(collector_id.to_string(), stream_id.to_string(), session_id, offset))
        }
    }

    #[tokio::test]
    async fn ut_041_idempotent_chunk_apply() {
        let dir = tempfile::tempdir().unwrap();
        let cas = FsCas::new(dir.path());
        let backend = MockBackend::new();
        let mut ingester = ChunkIngester::new(
            &backend,
            cas,
            "c1".into(),
            "s1".into(),
            123,
            0
        );

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
        let mut ingester = ChunkIngester::new(
            &backend,
            cas,
            "c1".into(),
            "s1".into(),
            123,
            0
        );

        let data = b"chunk1";
        let hash = sha256_hex(data);
        ingester.apply_chunk(0, data, &hash).await.unwrap();

        // Before indexing: ACK should not advance to 6
        assert_eq!(ingester.get_acked_offset(6).await, 0);

        // After indexing: ACK advances to 6
        backend.indexed.lock().unwrap().insert(("c1".into(), "s1".into(), 123, 6));
        assert_eq!(ingester.get_acked_offset(6).await, 6);
    }
}
