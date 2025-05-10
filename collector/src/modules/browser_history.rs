use crate::data_source::*;
use crate::logger::*;
use async_trait::async_trait;
use config::BrowserHistoryConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use data_modalities::browser::BrowserFrame;
use rusqlite::{Connection, Result};
use tokio::time::{sleep, Duration};

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct BrowserHistorySource {
    config: BrowserHistoryConfig,
    pub buffer: Arc<Mutex<Vec<BrowserFrame>>>,
}

impl BrowserHistorySource {
    pub fn new(config: BrowserHistoryConfig) -> Result<Self, DataSourceError> {
        Ok(BrowserHistorySource {
            config,
            buffer: Arc::new(Mutex::new(Vec::new())),
        })
    }
}

#[async_trait]
impl DataSource for BrowserHistorySource {
    type Config = BrowserHistoryConfig;

    fn new(config: BrowserHistoryConfig) -> Result<Self, DataSourceError> {
        BrowserHistorySource::new(config)
    }

    fn start(&self) -> Result<DataSourceHandle, DataSourceError> {
        if RUNNING.load(Ordering::SeqCst) {
            eprintln!("ScreenDataSource: Start called but task is already running.");
            return Err(DataSourceError::AlreadyRunning);
        }

        println!("ScreenDataSource: Starting data source task to store in memory...");
        RUNNING.store(true, Ordering::SeqCst);

        let source_clone = self.clone();

        let join_handle = tokio::spawn(async move {
            let task_result = source_clone.run().await;
            println!(
                "[Task] BrowserHistorySource (in-memory) background task finished with result: {:?}",
                task_result
            );
            task_result
        });

        println!("BrowserHistorySource: Data source task (in-memory) started successfully.");
        let new_join_handle = tokio::spawn(async { Ok(()) });
        Ok(DataSourceHandle {
            join: new_join_handle,
        })
    }

    async fn stop(&mut self) -> Result<(), DataSourceError> {
        RUNNING.store(false, Ordering::SeqCst);
        // FIXME, actually implmenet stop handles
        Ok(())
    }

    async fn run(&self) -> Result<(), DataSourceError> {
        while RUNNING.load(Ordering::SeqCst) {
            sleep(Duration::from_secs_f64(5.0)).await; //fixme
        }
        println!("BrowserHistorySource: In-memory run loop finished.");
        Ok(())
    }

    fn is_running(&self) -> bool {
        RUNNING.load(Ordering::SeqCst)
    }

    fn get_config(&self) -> Self::Config {
        self.config.clone()
    }
}

