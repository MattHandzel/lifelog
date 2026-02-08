use crate::data_source::{BufferedSource, DataSource, DataSourceHandle};
use async_trait::async_trait;
use config::ProcessesConfig;
use lifelog_core::{LifelogError, Utc, Uuid};
use lifelog_types::{to_pb_ts, ProcessFrame, ProcessInfo};
use prost::Message;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
#[cfg(target_os = "linux")]
use users::{Users, UsersCache};
use utils::buffer::DiskBuffer;

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct ProcessDataSource {
    config: ProcessesConfig,
    pub buffer: Arc<DiskBuffer>,
}

impl ProcessDataSource {
    pub fn new(config: ProcessesConfig) -> Result<Self, LifelogError> {
        let buffer_path = std::path::Path::new(&config.output_dir).join("buffer");
        let buffer = DiskBuffer::new(&buffer_path).map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        Ok(ProcessDataSource {
            config,
            buffer: Arc::new(buffer),
        })
    }

    // Helper to get processes. Kept similar to original logic but returns Proto types.
    #[cfg(target_os = "linux")]
    fn get_process_info(users_cache: &UsersCache) -> Result<Vec<ProcessInfo>, std::io::Error> {
        let mut processes = Vec::new();

        for entry in fs::read_dir("/proc")? {
            let entry = entry?;
            let file_name = entry.file_name();
            let pid_str = file_name.to_string_lossy();

            // Only process numeric directories
            if let Ok(pid) = pid_str.parse::<i32>() {
                let mut process = ProcessInfo {
                    pid,
                    ppid: 0,
                    name: String::new(),
                    exe: String::new(),
                    cmdline: String::new(),
                    status: String::new(),
                    cpu_usage: 0.0,
                    memory_usage: 0,
                    threads: 0,
                    user: String::new(),
                    start_time: 0.0,
                };

                // Read status file
                if let Ok(status) = fs::read_to_string(entry.path().join("status")) {
                    for line in status.lines() {
                        let mut parts = line.split_whitespace();
                        match parts.next() {
                            Some("Name:") => process.name = parts.collect::<Vec<_>>().join(" "),
                            Some("PPid:") => {
                                process.ppid = parts.next().unwrap_or("0").parse().unwrap_or(0)
                            }
                            Some("Uid:") => {
                                if let Some(uid) = parts.next().and_then(|s| s.parse().ok()) {
                                    if let Some(user) = users_cache.get_user_by_uid(uid) {
                                        process.user = user.name().to_string_lossy().into_owned();
                                    }
                                }
                            }
                            Some("Threads:") => {
                                process.threads = parts.next().unwrap_or("0").parse().unwrap_or(0)
                            }
                            Some("VmRSS:") => {
                                process.memory_usage =
                                    parts.next().and_then(|s| s.parse().ok()).unwrap_or(0)
                            }
                            Some("State:") => {
                                let state = parts.next().unwrap_or("?");
                                process.status = match state {
                                    "R" => "Running".to_string(),
                                    "S" => "Sleeping".to_string(),
                                    "D" => "Disk Sleep".to_string(),
                                    "Z" => "Zombie".to_string(),
                                    "T" => "Stopped".to_string(),
                                    "t" => "Tracing Stop".to_string(),
                                    "X" => "Dead".to_string(),
                                    "P" => "Parked".to_string(),
                                    _ => format!("Unknown ({})", state),
                                };
                            }
                            _ => {}
                        }
                    }
                }

                // Read exe symlink
                if let Ok(p) = fs::read_link(entry.path().join("exe")) {
                    if let Some(s) = p.to_str() {
                        process.exe = s.to_string();
                    }
                }

                // Read cmdline
                if let Ok(cmdline) = fs::read(entry.path().join("cmdline")) {
                    let s = String::from_utf8_lossy(&cmdline)
                        .replace('\0', " ")
                        .trim()
                        .to_string();
                    process.cmdline = s;
                }

                // Read stat file for CPU usage and start time
                if let Ok(stat) = fs::read_to_string(entry.path().join("stat")) {
                    let stats: Vec<&str> = stat.split_whitespace().collect();
                    if stats.len() > 21 {
                        process.start_time = stats[21].parse().unwrap_or(0.0);

                        // Calculate CPU usage from /proc/stat and process stats
                        if let Ok(cpu_stat) = fs::read_to_string("/proc/stat") {
                            if let Some(cpu_line) = cpu_stat.lines().find(|l| l.starts_with("cpu "))
                            {
                                let cpu_parts: Vec<&str> = cpu_line.split_whitespace().collect();
                                if cpu_parts.len() >= 8 {
                                    // Total CPU time = user + nice + system + idle + iowait + irq + softirq + steal
                                    let total_cpu_time: f64 = cpu_parts[1..8]
                                        .iter()
                                        .map(|s| s.parse::<f64>().unwrap_or(0.0))
                                        .sum();

                                    // Process CPU time = utime + stime (user + system time)
                                    if stats.len() >= 15 {
                                        let utime = stats[13].parse::<f64>().unwrap_or(0.0);
                                        let stime = stats[14].parse::<f64>().unwrap_or(0.0);
                                        let process_cpu_time = utime + stime;

                                        // CPU usage as percentage of a single core
                                        process.cpu_usage =
                                            (process_cpu_time / total_cpu_time) * 100.0;
                                    }
                                }
                            }
                        }
                    }
                }

                processes.push(process);
            }
        }

        Ok(processes)
    }

    #[cfg(not(target_os = "linux"))]
    fn get_process_info(_users_cache: &()) -> Result<Vec<ProcessInfo>, std::io::Error> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl DataSource for ProcessDataSource {
    type Config = ProcessesConfig;

    fn new(config: ProcessesConfig) -> Result<Self, LifelogError> {
        ProcessDataSource::new(config)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>> {
        Some(Arc::new(ProcessBufferedSource {
            stream_id: "processes".to_string(),
            buffer: self.buffer.clone(),
        }))
    }

    fn start(&self) -> Result<DataSourceHandle, LifelogError> {
        if RUNNING.load(Ordering::SeqCst) {
            return Err(LifelogError::AlreadyRunning);
        }

        tracing::info!("ProcessDataSource: Starting data source task");
        RUNNING.store(true, Ordering::SeqCst);

        let source_clone = self.clone();

        let _join_handle = tokio::spawn(async move {
            let task_result = source_clone.run().await;
            tracing::info!(result = ?task_result, "ProcessDataSource background task finished");
            task_result
        });

        let new_join_handle = tokio::spawn(async { Ok(()) });
        Ok(DataSourceHandle {
            join: new_join_handle,
        })
    }

    async fn stop(&mut self) -> Result<(), LifelogError> {
        RUNNING.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn run(&self) -> Result<(), LifelogError> {
        #[cfg(target_os = "linux")]
        let users_cache = UsersCache::new();
        #[cfg(not(target_os = "linux"))]
        let users_cache = ();

        while RUNNING.load(Ordering::SeqCst) {
            match Self::get_process_info(&users_cache) {
                Ok(processes) => {
                    if !processes.is_empty() {
                        let frame = ProcessFrame {
                            uuid: Uuid::new_v4().to_string(),
                            timestamp: to_pb_ts(Utc::now()),
                            processes,
                        };

                        let mut buf = Vec::new();
                        if let Err(e) = frame.encode(&mut buf) {
                            tracing::error!("Failed to encode ProcessFrame: {}", e);
                        } else if let Err(e) = self.buffer.append(&buf).await {
                            tracing::error!("Failed to append ProcessFrame to buffer: {}", e);
                        } else {
                            tracing::debug!("Stored process frame in WAL");
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to get process info: {}", e);
                }
            }
            sleep(Duration::from_secs_f64(self.config.interval)).await;
        }
        Ok(())
    }

    fn is_running(&self) -> bool {
        RUNNING.load(Ordering::SeqCst)
    }

    fn get_config(&self) -> Self::Config {
        self.config.clone()
    }
}

pub struct ProcessBufferedSource {
    stream_id: String,
    buffer: Arc<DiskBuffer>,
}

#[async_trait]
impl BufferedSource for ProcessBufferedSource {
    fn stream_id(&self) -> String {
        self.stream_id.clone()
    }

    async fn peek_upload_batch(
        &self,
        max_items: usize,
    ) -> Result<(u64, Vec<Vec<u8>>), LifelogError> {
        let (next_offset, raws) = self.buffer.peek_chunk(max_items).await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        Ok((next_offset, raws))
    }

    async fn commit_upload(&self, offset: u64) -> Result<(), LifelogError> {
        self.buffer.commit_offset(offset).await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        Ok(())
    }
}
