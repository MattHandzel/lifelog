#![allow(clippy::print_stdout)]
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod harness;

use harness::TestContext;
use lifelog_core::{DataOrigin, DataOriginType, Utc};
use lifelog_types::{DataModality, ScreenFrame};
use prost::Message;
use std::time::Duration;

#[tokio::test]
#[ignore = "integration test: requires PostgreSQL and Tesseract"]
async fn test_ocr_transformation_pipeline() {
    let _ = tracing_subscriber::fmt::try_init();
    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    // 1. Ingest a ScreenFrame
    // We must use the hardcoded device ID from Server::new for now: "FF:FF:FF:FF:FF:FF"
    let _collector_id = "FFFFFFFFFFFF"; // replaces FF:FF:FF:FF:FF:FF in get_table_name
    let stream_id = "screen";
    let session_id = 123u64;

    let stream_identity = lifelog_types::StreamIdentity {
        collector_id: "FF:FF:FF:FF:FF:FF".to_string(),
        stream_id: stream_id.to_string(),
        session_id,
    };

    // Create a small white image with some text would be ideal, but even a blank one should trigger the pipeline
    let frame_ts = lifelog_types::to_pb_ts(Utc::now());
    let frame = ScreenFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: frame_ts,
        width: 100,
        height: 100,
        image_bytes: vec![0xFF; 100 * 100 * 3], // White square
        mime_type: "image/jpeg".to_string(),
        t_device: frame_ts,
        t_canonical: frame_ts,
        t_end: frame_ts,
        ..Default::default()
    };

    let mut buf = Vec::new();
    frame.encode(&mut buf).unwrap();

    let chunk = lifelog_types::Chunk {
        stream: Some(stream_identity.clone()),
        offset: 0,
        data: buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    // Correct hash
    let real_hash = utils::cas::sha256_hex(&chunk.data);
    let chunk = lifelog_types::Chunk {
        hash: real_hash,
        ..chunk
    };

    let stream = tokio_stream::iter(vec![chunk]);
    client.upload_chunks(stream).await.expect("Ingest failed");

    // 2. Wait for ServerPolicy to trigger transformation
    // Server loop runs every 100ms. OcrTransform watermark polling should pick it up.

    let mut success = false;
    let destination_origin = DataOrigin::new(
        DataOriginType::DataOrigin(Box::new(DataOrigin::new(
            DataOriginType::DeviceId("FF:FF:FF:FF:FF:FF".to_string()),
            DataModality::Screen.as_str_name().to_string(),
        ))),
        DataModality::Ocr.as_str_name().to_string(),
    );
    let table = destination_origin.get_table_name();

    let (pg_client, pg_conn) = tokio_postgres::connect(&ctx.pg_url, tokio_postgres::NoTls)
        .await
        .expect("connect to test postgres");
    tokio::spawn(pg_conn);

    println!("Waiting for OCR frame derived from source screen frame");

    let frame_uuid = frame.uuid.clone();

    for i in 0..20 {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let rows = pg_client
            .query(
                "SELECT COUNT(*) AS cnt FROM frames WHERE modality = 'Ocr' AND payload->>'uuid' = $1",
                &[&frame_uuid],
            )
            .await
            .expect("Query failed");

        if let Some(row) = rows.first() {
            let count: i64 = row.get(0);
            if count > 0 {
                println!("OCR result found after {}ms", i * 1000);
                success = true;
                break;
            }
        }
    }

    assert!(
        success,
        "OCR transformation did not complete within timeout"
    );
}
