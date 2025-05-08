use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;

use crate::logger::LoggerError;
use tokio::task::JoinHandle;

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct DataSourceHandle {
    pub join: JoinHandle<Result<(), DataSourceError>>,
}

#[derive(Debug, thiserror::Error)]
pub enum DataSourceError {
    #[error("Screen Logger Error: {0}")]
    LoggerError(#[from] LoggerError),

    #[error("Logger task panicked or was cancelled: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Logger is already running.")]
    AlreadyRunning,

    #[error("Logger is not running.")]
    NotRunning,

    #[error("Failed to create logger instance during start.")]
    InitializationError(LoggerError),
}

#[async_trait]
pub trait DataSource: Any {
    type Config: Clone + Debug + Send + Sync + 'static;

    fn as_any(&self) -> &(dyn Any + '_) where Self: Sized {
        self
    }

    fn start(&self) -> Result<DataSourceHandle, DataSourceError>;

    async fn stop(&mut self) -> Result<(), DataSourceError>;

    async fn run(&self) -> Result<(), DataSourceError>;

    fn is_running(&self) -> bool;

    fn get_config(&self) -> Self::Config;

    fn new(config: Self::Config) -> Result<Self, DataSourceError>
    where
        Self: Sized;
}
