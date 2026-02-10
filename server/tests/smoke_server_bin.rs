#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod bin_e2e;

use lifelog_types::lifelog_server_service_client::LifelogServerServiceClient;
use lifelog_types::{to_pb_ts, Ack, AudioFrame, Chunk, GetDataRequest, KeystrokeFrame, Query};
use prost::Message;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

use bin_e2e::{pick_unused_port, sha256_hex, wait_for_surreal_ws_ready, wait_for_tcp_listen};

#[tokio::test]
async fn server_binary_e2e_audio_and_keystrokes_roundtrip() {
    // Realistic end-to-end test:
    // - starts SurrealDB (memory) as an external process
    // - starts the actual server backend binary
    // - uploads Audio + Keystroke frames over gRPC (same API collectors use)
    // - queries by LLQL and fetches results via GetData

    let _ = tracing_subscriber::fmt::try_init();

    let db_port = pick_unused_port();
    let server_port = pick_unused_port();
    let db_addr = format!("127.0.0.1:{db_port}");
    let server_addr = format!("http://127.0.0.1:{server_port}");
    let server_tcp_addr = format!("127.0.0.1:{server_port}");

    // Start SurrealDB (in-memory, ephemeral).
    let mut db_child = bin_e2e::ChildGuard::new(
        Command::new("surreal")
            .arg("start")
            .arg("--user")
            .arg("root")
            .arg("--pass")
            .arg("root")
            .arg("--bind")
            .arg(&db_addr)
            .arg("memory")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn surrealdb"),
    );

    wait_for_surreal_ws_ready(&db_addr, Duration::from_secs(10))
        .await
        .expect("surrealdb should accept ws connections");

    // Start server backend binary.
    let cas_dir = tempfile::tempdir().expect("tempdir");
    let mut server_child = bin_e2e::ChildGuard::new(
        Command::new(env!("CARGO_BIN_EXE_lifelog-server-backend"))
            .env("LIFELOG_HOST", "127.0.0.1")
            .env("LIFELOG_PORT", server_port.to_string())
            .env("LIFELOG_DB_ENDPOINT", db_addr.clone())
            .env("LIFELOG_CAS_PATH", cas_dir.path())
            // Server requires these for SurrealDB auth.
            .env("LIFELOG_DB_USER", "root")
            .env("LIFELOG_DB_PASS", "root")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn server"),
    );

    // Wait for server to bind.
    if let Err(e) = wait_for_tcp_listen(
        server_child.child_mut(),
        &server_tcp_addr,
        Duration::from_secs(10),
    ) {
        let server_out = server_child
            .take_child()
            .expect("take server child")
            .wait_with_output()
            .expect("read server output after failure");
        let db_out = db_child
            .take_child()
            .expect("take surrealdb child")
            .wait_with_output()
            .expect("read surrealdb output after failure");
        panic!(
            "server should listen: {e}\nserver stdout:\n{}\nserver stderr:\n{}\ndb stdout:\n{}\ndb stderr:\n{}",
            String::from_utf8_lossy(&server_out.stdout),
            String::from_utf8_lossy(&server_out.stderr),
            String::from_utf8_lossy(&db_out.stdout),
            String::from_utf8_lossy(&db_out.stderr),
        );
    }

    // Connect gRPC client.
    let mut client = tokio::time::timeout(
        Duration::from_secs(10),
        LifelogServerServiceClient::connect(server_addr.clone()),
    )
    .await
    .expect("timeout connecting to server")
    .expect("connect to server");

    let collector_id = "e2e-collector";
    let session_id = 1u64;

    let base = chrono::Utc::now() - chrono::Duration::seconds(5);

    // Upload keystrokes.
    let keys_ts = to_pb_ts(base);
    let keys = KeystrokeFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: keys_ts,
        text: "KeyA".to_string(),
        application: "test-app".to_string(),
        window_title: "test-window".to_string(),
        t_device: keys_ts,
        t_canonical: keys_ts,
        t_end: keys_ts,
        record_type: lifelog_types::RecordType::Point as i32,
        ..Default::default()
    };
    let mut keys_buf = Vec::new();
    keys.encode(&mut keys_buf).expect("encode keystrokes");
    let keys_chunk = Chunk {
        stream: Some(lifelog_types::StreamIdentity {
            collector_id: collector_id.to_string(),
            stream_id: "keystrokes".to_string(),
            session_id,
        }),
        offset: 0,
        data: keys_buf.clone(),
        hash: sha256_hex(&keys_buf),
    };

    let ack: Ack = client
        .upload_chunks(tokio_stream::iter(vec![keys_chunk]))
        .await
        .expect("upload keystrokes")
        .into_inner();
    assert!(ack.acked_offset > 0, "keystrokes ack should advance");

    // Upload audio.
    let audio_ts = to_pb_ts(base + chrono::Duration::seconds(1));
    let audio_end = to_pb_ts(base + chrono::Duration::seconds(2));
    let audio = AudioFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: audio_ts,
        audio_bytes: vec![7u8; 128],
        codec: "wav".to_string(),
        sample_rate: 48_000,
        channels: 1,
        duration_secs: 1.0,
        t_device: audio_ts,
        t_canonical: audio_ts,
        t_end: audio_end,
        record_type: lifelog_types::RecordType::Interval as i32,
        ..Default::default()
    };
    let mut audio_buf = Vec::new();
    audio.encode(&mut audio_buf).expect("encode audio");
    let audio_chunk = Chunk {
        stream: Some(lifelog_types::StreamIdentity {
            collector_id: collector_id.to_string(),
            stream_id: "audio".to_string(),
            session_id,
        }),
        offset: 0,
        data: audio_buf.clone(),
        hash: sha256_hex(&audio_buf),
    };
    let ack: Ack = client
        .upload_chunks(tokio_stream::iter(vec![audio_chunk]))
        .await
        .expect("upload audio")
        .into_inner();
    assert!(ack.acked_offset > 0, "audio ack should advance");

    // Give SurrealDB a moment for SEARCH indexes (keystrokes table has BM25 on text).
    sleep(Duration::from_millis(250)).await;

    // Query keystrokes back via LLQL (exercises catalog resolution + query planner + executor).
    let start =
        (base - chrono::Duration::seconds(10)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let end =
        (base + chrono::Duration::seconds(10)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let llql = format!(
        r#"llql:{{
          "target": {{"type":"modality","modality":"Keystrokes"}},
          "filter": {{"op":"and","terms":[
            {{"op":"time_range","start":"{start}","end":"{end}"}},
            {{"op":"contains","field":"text","text":"KeyA"}}
          ]}}
        }}"#
    );
    let query = Query {
        text: vec![llql],
        ..Default::default()
    };

    let keys_resp = client
        .query(lifelog_types::QueryRequest { query: Some(query) })
        .await
        .expect("query keystrokes")
        .into_inner();
    assert_eq!(keys_resp.keys.len(), 1, "expected one keystroke match");

    let data = client
        .get_data(GetDataRequest {
            keys: keys_resp.keys.clone(),
        })
        .await
        .expect("get_data")
        .into_inner();
    assert_eq!(data.data.len(), 1);

    let lifelog_types::LifelogData { payload } = &data.data[0];
    let Some(lifelog_types::lifelog_data::Payload::Keystrokeframe(got)) = payload else {
        panic!("expected keystrokeframe payload");
    };
    assert_eq!(got.text, "KeyA");
    assert_eq!(got.application, "test-app");
    assert_eq!(got.window_title, "test-window");

    // Cleanup processes.
    drop(server_child);
    drop(db_child);
}
