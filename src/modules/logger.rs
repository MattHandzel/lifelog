use async_trait::async_trait;
use tokio::time::{sleep, Duration};

#[async_trait]
trait DataLogger<T: LoggerConfig>: Send + Sync {
    fn connect_to_database(&self, config: T);

    fn log(&self, config: T);
    async fn start_logger(&self, config: T) {
        loop {
            self.log(config);

            sleep(Duration::from_secs_f64(config.interval)).await;
        }
    }
}
