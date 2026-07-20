//! End-to-end regression for MAT-1522: SetConfig must persist so a subsequent
//! GetConfig returns the new value, and a collector reconnect must not revert
//! an operator-set config.
//!
//! Drives the real `ServerHandle::{register_collector, apply_system_config,
//! get_config}` against a throwaway Postgres database (created on the local
//! socket, dropped at the end). No gRPC/TLS layer — the bug lived entirely in
//! the in-memory collector registry, so exercising the handle is sufficient and
//! deterministic.

use std::collections::HashMap;
use std::sync::Arc;

use config::ServerConfig;
use lifelog_server::server::{RegisteredCollector, Server, ServerHandle};
use lifelog_types::{CollectorConfig, ScreenConfig, SystemConfig};
use tokio::sync::RwLock;

fn cfg_with_interval(id: &str, interval: f64) -> CollectorConfig {
    CollectorConfig {
        id: id.to_string(),
        screen: Some(ScreenConfig {
            interval,
            ..Default::default()
        }),
        ..Default::default()
    }
}

type CmdRx = tokio::sync::mpsc::Receiver<Result<lifelog_types::ServerCommand, tonic::Status>>;

fn collector(id: &str, interval: f64) -> (RegisteredCollector, CmdRx) {
    let (tx, rx) = tokio::sync::mpsc::channel(8);
    (
        RegisteredCollector {
            id: id.to_string(),
            address: "test".to_string(),
            mac: id.to_string(),
            command_tx: tx,
            latest_config: Some(cfg_with_interval(id, interval)),
        },
        rx,
    )
}

fn screen_interval(cfg: &SystemConfig, id: &str) -> f64 {
    cfg.collectors[id].screen.as_ref().unwrap().interval
}

/// Create a uniquely named test database on the local Postgres socket and point
/// the server at it. Returns the admin connection URL and the test db name so
/// the caller can drop it afterward. Skips (returns None) when no local socket
/// Postgres is reachable.
async fn provision_db() -> Option<(String, String)> {
    let admin = "host=/run/postgresql dbname=postgres";
    let (client, conn) = match tokio_postgres::connect(admin, tokio_postgres::NoTls).await {
        Ok(v) => v,
        Err(_) => return None,
    };
    tokio::spawn(async move {
        let _ = conn.await;
    });
    let db_name = format!("lifelog_mat1522_{}", std::process::id());
    let _ = client
        .execute(&format!("DROP DATABASE IF EXISTS \"{db_name}\""), &[])
        .await;
    client
        .execute(&format!("CREATE DATABASE \"{db_name}\""), &[])
        .await
        .expect("create test db");
    std::env::set_var(
        "LIFELOG_POSTGRES_INGEST_URL",
        format!("host=/run/postgresql dbname={db_name}"),
    );
    Some((admin.to_string(), db_name))
}

async fn drop_db(admin: &str, db_name: &str) {
    if let Ok((client, conn)) = tokio_postgres::connect(admin, tokio_postgres::NoTls).await {
        tokio::spawn(async move {
            let _ = conn.await;
        });
        let _ = client
            .execute(&format!("DROP DATABASE IF EXISTS \"{db_name}\""), &[])
            .await;
    }
}

#[tokio::test]
async fn set_config_persists_and_survives_reconnect() {
    let Some((admin, db_name)) = provision_db().await else {
        eprintln!("skipping: no local Postgres socket");
        return;
    };

    let tmp = std::env::temp_dir().join(format!("mat1522-cas-{}", std::process::id()));
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        database_endpoint: format!("host=/run/postgresql dbname={db_name}"),
        database_name: db_name.clone(),
        server_name: "TestServer".to_string(),
        cas_path: tmp.display().to_string(),
        default_correlation_window_ms: 30_000,
        retention_policy_days: HashMap::new(),
        transforms: Vec::new(),
    };

    let server = Server::new(&config).await.expect("build server");
    let handle = ServerHandle::new(Arc::new(RwLock::new(server)));

    // Collector connects reporting screen interval 10.
    let (c, _rx) = collector("cam", 10.0);
    handle.register_collector(c).await;
    assert_eq!(screen_interval(&handle.get_config().await, "cam"), 10.0);

    // Operator SetConfig -> 15.
    let mut collectors = HashMap::new();
    collectors.insert("cam".to_string(), cfg_with_interval("cam", 15.0));
    handle
        .apply_system_config(SystemConfig {
            server: None,
            collectors,
        })
        .await
        .expect("apply config");

    // GetConfig must now return the new value — the original bug returned 10.
    assert_eq!(
        screen_interval(&handle.get_config().await, "cam"),
        15.0,
        "SetConfig did not persist into GetConfig"
    );

    // Collector reconnects, self-reporting the STALE interval 10. The operator
    // override is authoritative, so GetConfig must still be 15 and the registry
    // must not have accumulated a duplicate.
    let (c2, mut rx2) = collector("cam", 10.0);
    handle.register_collector(c2).await;
    assert_eq!(
        screen_interval(&handle.get_config().await, "cam"),
        15.0,
        "reconnect clobbered the operator-set config"
    );
    // The reconnecting collector must receive an UpdateConfig push so it applies
    // the operator config rather than its own stale one.
    let pushed = rx2
        .try_recv()
        .expect("operator config re-pushed on reconnect");
    let cmd = pushed.expect("push is a command");
    assert!(
        cmd.payload.contains("15"),
        "pushed config carries interval 15"
    );

    drop_db(&admin, &db_name).await;
}
