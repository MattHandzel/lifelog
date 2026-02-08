use async_trait::async_trait;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

use lifelog_core::LifelogError;
use tokio::task::JoinHandle;

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
}