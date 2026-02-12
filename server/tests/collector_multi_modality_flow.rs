#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod harness;

use harness::TestContext;
use lifelog_collector::collector::upload_manager::UploadManager;
use lifelog_collector::collector::Collector;
use lifelog_types::{GetDataRequest, Query, QueryRequest};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_collector_with_multiple_modalities_uploads_to_server() {
    let _ = tracing_subscriber::fmt::try_init();

    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    let tmp = tempfile::tempdir().expect("tempdir");
    let history_file = tmp.path().join("test_zsh_history");
    let now = chrono::Utc::now().timestamp();
    std::fs::write(
        &history_file,
        format!(": {now}:0;echo collector-multi-modal\n"),
    )
    .expect("write history");

    let mut cfg = config::create_default_config();
    cfg.id = "collector-multi-modal".to_string();
    cfg.host = "127.0.0.1".to_string();
    cfg.port = 7182;

    // Disable modalities that depend on hardware/desktop permissions in CI.
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

    // Keep multiple reliable modalities enabled.
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
    let collector = Arc::new(RwLock::new(Collector::new(
        Arc::new(cfg.clone()),
        ctx.server_addr.clone(),
        cfg.id.clone(),
        upload_trigger.clone(),
    )));

    {
        let mut c = collector.write().await;
        c.start().await.expect("collector start");
    }

    let collector_for_uploader = Arc::clone(&collector);
    let upload_task = tokio::spawn(async move {
        upload_mgr.run(collector_for_uploader).await;
    });

    // Let modules emit frames, then force upload cycle.
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    upload_trigger.send(()).await.expect("trigger upload");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

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

        assert!(
            !keys.is_empty(),
            "expected at least one {} key from collector upload",
            modality.as_str_name()
        );

        let data = client
            .get_data(GetDataRequest { keys })
            .await
            .expect("get_data should succeed")
            .into_inner();

        assert!(
            !data.data.is_empty(),
            "expected {} payload data",
            modality.as_str_name()
        );
    }

    {
        let mut c = collector.write().await;
        c.stop().await;
    }
    upload_task.abort();
}
