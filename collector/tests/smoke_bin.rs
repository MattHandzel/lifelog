#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::process::{Command, Stdio};
use std::time::Duration;

#[test]
fn collector_binary_starts_with_all_modules_disabled() {
    // This is a real process-level smoke test to ensure the collector binary can start.
    // We disable all modules to avoid relying on platform-specific permissions/devices.

    let tmp = tempfile::tempdir().expect("tempdir");

    let home = tmp.path();
    let cfg_dir = home.join(".config/lifelog");
    fs::create_dir_all(&cfg_dir).expect("create config dir");

    let cfg_path = cfg_dir.join("config.toml");
    fs::write(
        &cfg_path,
        r#"
# Minimal smoke config.
id = "smoke-test"
host = "127.0.0.1"
port = 7190
timestamp_format = "%Y-%m-%d_%H-%M-%S"

[screen]
enabled = false
interval = 20.0
output_dir = "./out/screen"
program = "gnome-screenshot"
timestamp_format = "%Y-%m-%d_%H-%M-%S"

[browser]
enabled = false
input_file = ""
output_file = "./out/browser"
browser_type = "chrome"

[camera]
enabled = false
interval = 10.0
output_dir = "./out/camera"
device = "/dev/video0"
resolution_x = 640
resolution_y = 480
fps = 30
timestamp_format = "%Y-%m-%d_%H-%M-%S"

[microphone]
enabled = false
output_dir = "./out/microphone"
sample_rate = 44100
chunk_duration_secs = 1
capture_interval_secs = 1
timestamp_format = "%Y-%m-%d_%H-%M-%S"
bits_per_sample = 16
channels = 1

[processes]
enabled = false
interval = 60.0
output_dir = "./out/processes"

[hyprland]
enabled = false
interval = 1.0
output_dir = "./out/hyprland"
log_clients = true
log_activewindow = true
log_workspace = true
log_active_monitor = true
log_devices = true

[weather]
enabled = false
interval = 1800.0
output_dir = "./out/weather"
api_key = ""
latitude = 0.0
longitude = 0.0

[wifi]
enabled = false
interval = 300.0
output_dir = "./out/wifi"
scan_command = ""

[clipboard]
enabled = false
interval = 2.0
output_dir = "./out/clipboard"
max_text_bytes = 262144

[shell_history]
enabled = false
interval = 2.0
output_dir = "./out/shell_history"
history_file = "./.zsh_history"
shell_type = "auto"

[mouse]
enabled = false
interval = 0.25
output_dir = "./out/mouse"

[window_activity]
enabled = false
interval = 1.0
output_dir = "./out/window_activity"
backend = "auto"

[keyboard]
enabled = false
interval = 1.0
output_dir = "./out/keystrokes"
"#,
    )
    .expect("write config");

    // Point at a closed port so we don't depend on a running server.
    let mut child = Command::new(env!("CARGO_BIN_EXE_lifelog-collector"))
        .arg("--server-address")
        .arg("http://127.0.0.1:1")
        .env("HOME", home)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn collector");

    std::thread::sleep(Duration::from_millis(750));

    // The process should still be alive (not crash-looped). If it already exited, fail.
    if let Ok(Some(status)) = child.try_wait() {
        let out = child
            .wait_with_output()
            .expect("read collector output after early exit");
        panic!(
            "collector exited early with status: {status}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    child.kill().expect("kill collector");
    let _ = child.wait();
}
