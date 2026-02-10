use crate::data_source::{BufferedSource, DataSource, DataSourceHandle};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use config::ShellHistoryConfig;
use lifelog_core::{LifelogError, Uuid};
use lifelog_types::{to_pb_ts, ShellHistoryFrame};
use prost::Message;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use utils::buffer::DiskBuffer;

static RUNNING: AtomicBool = AtomicBool::new(false);

type ShellEvent = (chrono::DateTime<Utc>, String);

#[derive(Debug, Clone)]
pub struct ShellHistoryDataSource {
    config: ShellHistoryConfig,
    pub buffer: Arc<DiskBuffer>,
}

impl ShellHistoryDataSource {
    pub fn new(config: ShellHistoryConfig) -> Result<Self, LifelogError> {
        let buffer_path = std::path::Path::new(&config.output_dir).join("buffer");
        let buffer = DiskBuffer::new(&buffer_path).map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        Ok(Self {
            config,
            buffer: Arc::new(buffer),
        })
    }

    fn cursor_path(&self) -> std::path::PathBuf {
        std::path::Path::new(&self.config.output_dir).join("history_cursor")
    }

    fn load_cursor(&self) -> u64 {
        std::fs::read_to_string(self.cursor_path())
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0)
    }

    fn save_cursor(&self, cursor: u64) {
        let _ = std::fs::create_dir_all(&self.config.output_dir);
        let _ = std::fs::write(self.cursor_path(), cursor.to_string());
    }

    fn parse_zsh_extended(line: &str) -> Option<(chrono::DateTime<Utc>, String)> {
        // Format: ": 1700000000:0;command"
        let line = line.trim_end_matches(&['\n', '\r'][..]);
        let rest = line.strip_prefix(": ")?;
        let (epoch_part, rest) = rest.split_once(':')?;
        let epoch: i64 = epoch_part.parse().ok()?;
        let (_duration_part, cmd_part) = rest.split_once(';')?;
        let cmd = cmd_part.trim();
        if cmd.is_empty() {
            return None;
        }
        Some((Utc.timestamp_opt(epoch, 0).single()?, cmd.to_string()))
    }

    fn parse_bash_histtime(lines: &[String]) -> Vec<ShellEvent> {
        // Format:
        //   # 1700000000
        //   command
        let mut out = Vec::new();
        let mut pending_ts: Option<chrono::DateTime<Utc>> = None;
        for l in lines {
            let line = l.trim_end_matches(&['\n', '\r'][..]);
            if let Some(rest) = line.strip_prefix('#') {
                let ts = rest.trim().parse::<i64>().ok();
                pending_ts = ts.and_then(|t| Utc.timestamp_opt(t, 0).single());
                continue;
            }
            let cmd = line.trim();
            if cmd.is_empty() {
                continue;
            }
            let ts = pending_ts.take().unwrap_or_else(Utc::now);
            out.push((ts, cmd.to_string()));
        }
        out
    }

    fn detect_shell_type(&self) -> ShellType {
        let t = self.config.shell_type.trim().to_lowercase();
        match t.as_str() {
            "zsh" => ShellType::Zsh,
            "bash" => ShellType::Bash,
            _ => ShellType::Auto,
        }
    }

    fn read_new_history(&self, from_offset: u64) -> Result<(u64, Vec<ShellEvent>), LifelogError> {
        let path = self.config.history_file.clone();
        if path.is_empty() {
            return Ok((from_offset, Vec::new()));
        }

        let meta = std::fs::metadata(&path).map_err(LifelogError::Io)?;
        let file_len = meta.len();
        let mut cursor = from_offset.min(file_len);
        if file_len < from_offset {
            // File rotated/truncated.
            cursor = 0;
        }

        let mut f = File::open(&path).map_err(LifelogError::Io)?;
        f.seek(SeekFrom::Start(cursor)).map_err(LifelogError::Io)?;
        let mut reader = BufReader::new(f);

        let mut lines = Vec::new();
        let mut buf = String::new();
        while reader.read_line(&mut buf).map_err(LifelogError::Io)? > 0 {
            lines.push(buf.clone());
            buf.clear();
        }

        // Advance cursor to the end of file.
        let new_cursor = file_len;

        if lines.is_empty() {
            return Ok((new_cursor, Vec::new()));
        }

        let shell_type = self.detect_shell_type();
        let mut events = Vec::new();

        // Auto-detect: if any line parses as zsh extended, treat all as zsh extended.
        let looks_like_zsh = matches!(shell_type, ShellType::Zsh)
            || (matches!(shell_type, ShellType::Auto)
                && lines.iter().any(|l| Self::parse_zsh_extended(l).is_some()));

        if looks_like_zsh {
            for l in &lines {
                if let Some((ts, cmd)) = Self::parse_zsh_extended(l) {
                    events.push((ts, cmd));
                }
            }
            return Ok((new_cursor, events));
        }

        // bash (HISTTIMEFORMAT) support; otherwise treat each line as a command with "now" ts.
        if matches!(shell_type, ShellType::Bash | ShellType::Auto) {
            let parsed = Self::parse_bash_histtime(&lines);
            if !parsed.is_empty() {
                return Ok((new_cursor, parsed));
            }
        }

        // Fallback: each non-empty line is a command (timestamp = now).
        for l in &lines {
            let cmd = l.trim();
            if cmd.is_empty() {
                continue;
            }
            events.push((Utc::now(), cmd.to_string()));
        }

        Ok((new_cursor, events))
    }
}

#[derive(Debug, Clone, Copy)]
enum ShellType {
    Auto,
    Zsh,
    Bash,
}

#[async_trait]
impl DataSource for ShellHistoryDataSource {
    type Config = ShellHistoryConfig;

    fn new(config: ShellHistoryConfig) -> Result<Self, LifelogError> {
        Self::new(config)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>> {
        Some(Arc::new(ShellHistoryBufferedSource {
            stream_id: "shell_history".to_string(),
            buffer: self.buffer.clone(),
        }))
    }

    fn start(&self) -> Result<DataSourceHandle, LifelogError> {
        if RUNNING.load(Ordering::SeqCst) {
            tracing::warn!("ShellHistoryDataSource: Start called but task is already running.");
            return Err(LifelogError::AlreadyRunning);
        }

        RUNNING.store(true, Ordering::SeqCst);
        let source_clone = self.clone();
        let _join_handle = tokio::spawn(async move { source_clone.run().await });

        Ok(DataSourceHandle {
            join: tokio::spawn(async { Ok(()) }),
        })
    }

    async fn stop(&mut self) -> Result<(), LifelogError> {
        RUNNING.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn run(&self) -> Result<(), LifelogError> {
        let mut cursor = self.load_cursor();

        while RUNNING.load(Ordering::SeqCst) {
            match self.read_new_history(cursor) {
                Ok((new_cursor, events)) => {
                    cursor = new_cursor;
                    if !events.is_empty() {
                        self.save_cursor(cursor);
                    }
                    for (ts_dt, cmd) in events {
                        let timestamp = to_pb_ts(ts_dt);
                        let frame = ShellHistoryFrame {
                            uuid: Uuid::new_v4().to_string(),
                            timestamp,
                            command: cmd,
                            working_dir: "".to_string(),
                            exit_code: -1,
                            t_device: timestamp,
                            t_canonical: timestamp,
                            t_end: timestamp,
                            ..Default::default()
                        };

                        let mut buf = Vec::new();
                        if let Err(e) = frame.encode(&mut buf) {
                            tracing::error!("Failed to encode ShellHistoryFrame: {}", e);
                        } else if let Err(e) = self.buffer.append(&buf).await {
                            tracing::error!("Failed to append ShellHistoryFrame to buffer: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!(error = %e, "Shell history read failed");
                }
            }

            let interval = if self.config.interval > 0.0 {
                self.config.interval
            } else {
                2.0
            };
            sleep(Duration::from_secs_f64(interval)).await;
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

pub struct ShellHistoryBufferedSource {
    stream_id: String,
    buffer: Arc<DiskBuffer>,
}

#[async_trait]
impl BufferedSource for ShellHistoryBufferedSource {
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_zsh_extended_line() {
        let (ts, cmd) =
            ShellHistoryDataSource::parse_zsh_extended(": 1700000000:0;ls -la").expect("parse");
        assert_eq!(cmd, "ls -la");
        assert_eq!(ts.timestamp(), 1700000000);
    }

    #[test]
    fn parse_bash_histtime_pairs() {
        let lines = vec![
            "# 1700000001\n".to_string(),
            "echo hi\n".to_string(),
            "pwd\n".to_string(), // should get now fallback since no timestamp line
        ];
        let out = ShellHistoryDataSource::parse_bash_histtime(&lines);
        assert_eq!(out[0].0.timestamp(), 1700000001);
        assert_eq!(out[0].1, "echo hi");
        assert_eq!(out[1].1, "pwd");
    }
}
