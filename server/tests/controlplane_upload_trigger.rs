#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod harness;

use harness::TestContext;
use lifelog_collector::collector::upload_manager::UploadManager;
use lifelog_collector::collector::{Collector, CollectorHandle};
use lifelog_types::{GetDataRequest, Query, QueryRequest};
use std::sync::Arc;

#[tokio::test]
async fn test_controlplane_begin_upload_session_triggers_collector_upload() {
    let _ = tracing_subscriber::fmt::try_init();

    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    let tmp = tempfile::tempdir().expect("tempdir");
    let history_file = tmp.path().join("test_zsh_history_controlplane");
    let now = chrono::Utc::now().timestamp();
    std::fs::write(
        &history_file,
        format!(": {now}:0;echo controlplane-triggered-upload\n"),
    )
    .expect("write history");

    let mut cfg = config::create_default_config();
    cfg.id = "collector-controlplane".to_string();
    cfg.host = "127.0.0.1".to_string();
    cfg.port = 7183;

    // Keep only reliable CI-safe modalities.
    cfg.screen.as_mut().expect("screen cfg").enabled = false;
    cfg.browser.as_mut().expect("browser cfg").enabled = false;
    cfg.camera.as_mut().expect("camera cfg").enabled = false;
    cfg.microphone.as_mut().expect("microphone cfg").enabled = false;
    cfg.hyprland.as_mut().expect("hyprland cfg").enabled = false;
    cfg.weather.as_mut().expect("weather cfg").enabled = false;
    cfg.clipboard.as_mut().expect("clipboard cfg").enabled = false;
    cfg.mouse.as_mut().expect("mouse cfg").enabled = false;
    cfg.window_activity
        .as_mut()
        .expect("window_activity cfg")
        .enabled = false;
    cfg.keyboard.as_mut().expect("keyboard cfg").enabled = false;

    cfg.processes.as_mut().expect("processes cfg").enabled = true;
    cfg.processes.as_mut().expect("processes cfg").interval = 0.25;
    cfg.processes.as_mut().expect("processes cfg").output_dir =
        tmp.path().join("processes").display().to_string();

    cfg.shell_history
        .as_mut()
        .expect("shell_history cfg")
        .enabled = true;
    cfg.shell_history
        .as_mut()
        .expect("shell_history cfg")
        .interval = 0.25;
    cfg.shell_history
        .as_mut()
        .expect("shell_history cfg")
        .output_dir = tmp.path().join("shell_history").display().to_string();
    cfg.shell_history
        .as_mut()
        .expect("shell_history cfg")
        .history_file = history_file.display().to_string();
    cfg.shell_history
        .as_mut()
        .expect("shell_history cfg")
        .shell_type = "zsh".to_string();

    let (upload_mgr, upload_trigger) = UploadManager::new(ctx.server_addr.clone(), cfg.id.clone());
    let collector_handle = CollectorHandle::new(Collector::new(
        Arc::new(cfg.clone()),
        ctx.server_addr.clone(),
        cfg.id.clone(),
        upload_trigger,
    ));

    {
        let mut c = collector_handle.collector.write().await;
        c.start().await.expect("collector start");
    }

    // Explicitly establish ControlPlane stream so server SyncData can deliver BeginUploadSession.
    {
        let mut c = collector_handle.collector.write().await;
        c.handshake(collector_handle.clone())
            .await
            .expect("collector handshake");
    }

    let collector_for_uploader = collector_handle.collector.clone();
    let upload_task = tokio::spawn(async move {
        upload_mgr.run(collector_for_uploader).await;
    });

    // No manual upload trigger here: data must arrive via server -> BeginUploadSession command.
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(25);
    let mut saw_processes = false;
    let mut saw_shell_history = false;

    while tokio::time::Instant::now() < deadline && !(saw_processes && saw_shell_history) {
        let start = (chrono::Utc::now() - chrono::Duration::minutes(10))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let end = (chrono::Utc::now() + chrono::Duration::minutes(10))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

        for modality in [
            lifelog_types::DataModality::Processes,
            lifelog_types::DataModality::ShellHistory,
        ] {
            let llql = format!(
                r#"llql:{{
              "target": {{"type":"modality","modality":"{}"}},
              "filter": {{"op":"time_range","start":"{}","end":"{}"}}
            }}"#,
                modality.as_str_name(),
                start,
                end
            );

            let keys = client
                .query(QueryRequest {
                    query: Some(Query {
                        text: vec![llql],
                        ..Default::default()
                    }),
                })
                .await
                .expect("query should succeed")
                .into_inner()
                .keys;

            if keys.is_empty() {
                continue;
            }

            let data = client
                .get_data(GetDataRequest { keys })
                .await
                .expect("get_data should succeed")
                .into_inner();

            if !data.data.is_empty() {
                if modality == lifelog_types::DataModality::Processes {
                    saw_processes = true;
                }
                if modality == lifelog_types::DataModality::ShellHistory {
                    saw_shell_history = true;
                }
            }
        }

        if !(saw_processes && saw_shell_history) {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }

    assert!(
        saw_processes,
        "expected Processes data from controlplane-triggered upload"
    );
    assert!(
        saw_shell_history,
        "expected ShellHistory data from controlplane-triggered upload"
    );

    {
        let mut c = collector_handle.collector.write().await;
        c.stop().await;
    }
    upload_task.abort();
}
