#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod harness;

use chrono::{Duration, Utc};
use harness::TestContext;
use lifelog_types::{AudioFrame, BrowserFrame, Query, QueryRequest};
use prost::Message;

#[tokio::test]
#[ignore = "integration test: requires PostgreSQL"]
async fn test_llql_canonical_example_audio_during_youtube_and_3b1b() {
    let _ = tracing_subscriber::fmt::try_init();

    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    let collector_id = "test-collector";
    let session_id = 42u64;

    let base = Utc::now() - Duration::minutes(5);

    // Ingest one AudioFrame interval [base, base+10s)
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
        sample_rate: 48_000,
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

    // Ingest one BrowserFrame point at base+6s matching "youtube".
    let browser_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "browser".to_string(),
        session_id,
    };

    let browser_ts = lifelog_types::to_pb_ts(base + Duration::seconds(6));
    let browser = BrowserFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: browser_ts,
        url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
        title: "Some Video".to_string(),
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

    // Seed an OcrRecord directly into the frames table so LLQL can resolve modality "Ocr".
    let (pg_client, pg_conn) = tokio_postgres::connect(&ctx.pg_url, tokio_postgres::NoTls)
        .await
        .expect("connect to test postgres");
    tokio::spawn(pg_conn);

    let ocr_uuid = lifelog_core::Uuid::new_v4().to_string();
    let ocr_ts = base + Duration::seconds(8);

    let payload = serde_json::json!({
        "uuid": ocr_uuid,
        "text": "Watching 3Blue1Brown",
        "timestamp": ocr_ts.to_rfc3339(),
    });

    pg_client
        .execute(
            "INSERT INTO frames (id, collector_id, stream_id, modality, t_canonical, t_end, t_ingest, payload, indexed, search_doc)
             VALUES ($1::uuid, $2, 'ocr', 'Ocr', $3, $3, now(), $4, true,
                     to_tsvector('english', $5))",
            &[
                &lifelog_core::Uuid::new_v4(),
                &collector_id,
                &ocr_ts,
                &payload,
                &"Watching 3Blue1Brown".to_string(),
            ],
        )
        .await
        .expect("insert ocr frame failed");

    // Give Postgres a moment to update indexes.
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;

    // Execute the canonical cross-modal query via LLQL.
    let llql = r#"llql:{
      "target": {"type":"modality","modality":"Audio"},
      "filter": {"op":"and","terms":[
        {"op":"during","stream":{"type":"modality","modality":"Browser"},"predicate":{"op":"contains","field":"url","text":"youtube"},"window":"5s"},
        {"op":"during","stream":{"type":"modality","modality":"Ocr"},"predicate":{"op":"contains","field":"text","text":"3Blue1Brown"},"window":"5s"}
      ]}
    }"#
    .to_string();

    let query = Query {
        text: vec![llql],
        ..Default::default()
    };

    let resp = client
        .query(QueryRequest { query: Some(query) })
        .await
        .expect("LLQL query failed")
        .into_inner();

    let uuids: Vec<String> = resp.keys.iter().map(|k| k.uuid.to_string()).collect();
    assert!(
        uuids.contains(&audio_uuid),
        "expected audio frame to match canonical DURING(browser AND ocr) query"
    );
}
