use async_trait::async_trait;
use lifelog_core::LifelogError;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct LoggerHandle {
    pub join: JoinHandle<Result<(), LifelogError>>,
}

// Generic logger trait that data loggers can implement
#[async_trait]
pub trait DataLogger: Sized + Send + Sync {
    type Config: Send + Sync + 'static;
    // This function should be called the first time the logger is created on the machine, this function contains
    // code to set up database tables, create directories if necessary, etc.
    // things such as setting up database tables, directories
    fn setup(&self, config: Self::Config) -> Result<LoggerHandle, LifelogError>;

    // This function should be called to start the logger
    // It should be able to run on multiple operating systems
    async fn run(&self) -> Result<(), LifelogError>;

    fn stop(&self);

    // Given the data storing method, it will log the data
    async fn log_data(&self) -> Result<Vec<u8>, LifelogError>;

    fn new(config: Self::Config) -> Result<Self, LifelogError>
    where
        Self: Sized;
}
