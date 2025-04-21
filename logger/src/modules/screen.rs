use async_trait::async_trait;
use chrono::Local;
use config::ScreenConfig;
use rusqlite::params;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{sleep, Duration};
use crate::logger::*;

use crate::setup::setup_screen_db;

static RUNNING: AtomicBool = AtomicBool::new(false);

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
            let _ = logger.run().await;
        });

        Ok(LoggerHandle { join, })
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
        let ts = now.timestamp() as f64
               + now.timestamp_subsec_nanos() as f64 / 1e9;
        let ts_fmt = now.format(&self.config.timestamp_format);
        let out = format!("{}/{}.png",
                          self.config.output_dir.display(),
                          ts_fmt);

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
            let cmd = if cfg!(target_os = "linux") { "grim" }
                      else { "screenshot.exe" };
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