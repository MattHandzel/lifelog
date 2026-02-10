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
    let in_ts = lifelog_types::to_pb_ts(base + Duration::seconds(5));
    let in_screen = ScreenFrame {
        uuid: in_uuid.clone(),
        timestamp: in_ts,
        width: 100,
        height: 100,
        image_bytes: vec![0; 10],
        mime_type: "image/jpeg".to_string(),
        t_device: in_ts,
        t_canonical: in_ts,
        t_end: in_ts,
        ..Default::default()
    };
    let mut in_buf = Vec::new();
    in_screen.encode(&mut in_buf).unwrap();

    let out_uuid = lifelog_core::Uuid::new_v4().to_string();
    let out_ts = lifelog_types::to_pb_ts(base + Duration::seconds(30));
    let out_screen = ScreenFrame {
        uuid: out_uuid.clone(),
        timestamp: out_ts,
        width: 100,
        height: 100,
        image_bytes: vec![0; 10],
        mime_type: "image/jpeg".to_string(),
        t_device: out_ts,
        t_canonical: out_ts,
        t_end: out_ts,
        ..Default::default()
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

    let pcm_ts = lifelog_types::to_pb_ts(base);
    let pcm_end = lifelog_types::to_pb_ts(base + Duration::seconds(10));
    let pcm = AudioFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: pcm_ts,
        audio_bytes: vec![1; 10],
        codec: "pcm".to_string(),
        sample_rate: 48000,
        channels: 1,
        duration_secs: 10.0,
        t_device: pcm_ts,
        t_canonical: pcm_ts,
        t_end: pcm_end,
        ..Default::default()
    };
    let mut pcm_buf = Vec::new();
    pcm.encode(&mut pcm_buf).unwrap();

    let aac_ts = lifelog_types::to_pb_ts(base + Duration::seconds(120));
    let aac_end = lifelog_types::to_pb_ts(base + Duration::seconds(130));
    let aac = AudioFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: aac_ts,
        audio_bytes: vec![2; 10],
        codec: "aac".to_string(),
        sample_rate: 48000,
        channels: 1,
        duration_secs: 10.0,
        t_device: aac_ts,
        t_canonical: aac_ts,
        t_end: aac_end,
        ..Default::default()
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
            window: Duration::seconds(0),
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

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn test_during_conjunction_intersects_intervals() {
    let _ = tracing_subscriber::fmt::try_init();

    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    let collector_id = "test-collector";
    let session_id = 1u64;

    let base = Utc::now() - Duration::minutes(5);

    // Screen points: one outside intersection, one inside intersection.
    let screen_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "screen".to_string(),
        session_id,
    };

    let outside_uuid = lifelog_core::Uuid::new_v4().to_string();
    let outside_ts = lifelog_types::to_pb_ts(base + Duration::seconds(2));
    let outside_screen = ScreenFrame {
        uuid: outside_uuid.clone(),
        timestamp: outside_ts,
        width: 100,
        height: 100,
        image_bytes: vec![0; 10],
        mime_type: "image/jpeg".to_string(),
        t_device: outside_ts,
        t_canonical: outside_ts,
        t_end: outside_ts,
        ..Default::default()
    };
    let mut outside_buf = Vec::new();
    outside_screen.encode(&mut outside_buf).unwrap();

    let inside_uuid = lifelog_core::Uuid::new_v4().to_string();
    let inside_ts = lifelog_types::to_pb_ts(base + Duration::seconds(7));
    let inside_screen = ScreenFrame {
        uuid: inside_uuid.clone(),
        timestamp: inside_ts,
        width: 100,
        height: 100,
        image_bytes: vec![0; 10],
        mime_type: "image/jpeg".to_string(),
        t_device: inside_ts,
        t_canonical: inside_ts,
        t_end: inside_ts,
        ..Default::default()
    };
    let mut inside_buf = Vec::new();
    inside_screen.encode(&mut inside_buf).unwrap();

    let outside_chunk = lifelog_types::Chunk {
        stream: Some(screen_stream.clone()),
        offset: 0,
        data: outside_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let outside_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&outside_chunk.data),
        ..outside_chunk
    };

    let inside_offset = outside_chunk.data.len() as u64;
    let inside_chunk = lifelog_types::Chunk {
        stream: Some(screen_stream),
        offset: inside_offset,
        data: inside_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let inside_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&inside_chunk.data),
        ..inside_chunk
    };

    client
        .upload_chunks(tokio_stream::iter(vec![outside_chunk, inside_chunk]))
        .await
        .expect("Ingest screen failed");

    // Audio intervals:
    // - pcm: [base, base+10]
    // - aac: [base+5, base+15]
    // intersection: [base+5, base+10]
    let audio_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "audio".to_string(),
        session_id,
    };

    let pcm_ts = lifelog_types::to_pb_ts(base);
    let pcm_end = lifelog_types::to_pb_ts(base + Duration::seconds(10));
    let pcm = AudioFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: pcm_ts,
        audio_bytes: vec![1; 10],
        codec: "pcm".to_string(),
        sample_rate: 48000,
        channels: 1,
        duration_secs: 10.0,
        t_device: pcm_ts,
        t_canonical: pcm_ts,
        t_end: pcm_end,
        ..Default::default()
    };
    let mut pcm_buf = Vec::new();
    pcm.encode(&mut pcm_buf).unwrap();

    let aac_ts = lifelog_types::to_pb_ts(base + Duration::seconds(5));
    let aac_end = lifelog_types::to_pb_ts(base + Duration::seconds(15));
    let aac = AudioFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: aac_ts,
        audio_bytes: vec![2; 10],
        codec: "aac".to_string(),
        sample_rate: 48000,
        channels: 1,
        duration_secs: 10.0,
        t_device: aac_ts,
        t_canonical: aac_ts,
        t_end: aac_end,
        ..Default::default()
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

    let filter = ast::Expression::And(
        Box::new(ast::Expression::During {
            stream: ast::StreamSelector::StreamId(audio_origin.get_table_name()),
            predicate: Box::new(ast::Expression::Eq(
                "codec".to_string(),
                ast::Value::String("pcm".to_string()),
            )),
            window: Duration::seconds(0),
        }),
        Box::new(ast::Expression::During {
            stream: ast::StreamSelector::StreamId(audio_origin.get_table_name()),
            predicate: Box::new(ast::Expression::Eq(
                "codec".to_string(),
                ast::Value::String("aac".to_string()),
            )),
            window: Duration::seconds(0),
        }),
    );

    let query = ast::Query {
        target: ast::StreamSelector::StreamId(screen_origin.get_table_name()),
        filter,
    };

    let plan = planner::Planner::plan(&query, &[screen_origin.clone(), audio_origin.clone()]);
    let keys = executor::execute(&db, plan)
        .await
        .expect("DURING conjunction query execution failed");

    let keys_str: Vec<String> = keys.iter().map(|k| k.uuid.to_string()).collect();

    assert!(
        keys_str.contains(&inside_uuid),
        "Expected screen record inside pcm∩aac intersection to match"
    );
    assert!(
        !keys_str.contains(&outside_uuid),
        "Did not expect screen record outside intersection to match"
    );
}

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn test_during_interval_target_overlaps_point_sources() {
    let _ = tracing_subscriber::fmt::try_init();

    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    let collector_id = "test-collector";
    let session_id = 1u64;

    let base = Utc::now() - Duration::minutes(5);

    // Target: one AudioFrame interval [base, base+10s)
    let audio_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "audio".to_string(),
        session_id,
    };
    let audio_uuid = lifelog_core::Uuid::new_v4().to_string();
    let audio_ts = lifelog_types::to_pb_ts(base);
    let audio_end = lifelog_types::to_pb_ts(base + Duration::seconds(10));
    let audio = AudioFrame {
        uuid: audio_uuid.clone(),
        timestamp: audio_ts,
        audio_bytes: vec![1; 10],
        codec: "pcm".to_string(),
        sample_rate: 48000,
        channels: 1,
        duration_secs: 10.0,
        t_device: audio_ts,
        t_canonical: audio_ts,
        t_end: audio_end,
        ..Default::default()
    };
    let mut audio_buf = Vec::new();
    audio.encode(&mut audio_buf).unwrap();
    let audio_chunk = lifelog_types::Chunk {
        stream: Some(audio_stream),
        offset: 0,
        data: audio_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let audio_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&audio_chunk.data),
        ..audio_chunk
    };
    client
        .upload_chunks(tokio_stream::iter(vec![audio_chunk]))
        .await
        .expect("Ingest audio failed");

    // Source: one ScreenFrame point at base+5s.
    let screen_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "screen".to_string(),
        session_id,
    };
    let screen_ts = lifelog_types::to_pb_ts(base + Duration::seconds(5));
    let screen = ScreenFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: screen_ts,
        width: 123,
        height: 100,
        image_bytes: vec![0; 10],
        mime_type: "image/jpeg".to_string(),
        t_device: screen_ts,
        t_canonical: screen_ts,
        t_end: screen_ts,
        ..Default::default()
    };
    let mut screen_buf = Vec::new();
    screen.encode(&mut screen_buf).unwrap();
    let screen_chunk = lifelog_types::Chunk {
        stream: Some(screen_stream),
        offset: 0,
        data: screen_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let screen_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&screen_chunk.data),
        ..screen_chunk
    };
    client
        .upload_chunks(tokio_stream::iter(vec![screen_chunk]))
        .await
        .expect("Ingest screen failed");

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
        target: ast::StreamSelector::StreamId(audio_origin.get_table_name()),
        filter: ast::Expression::During {
            stream: ast::StreamSelector::StreamId(screen_origin.get_table_name()),
            predicate: Box::new(ast::Expression::Eq(
                "width".to_string(),
                ast::Value::Int(123),
            )),
            window: Duration::seconds(0),
        },
    };

    let plan = planner::Planner::plan(&query, &[screen_origin, audio_origin.clone()]);
    let keys = executor::execute(&db, plan)
        .await
        .expect("DURING interval-target query execution failed");

    let keys_str: Vec<String> = keys.iter().map(|k| k.uuid.to_string()).collect();
    assert!(
        keys_str.contains(&audio_uuid),
        "Expected audio interval overlapping source point to match"
    );
}
