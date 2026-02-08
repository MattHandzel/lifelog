#![allow(clippy::print_stdout)]
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod harness;

use harness::TestContext;
use lifelog_core::Utc;
use lifelog_types::{BrowserFrame, GetDataRequest, Query, QueryRequest, ScreenFrame};
use prost::Message;

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn test_cross_modal_query() {
    let _ = tracing_subscriber::fmt::try_init();
    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    let collector_id = "test-collector";
    let session_id = 1u64;

    // 1. Ingest ScreenFrame
    let screen_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "screen".to_string(),
        session_id,
    };
    let screen_frame = ScreenFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: Some(lifelog_types::to_pb_ts(Utc::now()).unwrap()),
        width: 100,
        height: 100,
        image_bytes: vec![0; 10], // Dummy image
        mime_type: "image/jpeg".to_string(),
    };
    let mut screen_buf = Vec::new();
    screen_frame.encode(&mut screen_buf).unwrap();
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

    // 2. Ingest BrowserFrame
    let browser_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "browser".to_string(),
        session_id,
    };
    let browser_frame = BrowserFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: Some(lifelog_types::to_pb_ts(Utc::now()).unwrap()),
        url: "https://example.com/rust-lang".to_string(),
        title: "Rust Programming Language".to_string(),
        visit_count: 1,
    };
    let mut browser_buf = Vec::new();
    browser_frame.encode(&mut browser_buf).unwrap();
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

    let stream_screen = tokio_stream::iter(vec![screen_chunk]);
    client
        .upload_chunks(stream_screen)
        .await
        .expect("Ingest screen failed");

    let stream_browser = tokio_stream::iter(vec![browser_chunk]);
    client
        .upload_chunks(stream_browser)
        .await
        .expect("Ingest browser failed");

    // Wait for indexing
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // DEBUG: check if record exists directly
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

    let table = format!("{}:Browser", collector_id);
    let mut db_resp = db
        .query(format!("SELECT * FROM `{table}`"))
        .await
        .expect("Debug query failed");
    let all_browser: Vec<lifelog_types::BrowserRecord> = db_resp.take(0).expect("Take failed");
    println!("DEBUG: All browser records in {}: {:?}", table, all_browser);

    // 3. Perform Unified Search via Query and GetData

    // Search for "Rust"
    let query = Query {
        text: vec!["Rust".to_string()],
        ..Default::default()
    };
    let search_req = QueryRequest { query: Some(query) };

    let response = client.query(search_req).await.expect("Query failed");
    let keys = response.into_inner().keys;

    assert!(
        !keys.is_empty(),
        "Should have found at least one match for 'Rust'"
    );

    // Get data for these keys
    let get_data_req = GetDataRequest { keys };
    let response = client.get_data(get_data_req).await.expect("GetData failed");
    let results = response.into_inner().data;

    // Verify browser result
    let has_browser_result = results.iter().any(|r| {
        if let Some(lifelog_types::lifelog_data::Payload::Browserframe(bf)) = &r.payload {
            bf.title.contains("Rust")
        } else {
            false
        }
    });
    assert!(
        has_browser_result,
        "Should have returned the browser result"
    );

    // Search for everything (all modalities)
    let query_all = Query {
        text: vec!["*".to_string()],
        ..Default::default()
    };
    let search_all = QueryRequest {
        query: Some(query_all),
    };
    let _response = client.query(search_all).await.expect("Query failed");
}
