use crate::logger::*;
use async_trait::async_trait;
use chrono::Local;
use config::ScreenConfig;
use rusqlite::params;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{sleep, Duration};

use crate::setup::setup_screen_db;
use crate::data_source::DataSource;

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, thiserror::Error)]
pub enum ScreenDataSourceError {
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

#[derive(Debug)]
pub struct ScreenDataSource {
    config: ScreenConfig,
    handle: Option<LoggerHandle>,
}

impl ScreenDataSource {
    pub fn new(config: ScreenConfig) -> Self {
        ScreenDataSource {
            config,
            handle: None,
        }
    }
}

#[async_trait]
impl DataSource for ScreenDataSource {
    type Config = ScreenConfig;
    type Error = ScreenDataSourceError;

    fn start(&mut self) -> Result<(), Self::Error> {
        if self.handle.is_some() {
            eprintln!("ScreenDataSource: Start called but logger is already running.");
            return Err(ScreenDataSourceError::AlreadyRunning);
        }

        println!("ScreenDataSource: Starting logger...");
        let logger = ScreenLogger::new(self.config.clone())
            .map_err(ScreenDataSourceError::InitializationError)?;

        let handle = logger.setup()
             .map_err(ScreenDataSourceError::LoggerError)?;
        self.handle = Some(handle);

        println!("ScreenDataSource: Logger started successfully.");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Self::Error> {
        if let Some(handle) = self.handle.take() {
            println!("ScreenDataSource: Stopping logger...");

            RUNNING.store(false, Ordering::SeqCst);
            match handle.join.await {
                Ok(Ok(())) => {
                    println!("ScreenDataSource: Logger stopped successfully (task returned Ok).");
                    Ok(())
                }
                Ok(Err(logger_err)) => {
                    eprintln!("ScreenDataSource: Logger task finished with error: {}", logger_err);
                    Err(ScreenDataSourceError::LoggerError(logger_err))
                }

                Err(join_err) => {
                    eprintln!("ScreenDataSource: Logger task failed to join (panic/cancel): {}", join_err);
                    Err(ScreenDataSourceError::JoinError(join_err))
                }
            }
        } else {
            eprintln!("ScreenDataSource: Stop called but logger was not running.");
            Err(ScreenDataSourceError::NotRunning)
        }
    }

    fn is_running(&self) -> bool {
        self.handle.is_some()
    }

    fn get_config(&self) -> Self::Config {
        self.config.clone()
    }
}

pub struct ScreenLogger {
    config: ScreenConfig,
}

impl ScreenLogger {
    pub fn new(config: ScreenConfig) -> Result<Self, LoggerError> {
        Ok(ScreenLogger { config })
    }

    pub fn setup(&self) -> Result<LoggerHandle, LoggerError> {
        DataLogger::setup(self, self.config.clone())
    }
}

#[async_trait]
impl DataLogger for ScreenLogger {
    type Config = ScreenConfig;

    fn new(config: ScreenConfig) -> Result<Self, LoggerError> {
        ScreenLogger::new(config)
    }

    fn setup(&self, config: ScreenConfig) -> Result<LoggerHandle, LoggerError> {
        let logger = Self::new(config)?;
        let join = tokio::spawn(async move {

            let task_result = logger.run().await;

            println!("[Task] Background task finished with result: {:?}", task_result);

            task_result
        });

        Ok(LoggerHandle { join })
    }

    async fn run(&self) -> Result<(), LoggerError> {
        RUNNING.store(true, Ordering::SeqCst);
        while RUNNING.load(Ordering::SeqCst) {
            self.log_data().await?;
            sleep(Duration::from_secs_f64(self.config.interval)).await;
        }
        Ok(())
    }

    fn stop(&self) {
        RUNNING.store(false, Ordering::SeqCst);
    }

    async fn log_data(&self) -> Result<(), LoggerError> {
        let conn = setup_screen_db(Path::new(&self.config.output_dir))?;
        let now = Local::now();
        let ts = now.timestamp() as f64 + now.timestamp_subsec_nanos() as f64 / 1e9;
        let ts_fmt = now.format(&self.config.timestamp_format);
        let out = format!("{}/{}.png", self.config.output_dir.display(), ts_fmt);

        #[cfg(target_os = "macos")]
        {
            Command::new("screencapture")
                .arg("-x")
                .arg("-t")
                .arg("png")
                .arg(&out)
                .status()
                .map_err(LoggerError::Io)?;
        }
        #[cfg(not(target_os = "macos"))]
        {
            let cmd = if cfg!(target_os = "linux") {
                "grim"
            } else {
                "screenshot.exe"
            };
            Command::new(cmd)
                .arg("-t")
                .arg("png")
                .arg(&out)
                .status()
                .map_err(LoggerError::Io)?;
        }

        conn.execute("INSERT INTO screen VALUES (?1)", params![ts])?;
        Ok(())
    }
}
