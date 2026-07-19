use config::CollectorConfig;
use std::fs;
use std::path::Path;
use sysinfo::System;

pub struct PidLock {
    path: std::path::PathBuf,
}

impl PidLock {
    pub fn acquire(dir: &Path) -> Result<Self, String> {
        let path = dir.join("lifelog-collector.pid");
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(pid) = contents.trim().parse::<u32>() {
                let check = Path::new("/proc").join(pid.to_string());
                if check.exists() {
                    return Err(format!(
                        "Another collector instance is running (pid {})",
                        pid
                    ));
                }
            }
        }

        let pid = std::process::id();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&path, pid.to_string())
            .map_err(|e| format!("Failed to write PID lock: {}", e))?;
        Ok(Self { path })
    }
}

impl Drop for PidLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

// Ensures the directory exists, creating it if necessary.
pub fn ensure_directory(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn initialize_project(config: &CollectorConfig) -> std::io::Result<()> {
    // TODO: Check to see if all of these things exist or not
    // TODO: These should be moved inside their respective loggers

    if let Some(ref s) = config.screen {
        ensure_directory(Path::new(&s.output_dir))?;
    }
    if let Some(ref c) = config.camera {
        ensure_directory(Path::new(&c.output_dir))?;
    }
    if let Some(ref m) = config.microphone {
        ensure_directory(Path::new(&m.output_dir))?;
    }
    Ok(())
}

pub fn n_processes_already_running(
    process_name: &str,
    num_processes_that_should_be_running: i32,
) -> bool {
    let system = System::new_all();
    let mut process_count = 0;
    for process in system.processes_by_name(std::ffi::OsStr::new(process_name)) {
        if process.name() == process_name {
            process_count += 1;
            if process_count >= num_processes_that_should_be_running {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_ensure_directory() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("subdir");
        assert!(!path.exists());
        ensure_directory(&path).unwrap();
        assert!(path.exists());
        assert!(path.is_dir());
    }
}
