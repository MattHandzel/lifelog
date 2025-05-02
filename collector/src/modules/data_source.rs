use std::fmt::Debug;
use std::error::Error as StdError;
use async_trait::async_trait;

#[async_trait]
pub trait DataSource {
    type Config: Clone + Debug + Send + Sync + 'static;

    type Error: StdError + Send + Sync + 'static;

    fn start(&mut self) -> Result<(), Self::Error>;

    async fn stop(&mut self) -> Result<(), Self::Error>;

    fn is_running(&self) -> bool;

    fn get_config(&self) -> Self::Config;
}
