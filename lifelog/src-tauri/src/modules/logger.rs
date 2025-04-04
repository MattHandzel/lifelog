use async_trait::async_trait;

// Define a trait for logger configuration
pub trait LoggerConfig: Send + Sync {
    fn interval(&self) -> f64;
}

// Generic logger trait that data loggers can implement
#[async_trait]
pub trait DataLogger {
    async fn start<T: LoggerConfig + 'static>(&self, config: &T);
    fn stop(&self);
    fn is_running(&self) -> bool;
}
