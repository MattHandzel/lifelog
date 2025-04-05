use crate::setup;
use chrono::Local;
use config::ProcessesConfig;
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

pub struct ProcessInfo {
    pub pid: i32,
    pub ppid: i32,
    pub name: String,
    pub exe: Option<String>,
    pub cmdline: Option<String>,
    pub status: String,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<i64>,
    pub threads: i32,
    pub user: Option<String>,
    pub start_time: f64,
}

// Public function to get current processes for the frontend
pub fn get_current_processes(users_cache: &UsersCache) -> Result<Vec<ProcessInfo>, std::io::Error> {
    get_process_info(users_cache)
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

                    // Calculate CPU usage from /proc/stat and process stats
                    if let Ok(cpu_stat) = fs::read_to_string("/proc/stat") {
                        if let Some(cpu_line) = cpu_stat.lines().find(|l| l.starts_with("cpu ")) {
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
                                        Some((process_cpu_time / total_cpu_time) * 100.0);
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
