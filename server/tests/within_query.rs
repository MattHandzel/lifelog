#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod harness;

use chrono::{Duration, Utc};
use harness::TestContext;
use lifelog_core::{DataOrigin, DataOriginType};
use lifelog_server::query::{ast, executor, planner};
use lifelog_types::{BrowserFrame, ScreenFrame};
use prost::Message;

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn test_within_returns_target_records_near_source_matches() {
    let _ = tracing_subscriber::fmt::try_init();

    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    let collector_id = "test-collector";
    let session_id = 1u64;

    let base = Utc::now() - Duration::minutes(5);

    // Ingest two ScreenFrames: one close to the browser event, one far away.
    let screen_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "screen".to_string(),
        session_id,
    };

    let close_screen_uuid = lifelog_core::Uuid::new_v4().to_string();
    let close_ts = lifelog_types::to_pb_ts(base);
    let close_screen = ScreenFrame {
        uuid: close_screen_uuid.clone(),
        timestamp: close_ts,
        width: 100,
        height: 100,
        image_bytes: vec![0; 10],
        mime_type: "image/jpeg".to_string(),
        t_device: close_ts,
        t_canonical: close_ts,
        t_end: close_ts,
        ..Default::default()
    };
    let mut close_screen_buf = Vec::new();
    close_screen.encode(&mut close_screen_buf).unwrap();

    let far_screen_uuid = lifelog_core::Uuid::new_v4().to_string();
    let far_ts = lifelog_types::to_pb_ts(base + Duration::seconds(120));
    let far_screen = ScreenFrame {
        uuid: far_screen_uuid.clone(),
        timestamp: far_ts,
        width: 100,
        height: 100,
        image_bytes: vec![0; 10],
        mime_type: "image/jpeg".to_string(),
        t_device: far_ts,
        t_canonical: far_ts,
        t_end: far_ts,
        ..Default::default()
    };
    let mut far_screen_buf = Vec::new();
    far_screen.encode(&mut far_screen_buf).unwrap();

    let close_screen_chunk = lifelog_types::Chunk {
        stream: Some(screen_stream.clone()),
        offset: 0,
        data: close_screen_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let close_screen_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&close_screen_chunk.data),
        ..close_screen_chunk
    };

    let far_offset = close_screen_chunk.data.len() as u64;
    let far_screen_chunk = lifelog_types::Chunk {
        stream: Some(screen_stream.clone()),
        offset: far_offset,
        data: far_screen_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let far_screen_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&far_screen_chunk.data),
        ..far_screen_chunk
    };

    client
        .upload_chunks(tokio_stream::iter(vec![
            close_screen_chunk,
            far_screen_chunk,
        ]))
        .await
        .expect("Ingest screen failed");

    // Ingest a BrowserFrame near the close screen.
    let browser_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "browser".to_string(),
        session_id,
    };
    let browser_ts = lifelog_types::to_pb_ts(base + Duration::seconds(2));
    let browser = BrowserFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: browser_ts,
        url: "https://example.com/rust-lang".to_string(),
        title: "Rust Programming Language".to_string(),
        visit_count: 1,
        t_device: browser_ts,
        t_canonical: browser_ts,
        t_end: browser_ts,
        ..Default::default()
    };
    let mut browser_buf = Vec::new();
    browser.encode(&mut browser_buf).unwrap();

    let browser_chunk = lifelog_types::Chunk {
        stream: Some(browser_stream),
        offset: 0,
        data: browser_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let browser_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&browser_chunk.data),
        ..browser_chunk
    };

    client
        .upload_chunks(tokio_stream::iter(vec![browser_chunk]))
        .await
        .expect("Ingest browser failed");

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
    let browser_origin = DataOrigin::new(
        DataOriginType::DeviceId(collector_id.to_string()),
        "Browser".to_string(),
    );

    let query = ast::Query {
        target: ast::StreamSelector::StreamId(screen_origin.get_table_name()),
        filter: ast::Expression::Within {
            stream: ast::StreamSelector::StreamId(browser_origin.get_table_name()),
            predicate: Box::new(ast::Expression::Contains(
                "title".to_string(),
                "Rust".to_string(),
            )),
            window: Duration::seconds(5),
        },
    };

    let plan = planner::Planner::plan(&query, &[screen_origin.clone(), browser_origin.clone()]);
    let keys = executor::execute(&db, plan)
        .await
        .expect("WITHIN query execution failed");

    let keys_str: Vec<String> = keys.iter().map(|k| k.uuid.to_string()).collect();

    assert!(
        keys_str.contains(&close_screen_uuid),
        "Expected close screen record to match WITHIN window"
    );
    assert!(
        !keys_str.contains(&far_screen_uuid),
        "Did not expect far screen record to match WITHIN window"
    );
}

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn test_within_multiple_terms_intersects_windows() {
    let _ = tracing_subscriber::fmt::try_init();

    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    let collector_id = "test-collector";
    let session_id = 2u64;

    let base = Utc::now() - Duration::minutes(5);

    // Target: two screen points, only one falls into the intersection of the two WITHIN windows.
    let screen_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "screen".to_string(),
        session_id,
    };

    let hit_uuid = lifelog_core::Uuid::new_v4().to_string();
    let hit_ts = lifelog_types::to_pb_ts(base + Duration::seconds(8));
    let hit_screen = ScreenFrame {
        uuid: hit_uuid.clone(),
        timestamp: hit_ts,
        width: 100,
        height: 100,
        image_bytes: vec![0; 10],
        mime_type: "image/jpeg".to_string(),
        t_device: hit_ts,
        t_canonical: hit_ts,
        t_end: hit_ts,
        ..Default::default()
    };
    let mut hit_buf = Vec::new();
    hit_screen.encode(&mut hit_buf).unwrap();

    let miss_uuid = lifelog_core::Uuid::new_v4().to_string();
    let miss_ts = lifelog_types::to_pb_ts(base + Duration::seconds(40));
    let miss_screen = ScreenFrame {
        uuid: miss_uuid.clone(),
        timestamp: miss_ts,
        width: 100,
        height: 100,
        image_bytes: vec![0; 10],
        mime_type: "image/jpeg".to_string(),
        t_device: miss_ts,
        t_canonical: miss_ts,
        t_end: miss_ts,
        ..Default::default()
    };
    let mut miss_buf = Vec::new();
    miss_screen.encode(&mut miss_buf).unwrap();

    let hit_chunk = lifelog_types::Chunk {
        stream: Some(screen_stream.clone()),
        offset: 0,
        data: hit_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let hit_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&hit_chunk.data),
        ..hit_chunk
    };

    let miss_offset = hit_chunk.data.len() as u64;
    let miss_chunk = lifelog_types::Chunk {
        stream: Some(screen_stream.clone()),
        offset: miss_offset,
        data: miss_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let miss_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&miss_chunk.data),
        ..miss_chunk
    };

    client
        .upload_chunks(tokio_stream::iter(vec![hit_chunk, miss_chunk]))
        .await
        .expect("Ingest screen failed");

    // Source: two browser points close together, with distinct predicates.
    let browser_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "browser".to_string(),
        session_id,
    };

    let a_ts = lifelog_types::to_pb_ts(base + Duration::seconds(10));
    let a = BrowserFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: a_ts,
        url: "https://example.com/a".to_string(),
        title: "Rust".to_string(),
        visit_count: 1,
        t_device: a_ts,
        t_canonical: a_ts,
        t_end: a_ts,
        ..Default::default()
    };
    let mut a_buf = Vec::new();
    a.encode(&mut a_buf).unwrap();

    let b_ts = lifelog_types::to_pb_ts(base + Duration::seconds(12));
    let b = BrowserFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: b_ts,
        url: "https://example.com/b".to_string(),
        title: "Language".to_string(),
        visit_count: 1,
        t_device: b_ts,
        t_canonical: b_ts,
        t_end: b_ts,
        ..Default::default()
    };
    let mut b_buf = Vec::new();
    b.encode(&mut b_buf).unwrap();

    let a_chunk = lifelog_types::Chunk {
        stream: Some(browser_stream.clone()),
        offset: 0,
        data: a_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let a_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&a_chunk.data),
        ..a_chunk
    };

    let b_offset = a_chunk.data.len() as u64;
    let b_chunk = lifelog_types::Chunk {
        stream: Some(browser_stream),
        offset: b_offset,
        data: b_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let b_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&b_chunk.data),
        ..b_chunk
    };

    client
        .upload_chunks(tokio_stream::iter(vec![a_chunk, b_chunk]))
        .await
        .expect("Ingest browser failed");

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
    let browser_origin = DataOrigin::new(
        DataOriginType::DeviceId(collector_id.to_string()),
        "Browser".to_string(),
    );

    let query = ast::Query {
        target: ast::StreamSelector::StreamId(screen_origin.get_table_name()),
        filter: ast::Expression::And(
            Box::new(ast::Expression::Within {
                stream: ast::StreamSelector::StreamId(browser_origin.get_table_name()),
                predicate: Box::new(ast::Expression::Contains(
                    "title".to_string(),
                    "Rust".to_string(),
                )),
                window: Duration::seconds(5),
            }),
            Box::new(ast::Expression::Within {
                stream: ast::StreamSelector::StreamId(browser_origin.get_table_name()),
                predicate: Box::new(ast::Expression::Contains(
                    "title".to_string(),
                    "Language".to_string(),
                )),
                window: Duration::seconds(5),
            }),
        ),
    };

    let plan = planner::Planner::plan(&query, &[screen_origin.clone(), browser_origin.clone()]);
    let keys = executor::execute(&db, plan)
        .await
        .expect("multi-WITHIN query execution failed");

    let keys_str: Vec<String> = keys.iter().map(|k| k.uuid.to_string()).collect();
    assert!(
        keys_str.contains(&hit_uuid),
        "Expected screen record to match the intersection of both WITHIN windows"
    );
    assert!(
        !keys_str.contains(&miss_uuid),
        "Did not expect screen record outside the intersection to match"
    );
}
