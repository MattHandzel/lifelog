use crate::data_source::*;
use async_trait::async_trait;
use chrono::Utc;
use config::BrowserHistoryConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{fs, io::Read};

use data_modalities::browser::BrowserFrame;
use lifelog_core::Uuid;
use lifelog_proto::to_pb_ts;
use rusqlite::Connection;
use tokio::time::{sleep, Duration};

static RUNNING: AtomicBool = AtomicBool::new(false);
// Chrome uses windows epoch, not unix
const WINDOWS_EPOCH_MICROS: i64 = 11644473600000000;

#[derive(Debug, Clone)]
pub struct BrowserHistorySource {
    config: BrowserHistoryConfig,
}

impl BrowserHistorySource {
    pub fn new(config: BrowserHistoryConfig) -> Result<Self, DataSourceError> {
        Ok(BrowserHistorySource { config })
    }

    pub fn get_data(&mut self) -> Result<Vec<BrowserFrame>, DataSourceError> {
        let last_query = match fs::File::open(&self.config.output_file) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .map_err(DataSourceError::Io)?;
                contents
                    .parse::<i64>()
                    .map(|ts_micros| {
                        let unix_micros = ts_micros - WINDOWS_EPOCH_MICROS;
                        let unix_secs = unix_micros / 1_000_000;
                        let unix_nanos = (unix_micros % 1_000_000) * 1_000;
                        ::chrono::DateTime::from_timestamp(unix_secs, unix_nanos as u32)
                            .unwrap_or_default()
                    })
                    .unwrap_or_else(|_| Utc::now())
            }
            Err(_) => {
                let windows_epoch_micros_str = WINDOWS_EPOCH_MICROS.to_string();
                let windows_epoch_dt = ::chrono::DateTime::from_timestamp(
                    WINDOWS_EPOCH_MICROS / 1_000_000,
                    ((WINDOWS_EPOCH_MICROS % 1_000_000) * 1_000) as u32,
                )
                .unwrap_or_default();
                if let Err(e) = fs::write(&self.config.output_file, &windows_epoch_micros_str) {
                    tracing::error!(error = %e, "Error creating output file");
                }
                windows_epoch_dt
            }
        };

        let history_path = &self.config.input_file;
        let ts = Utc::now();

        let last_query_chrome_micros = (last_query.timestamp() * 1_000_000)
            + last_query.timestamp_subsec_micros() as i64
            + WINDOWS_EPOCH_MICROS;
        let now_chrome_micros = (ts.timestamp() * 1_000_000)
            + ts.timestamp_subsec_micros() as i64
            + WINDOWS_EPOCH_MICROS;

        let conn = Connection::open(history_path)?;

        let mut stmt = conn.prepare(
            "SELECT urls.url, title, visit_time, visit_count FROM urls INNER JOIN visits ON urls.id = visits.url WHERE visit_time > ? AND visit_time <= ?"
        )?;

        let history_iter =
            stmt.query_map([last_query_chrome_micros, now_chrome_micros], |row| {
                Ok(BrowserFrame {
                    uuid: Uuid::new_v4().to_string(), //use v6
                    url: row.get::<_, String>(0)?,
                    title: row.get::<_, String>(1)?,
                    timestamp: {
                        let visit_time_chrome_micros: i64 = row.get(2)?;
                        let unix_micros = visit_time_chrome_micros - WINDOWS_EPOCH_MICROS;
                        let unix_secs = unix_micros / 1_000_000;
                        let unix_nanos = (unix_micros % 1_000_000) * 1_000;
                        let ts = ::chrono::DateTime::from_timestamp(unix_secs, unix_nanos as u32)
                            .unwrap_or_default();
                        to_pb_ts(ts)
                    },
                    visit_count: row.get::<_, u32>(3)?,
                })
            })?;

        let mut history_entries = Vec::new();
        for entry in history_iter {
            history_entries.push(entry?);
        }

        if let Err(e) = fs::write(&self.config.output_file, now_chrome_micros.to_string()) {
            tracing::error!(error = %e, "Error saving last query time (Chrome format)");
        }

        Ok(history_entries)
    }
}

#[async_trait]
impl DataSource for BrowserHistorySource {
    type Config = BrowserHistoryConfig;

    fn new(config: BrowserHistoryConfig) -> Result<Self, DataSourceError> {
        BrowserHistorySource::new(config)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn start(&self) -> Result<DataSourceHandle, DataSourceError> {
        if RUNNING.load(Ordering::SeqCst) {
            tracing::warn!("BrowserHistorySource: Start called but task is already running.");
            return Err(DataSourceError::AlreadyRunning);
        }

        tracing::info!("BrowserHistorySource: Starting data source task to store in memory");
        RUNNING.store(true, Ordering::SeqCst);

        let source_clone = self.clone();

        let _join_handle = tokio::spawn(async move {
            let task_result = source_clone.run().await;
            tracing::info!(result = ?task_result, "BrowserHistorySource (in-memory) background task finished");
            task_result
        });

        tracing::info!("BrowserHistorySource: Data source task (in-memory) started successfully");
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
        tracing::info!("BrowserHistorySource: In-memory run loop finished");
        Ok(())
    }

    fn is_running(&self) -> bool {
        RUNNING.load(Ordering::SeqCst)
    }

    fn get_config(&self) -> Self::Config {
        self.config.clone()
    }
}
