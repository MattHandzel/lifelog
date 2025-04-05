use async_trait::async_trait;

// Define a trait for logger configuration
pub trait LoggerConfig: Send + Sync {
    fn interval(&self) -> f64;
}

// Generic logger trait that data loggers can implement
#[async_trait]
pub trait DataLogger {
    // This function should be called the first time the logger is created on the machine, this function contains
    // code to set up database tables, create directories if necessary, etc.
    // things such as setting up database tables,
    async fn initialize(&self);

    // This function should be called to start the logger
    // It should be able to run on multiple operating systems
    async fn start<T: LoggerConfig + 'static>(&self, config: &T);

    // This function should be called to stop the logger
    async fn stop(&self);
}
