use async_trait::async_trait;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

use lifelog_core::LifelogError;
use tokio::task::JoinHandle;
use utils::buffer::DiskBuffer;

#[derive(Debug)]
pub struct DataSourceHandle {
    pub join: JoinHandle<Result<(), LifelogError>>,
}

#[async_trait]
pub trait DataSource: Any {
    type Config: Clone + Debug + Send + Sync + 'static;

    fn as_any(&self) -> &dyn Any;

    fn start(&self) -> Result<DataSourceHandle, LifelogError>;

    async fn stop(&mut self) -> Result<(), LifelogError>;

    async fn run(&self) -> Result<(), LifelogError>;

    fn is_running(&self) -> bool;

    fn get_config(&self) -> Self::Config;

    fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>> {
        None
    }

    fn new(config: Self::Config) -> Result<Self, LifelogError>
    where
        Self: Sized;
}

#[async_trait]
pub trait BufferedSource: Send + Sync {
    fn stream_id(&self) -> String;
    /// Returns (new_offset, items) where items are serialized Prost messages ready for upload.
    async fn peek_upload_batch(
        &self,
        max_items: usize,
    ) -> Result<(u64, Vec<Vec<u8>>), LifelogError>;
    async fn commit_upload(&self, offset: u64) -> Result<(), LifelogError>;
    /// Bytes buffered in the WAL but not yet committed as uploaded. Drives state reporting.
    async fn uncommitted_size(&self) -> Result<u64, LifelogError>;
}

/// The single WAL-backed `BufferedSource` shared by every disk-buffered modality.
/// Replaces the byte-identical per-module `XBufferedSource` structs: a module's
/// `get_buffered_source()` just hands back one of these with its stream id + buffer.
pub struct DiskBufferedSource {
    pub stream_id: String,
    pub buffer: Arc<DiskBuffer>,
}

impl DiskBufferedSource {
    pub fn new(stream_id: impl Into<String>, buffer: Arc<DiskBuffer>) -> Self {
        Self {
            stream_id: stream_id.into(),
            buffer,
        }
    }
}

#[async_trait]
impl BufferedSource for DiskBufferedSource {
    fn stream_id(&self) -> String {
        self.stream_id.clone()
    }

    async fn peek_upload_batch(
        &self,
        max_items: usize,
    ) -> Result<(u64, Vec<Vec<u8>>), LifelogError> {
        self.buffer.peek_chunk(max_items).await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })
    }

    async fn commit_upload(&self, offset: u64) -> Result<(), LifelogError> {
        self.buffer.commit_offset(offset).await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })
    }

    async fn uncommitted_size(&self) -> Result<u64, LifelogError> {
        self.buffer.get_uncommitted_size().await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })
    }
}
