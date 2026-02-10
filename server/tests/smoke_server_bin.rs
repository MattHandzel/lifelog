#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

// This test actually runs the server *binary*, but requires a SurrealDB instance.
// Keep it ignored by default so `just test` works in dev/CI without DB.
#[test]
#[ignore = "smoke test: requires SurrealDB running at LIFELOG_DB_ENDPOINT"]
fn server_binary_starts_and_listens() {
    let db = std::env::var("LIFELOG_DB_ENDPOINT").unwrap_or_else(|_| "127.0.0.1:7183".to_string());

    let port = portpicker::pick_unused_port().expect("pick port");
    let addr = format!("127.0.0.1:{port}");

    let mut child = Command::new(env!("CARGO_BIN_EXE_lifelog-server-backend"))
        .env("LIFELOG_HOST", "127.0.0.1")
        .env("LIFELOG_PORT", port.to_string())
        .env("LIFELOG_DB_ENDPOINT", db)
        .env("LIFELOG_CAS_PATH", tempfile::tempdir().unwrap().path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn server");

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut listening = false;
    while Instant::now() < deadline {
        if let Ok(Some(status)) = child.try_wait() {
            panic!("server exited early with status: {status}");
        }

        if TcpStream::connect(&addr).is_ok() {
            listening = true;
            break;
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    if !listening {
        child.kill().ok();
        let _ = child.wait();
        panic!("server did not start listening on {addr} within deadline");
    }

    child.kill().expect("kill server");
    let _ = child.wait();
}
