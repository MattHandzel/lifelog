#![allow(clippy::print_stdout)]
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod harness;

use harness::TestContext;
use lifelog_core::Utc;
use lifelog_types::{BrowserFrame, GetDataRequest, Query, QueryRequest};
use prost::Message;
use std::time::{Duration, Instant};

#[tokio::test]
#[ignore = "performance suite: slow and resource intensive"]
async fn test_performance_suite_smoke() {
    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    let collector_id = "perf-collector";
    let session_id = 999u64;
    let num_records = 500;

    println!("Starting Performance Suite...");
    println!("Seeding {} browser records...", num_records);

    // 1. Measure Ingest Throughput
    let start_ingest = Instant::now();
    for i in 0..num_records {
        let browser_frame = BrowserFrame {
            uuid: lifelog_core::Uuid::new_v4().to_string(),
            timestamp: Some(lifelog_types::to_pb_ts(Utc::now()).unwrap()),
            url: format!("https://example.com/page-{}", i),
            title: format!("Rust Performance Testing Page {}", i),
            visit_count: 1,
        };
        let mut buf = Vec::new();
        browser_frame.encode(&mut buf).unwrap();

        let stream_id = lifelog_types::StreamIdentity {
            collector_id: collector_id.to_string(),
            stream_id: "browser".to_string(),
            session_id,
        };

        let chunk = lifelog_types::Chunk {
            stream: Some(stream_id),
            offset: (i as u64) * 1000, // Dummy offset
            data: buf,
            hash: utils::cas::sha256_hex(&[]),
        };
        let chunk = lifelog_types::Chunk {
            hash: utils::cas::sha256_hex(&chunk.data),
            ..chunk
        };

        let stream = tokio_stream::iter(vec![chunk]);
        client.upload_chunks(stream).await.expect("Ingest failed");
    }
    let ingest_duration = start_ingest.elapsed();
    let ingest_throughput = (num_records as f64) / ingest_duration.as_secs_f64();
    println!(
        "Ingest Throughput: {:.2} records/sec (Total: {:?})",
        ingest_throughput, ingest_duration
    );

    // 2. Wait for indexing (SurrealDB memory KVS is fast but let's give it a bit)
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 3. Measure Query Latency
    println!("Measuring Query Latency...");
    let query = Query {
        text: vec!["Performance".to_string()],
        ..Default::default()
    };
    let search_req = QueryRequest { query: Some(query) };

    let start_query = Instant::now();
    let response = client.query(search_req).await.expect("Query failed");
    let query_duration = start_query.elapsed();
    let keys = response.into_inner().keys;
    println!(
        "Query Latency: {:?} (Results: {})",
        query_duration,
        keys.len()
    );

    // 4. Measure GetData Latency
    println!("Measuring GetData Latency for 50 records...");
    let sample_keys = if keys.len() > 50 {
        keys[0..50].to_vec()
    } else {
        keys.clone()
    };

    let get_data_req = GetDataRequest { keys: sample_keys };
    let start_get = Instant::now();
    let response = client.get_data(get_data_req).await.expect("GetData failed");
    let get_duration = start_get.elapsed();
    let results = response.into_inner().data;
    println!(
        "GetData Latency: {:?} (Results: {})",
        get_duration,
        results.len()
    );

    // 5. Baseline Thresholds
    assert!(
        ingest_throughput > 10.0,
        "Ingest throughput too low: {:.2} < 10.0",
        ingest_throughput
    );
    assert!(
        query_duration < Duration::from_millis(1000),
        "Query latency too high: {:?}",
        query_duration
    );
    assert!(
        get_duration < Duration::from_millis(2000),
        "GetData latency too high: {:?}",
        get_duration
    );
}
