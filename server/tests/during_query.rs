#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod harness;

use chrono::{Duration, Utc};
use harness::TestContext;
use lifelog_core::{DataOrigin, DataOriginType};
use lifelog_server::query::{ast, executor, planner};
use lifelog_types::{AudioFrame, ScreenFrame};
use prost::Message;

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn test_during_returns_target_records_inside_source_intervals() {
    let _ = tracing_subscriber::fmt::try_init();

    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    let collector_id = "test-collector";
    let session_id = 1u64;

    let base = Utc::now() - Duration::minutes(5);

    // Ingest two ScreenFrames: one during the matching audio interval, one outside.
    let screen_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "screen".to_string(),
        session_id,
    };

    let in_uuid = lifelog_core::Uuid::new_v4().to_string();
    let in_screen = ScreenFrame {
        uuid: in_uuid.clone(),
        timestamp: Some(lifelog_types::to_pb_ts(base + Duration::seconds(5)).unwrap()),
        width: 100,
        height: 100,
        image_bytes: vec![0; 10],
        mime_type: "image/jpeg".to_string(),
    };
    let mut in_buf = Vec::new();
    in_screen.encode(&mut in_buf).unwrap();

    let out_uuid = lifelog_core::Uuid::new_v4().to_string();
    let out_screen = ScreenFrame {
        uuid: out_uuid.clone(),
        timestamp: Some(lifelog_types::to_pb_ts(base + Duration::seconds(30)).unwrap()),
        width: 100,
        height: 100,
        image_bytes: vec![0; 10],
        mime_type: "image/jpeg".to_string(),
    };
    let mut out_buf = Vec::new();
    out_screen.encode(&mut out_buf).unwrap();

    let in_chunk = lifelog_types::Chunk {
        stream: Some(screen_stream.clone()),
        offset: 0,
        data: in_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let in_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&in_chunk.data),
        ..in_chunk
    };

    let out_offset = in_chunk.data.len() as u64;
    let out_chunk = lifelog_types::Chunk {
        stream: Some(screen_stream),
        offset: out_offset,
        data: out_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let out_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&out_chunk.data),
        ..out_chunk
    };

    client
        .upload_chunks(tokio_stream::iter(vec![in_chunk, out_chunk]))
        .await
        .expect("Ingest screen failed");

    // Ingest two AudioFrames: one matching (codec=pcm, duration=10s), one non-matching.
    let audio_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "audio".to_string(),
        session_id,
    };

    let pcm = AudioFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: Some(lifelog_types::to_pb_ts(base).unwrap()),
        audio_bytes: vec![1; 10],
        codec: "pcm".to_string(),
        sample_rate: 48000,
        channels: 1,
        duration_secs: 10.0,
    };
    let mut pcm_buf = Vec::new();
    pcm.encode(&mut pcm_buf).unwrap();

    let aac = AudioFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: Some(lifelog_types::to_pb_ts(base + Duration::seconds(120)).unwrap()),
        audio_bytes: vec![2; 10],
        codec: "aac".to_string(),
        sample_rate: 48000,
        channels: 1,
        duration_secs: 10.0,
    };
    let mut aac_buf = Vec::new();
    aac.encode(&mut aac_buf).unwrap();

    let pcm_chunk = lifelog_types::Chunk {
        stream: Some(audio_stream.clone()),
        offset: 0,
        data: pcm_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let pcm_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&pcm_chunk.data),
        ..pcm_chunk
    };

    let aac_offset = pcm_chunk.data.len() as u64;
    let aac_chunk = lifelog_types::Chunk {
        stream: Some(audio_stream),
        offset: aac_offset,
        data: aac_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let aac_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&aac_chunk.data),
        ..aac_chunk
    };

    client
        .upload_chunks(tokio_stream::iter(vec![pcm_chunk, aac_chunk]))
        .await
        .expect("Ingest audio failed");

    tokio::time::sleep(std::time::Duration::from_millis(250)).await;

    let db = surrealdb::Surreal::new::<surrealdb::engine::remote::ws::Ws>(&ctx.db_addr)
        .await
        .expect("DB Connect failed");
    db.signin(surrealdb::opt::auth::Root {
        username: "root",
        password: "root",
    })
    .await
    .expect("DB Signin failed");
    db.use_ns("lifelog")
        .use_db("test_db")
        .await
        .expect("DB Select failed");

    let screen_origin = DataOrigin::new(
        DataOriginType::DeviceId(collector_id.to_string()),
        "Screen".to_string(),
    );
    let audio_origin = DataOrigin::new(
        DataOriginType::DeviceId(collector_id.to_string()),
        "Audio".to_string(),
    );

    let query = ast::Query {
        target: ast::StreamSelector::StreamId(screen_origin.get_table_name()),
        filter: ast::Expression::During {
            stream: ast::StreamSelector::StreamId(audio_origin.get_table_name()),
            predicate: Box::new(ast::Expression::Eq(
                "codec".to_string(),
                ast::Value::String("pcm".to_string()),
            )),
        },
    };

    let plan = planner::Planner::plan(&query, &[screen_origin.clone(), audio_origin.clone()]);
    let keys = executor::execute(&db, plan)
        .await
        .expect("DURING query execution failed");

    let keys_str: Vec<String> = keys.iter().map(|k| k.uuid.to_string()).collect();

    assert!(
        keys_str.contains(&in_uuid),
        "Expected screen record during pcm audio interval to match"
    );
    assert!(
        !keys_str.contains(&out_uuid),
        "Did not expect screen record outside pcm audio interval to match"
    );
}
