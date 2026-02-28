#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod harness;

use chrono::{Duration, Utc};
use harness::TestContext;
use lifelog_types::{BrowserFrame, ReplayRequest, ScreenFrame};
use prost::Message;

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn test_replay_query_returns_ordered_steps_with_context() {
    let _ = tracing_subscriber::fmt::try_init();

    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    let collector_id = "test-collector";
    let session_id = 3u64;

    let base = Utc::now() - Duration::minutes(10);

    // Ingest three ScreenFrames to form two full steps + one trailing step.
    let screen_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "screen".to_string(),
        session_id,
    };

    let mut screen_chunks = Vec::new();
    let mut offset = 0;
    for i in 0..3 {
        let ts = lifelog_types::to_pb_ts(base + Duration::seconds(i * 10)); // t=0, t=10, t=20
        let screen = ScreenFrame {
            uuid: lifelog_core::Uuid::new_v4().to_string(),
            timestamp: ts.clone(),
            width: 100,
            height: 100,
            image_bytes: vec![0; 10],
            mime_type: "image/jpeg".to_string(),
            t_device: ts.clone(),
            t_canonical: ts.clone(),
            t_end: ts,
            ..Default::default()
        };
        let mut buf = Vec::new();
        screen.encode(&mut buf).unwrap();

        let chunk = lifelog_types::Chunk {
            stream: Some(screen_stream.clone()),
            offset,
            data: buf.clone(),
            hash: utils::cas::sha256_hex(&buf),
        };
        offset += buf.len() as u64;
        screen_chunks.push(chunk);
    }

    client
        .upload_chunks(tokio_stream::iter(screen_chunks))
        .await
        .expect("Ingest screen failed");

    // Ingest a BrowserFrame (Context) at t=15 (falls in step 2: [10, 20))
    let browser_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "browser".to_string(),
        session_id,
    };
    let browser_ts = lifelog_types::to_pb_ts(base + Duration::seconds(15));
    let browser = BrowserFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: browser_ts.clone(),
        url: "https://example.com/replay".to_string(),
        title: "Replay Test".to_string(),
        visit_count: 1,
        t_device: browser_ts.clone(),
        t_canonical: browser_ts.clone(),
        t_end: browser_ts,
        ..Default::default()
    };
    let mut browser_buf = Vec::new();
    browser.encode(&mut browser_buf).unwrap();

    let browser_chunk = lifelog_types::Chunk {
        stream: Some(browser_stream),
        offset: 0,
        data: browser_buf.clone(),
        hash: utils::cas::sha256_hex(&browser_buf),
    };

    client
        .upload_chunks(tokio_stream::iter(vec![browser_chunk]))
        .await
        .expect("Ingest browser failed");

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Execute Replay Request
    let req = ReplayRequest {
        window: Some(lifelog_types::Timerange {
            start: lifelog_types::to_pb_ts(base - Duration::seconds(5)),
            end: lifelog_types::to_pb_ts(base + Duration::seconds(25)),
        }),
        screen_origin: format!("{}:Screen", collector_id),
        context_origins: vec![format!("{}:Browser", collector_id)],
        max_steps: 100,
        context_pad_ms: 0,
        max_context_per_step: 10,
    };

    let resp = client
        .replay(req)
        .await
        .expect("Replay RPC failed")
        .into_inner();

    assert_eq!(resp.steps.len(), 3, "Expected 3 steps");

    // Check step 1: [0, 10)
    assert_eq!(
        resp.steps[0].context_keys.len(),
        0,
        "Step 1 should have no context"
    );

    // Check step 2: [10, 20) - Should contain the browser frame
    assert_eq!(
        resp.steps[1].context_keys.len(),
        1,
        "Step 2 should have 1 context key"
    );

    // Check step 3: [20, window_end)
    assert_eq!(
        resp.steps[2].context_keys.len(),
        0,
        "Step 3 should have no context"
    );
}
