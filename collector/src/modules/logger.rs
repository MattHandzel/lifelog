use async_trait::async_trait;
use thiserror::Error;
use tokio::task::JoinHandle;

#[derive(Debug, Error)]
pub enum LoggerError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Failed to create temporary file: {0}")]
    TempFileError(#[from] tempfile::PersistError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

#[derive(Debug)]
pub struct LoggerHandle {
    pub join: JoinHandle<Result<(), LoggerError>>,
}

// Generic logger trait that data loggers can implement
#[async_trait]
pub trait DataLogger: Sized + Send + Sync {
    type Config: Send + Sync + 'static;
    // This function should be called the first time the logger is created on the machine, this function contains
    // code to set up database tables, create directories if necessary, etc.
    // things such as setting up database tables, directories
    fn setup(&self, config: Self::Config) -> Result<LoggerHandle, LoggerError>;

    // This function should be called to start the logger
    // It should be able to run on multiple operating systems
    async fn run(&self) -> Result<(), LoggerError>;

    fn stop(&self);

    // Given the data storing method, it will log the data
    async fn log_data(&self) -> Result<Vec<u8>, LoggerError>;

    fn new(config: Self::Config) -> Result<Self, LoggerError>
    where
        Self: Sized;
}
