use crate::config::ProcessesConfig;
use crate::setup;
use chrono::Local;
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;
use std::process::Command;
use tokio::time::{sleep, Duration};
use users::{Users, UsersCache};

//impl DataLogger for ProcessLogger {
//
//
//}

// TODO: Make this logger work with windows (see how activity watch does this)
pub async fn start_logger(config: &ProcessesConfig) {
    let conn = setup::setup_process_db(Path::new(&config.output_dir))
        .expect("Failed to set up process database");

    let users_cache = UsersCache::new();

    loop {
        let timestamp = Local::now().timestamp() as f64
            + Local::now().timestamp_subsec_nanos() as f64 / 1_000_000_000.0;

        if let Ok(processes) = get_process_info(&users_cache) {
            for process in processes {
                conn.execute(
                    "INSERT INTO processes VALUES (
                        ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12
                    )",
                    params![
                        timestamp,
                        process.pid,
                        process.ppid,
                        process.name,
                        process.exe,
                        process.cmdline,
                        process.status,
                        process.cpu_usage,
                        process.memory_usage,
                        process.threads,
                        process.user,
                        process.start_time
                    ],
                )
                .unwrap();
            }
        }

        sleep(Duration::from_secs_f64(config.interval)).await;
    }
}

struct ProcessInfo {
    pid: i32,
    ppid: i32,
    name: String,
    exe: Option<String>,
    cmdline: Option<String>,
    status: String,
    cpu_usage: Option<f64>,
    memory_usage: Option<i64>,
    threads: i32,
    user: Option<String>,
    start_time: f64,
}

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
                exe: None,
                cmdline: None,
                status: String::new(),
                cpu_usage: None,
                memory_usage: None,
                threads: 0,
                user: None,
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
                                process.user = users_cache
                                    .get_user_by_uid(uid)
                                    .map(|u| u.name().to_string_lossy().into_owned());
                            }
                        }
                        Some("Threads:") => {
                            process.threads = parts.next().unwrap_or("0").parse().unwrap_or(0)
                        }
                        Some("VmRSS:") => {
                            process.memory_usage = parts.next().and_then(|s| s.parse().ok())
                        }
                        _ => {}
                    }
                }
            }

            // Read exe symlink
            process.exe = fs::read_link(entry.path().join("exe"))
                .ok()
                .and_then(|p| p.to_str().map(String::from));

            // Read cmdline
            if let Ok(cmdline) = fs::read(entry.path().join("cmdline")) {
                process.cmdline = String::from_utf8_lossy(&cmdline)
                    .replace('\0', " ")
                    .trim()
                    .to_string()
                    .into();
            }

            // Read stat file for CPU usage and start time
            if let Ok(stat) = fs::read_to_string(entry.path().join("stat")) {
                let stats: Vec<&str> = stat.split_whitespace().collect();
                if stats.len() > 21 {
                    process.start_time = stats[21].parse().unwrap_or(0.0);
                    // Calculate CPU usage (would need previous sample for proper calculation)
                }
            }

            processes.push(process);
        }
    }

    Ok(processes)
}
