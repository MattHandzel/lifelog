#![allow(clippy::expect_used)]

use std::net::TcpStream;
use std::process::Child;
use std::time::{Duration, Instant};

pub fn sha256_hex(bytes: &[u8]) -> String {
    utils::cas::sha256_hex(bytes)
}

pub fn pick_unused_port() -> u16 {
    portpicker::pick_unused_port().expect("pick unused port")
}

pub fn write_test_tls_materials(dir: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let cert_path = dir.join("test-cert.pem");
    let key_path = dir.join("test-key.pem");
    let status = std::process::Command::new("openssl")
        .args([
            "req",
            "-x509",
            "-newkey",
            "rsa:2048",
            "-sha256",
            "-days",
            "3650",
            "-nodes",
            "-keyout",
            key_path.to_str().expect("key path utf8"),
            "-out",
            cert_path.to_str().expect("cert path utf8"),
            "-subj",
            "/CN=localhost",
            "-addext",
            "subjectAltName=DNS:localhost,IP:127.0.0.1",
            "-addext",
            "basicConstraints=critical,CA:FALSE",
            "-addext",
            "keyUsage=critical,digitalSignature,keyEncipherment",
            "-addext",
            "extendedKeyUsage=serverAuth",
        ])
        .status()
        .expect("openssl must be available for integration tests");
    assert!(status.success(), "failed to generate test TLS materials");
    (cert_path, key_path)
}

pub fn wait_for_tcp_listen(child: &mut Child, addr: &str, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("process exited early with status: {status}"));
        }

        if TcpStream::connect(addr).is_ok() {
            return Ok(());
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    Err(format!("timed out waiting for TCP listen at {addr}"))
}

pub struct ChildGuard {
    child: Option<Child>,
}

impl ChildGuard {
    pub fn new(child: Child) -> Self {
        Self { child: Some(child) }
    }

    pub fn child_mut(&mut self) -> &mut Child {
        self.child.as_mut().expect("child already taken")
    }

    pub fn take_child(&mut self) -> Option<Child> {
        self.child.take()
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub async fn wait_for_surreal_ws_ready(addr: &str, timeout: Duration) -> Result<(), String> {
    use surrealdb::engine::remote::ws::Ws;
    use surrealdb::opt::auth::Root;
    use surrealdb::Surreal;

    let deadline = std::time::Instant::now() + timeout;
    loop {
        if let Ok(db) = Surreal::new::<Ws>(addr).await {
            let signed = db
                .signin(Root {
                    username: "root",
                    password: "root",
                })
                .await;
            if signed.is_ok() {
                return Ok(());
            }
        }

        if std::time::Instant::now() >= deadline {
            return Err(format!("timed out waiting for surreal ws ready at {addr}"));
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
