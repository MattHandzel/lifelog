use async_trait::async_trait;
use std::any::Any;
use std::fmt::Debug;

use crate::logger::LoggerError;
use tokio::task::JoinHandle;

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

    #[error("Mutex lock error: {0}")]
    MutexError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[async_trait]
pub trait DataSource: Any {
    type Config: Clone + Debug + Send + Sync + 'static;

    fn as_any(&self) -> &dyn Any;

    fn start(&self) -> Result<DataSourceHandle, DataSourceError>;

    async fn stop(&mut self) -> Result<(), DataSourceError>;

    // I should probably add a get_data func here... would be better
    // or find some way to implmement here rather than in each child class?
    // also a clear buffer function maybe?

    async fn run(&self) -> Result<(), DataSourceError>;

    fn is_running(&self) -> bool;

    fn get_config(&self) -> Self::Config;

    fn new(config: Self::Config) -> Result<Self, DataSourceError>
    where
        Self: Sized;
}
