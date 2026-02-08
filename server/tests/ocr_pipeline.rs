#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod harness;

use harness::TestContext;
use lifelog_types::{ScreenFrame,  DataModality};
use lifelog_core::{DataOrigin, DataOriginType, Utc};
use prost::Message;
use std::time::Duration;

#[tokio::test]
#[ignore = "integration test: requires SurrealDB and Tesseract"]
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
    let frame = ScreenFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: Some(lifelog_types::to_pb_ts(Utc::now()).unwrap()),
        width: 100,
        height: 100,
        image_bytes: vec![0xFF; 100 * 100 * 3], // White square
        mime_type: "image/jpeg".to_string(),
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

    println!("Waiting for OCR frame in table: {}", table);

    let frame_uuid = frame.uuid.clone();

    for i in 0..20 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        let mut response = db.query(format!("SELECT count() FROM `{table}` WHERE uuid = $uuid"))
            .bind(("uuid", frame_uuid.clone()))
            .await
            .expect("Query failed");
        
        let results: Vec<serde_json::Value> = response.take(0).expect("Take failed");
        if let Some(count_obj) = results.first() {
            let count = count_obj.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            if count > 0 {
                println!("OCR result found after {}ms", i * 1000);
                success = true;
                break;
            }
        }
    }

    assert!(success, "OCR transformation did not complete within timeout");
}