//! Validation suite skeleton.
//!
//! This maps directly to `VALIDATION_SUITE.md` integration test IDs.
//! Tests are `#[ignore]` until the underlying end-to-end plumbing exists.

mod harness;
use harness::TestContext;

#[tokio::test]
async fn test_harness_smoke() {
    let ctx = TestContext::new().await;
    println!("Server address: {}", ctx.server_addr);
    // ctx dropped here kills surrealdb
}

#[test]
#[ignore = "IT-010 (VALIDATION_SUITE.md): requires deterministic ingest + query engine + indexes"]
fn it_010_cross_modal_query_end_to_end() {}

#[test]
#[ignore = "IT-060 (VALIDATION_SUITE.md): requires multi-collector canonical time + time_quality"]
fn it_060_canonical_time_across_devices() {}

#[test]
#[ignore = "IT-080 (VALIDATION_SUITE.md): requires disk WAL + crash/restart + upload protocol"]
fn it_080_crash_restart_durability() {}

#[tokio::test]
async fn it_090_resume_upload_with_byte_offsets() {
    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    let collector_id = "test-collector";
    let stream_id = "test-stream";
    let session_id = 12345u64;

    // 1. Upload first chunk
    let data1 = b"hello world";
    let hash1 = utils::cas::sha256_hex(data1);
    let chunk1 = lifelog_proto::Chunk {
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        session_id,
        offset: 0,
        data: data1.to_vec(),
        hash: hash1,
    };

    let stream = tokio_stream::iter(vec![chunk1]);
    let response = client.upload_chunks(stream).await.expect("Upload failed");
    let ack = response.into_inner();
    // Indexed is false, so ACK should be 0
    assert_eq!(ack.acked_offset, 0);

    // 2. Get offset (should be 11 because we have chunk 0..11)
    let offset_resp = client.get_upload_offset(lifelog_proto::GetUploadOffsetRequest {
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        session_id,
    }).await.expect("Get offset failed").into_inner();
    
    assert_eq!(offset_resp.offset, 11);

    // 3. Upload second chunk at offset 11
    let data2 = b" next part"; // length 10
    let hash2 = utils::cas::sha256_hex(data2);
    let chunk2 = lifelog_proto::Chunk {
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        session_id,
        offset: 11,
        data: data2.to_vec(),
        hash: hash2,
    };

    let stream = tokio_stream::iter(vec![chunk2]);
    let response = client.upload_chunks(stream).await.expect("Upload failed");
    let _ack = response.into_inner();
    
    // Verify highest offset in DB is now 11 + 10 = 21
    let offset_resp = client.get_upload_offset(lifelog_proto::GetUploadOffsetRequest {
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        session_id,
    }).await.expect("Get offset failed").into_inner();
    
    assert_eq!(offset_resp.offset, 21);
}

#[tokio::test]
async fn it_081_ack_implies_queryable() {
    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    let collector_id = "test-collector-81";
    let stream_id = "test-stream-81";
    let session_id = 9999u64;

    let data = b"chunk1";
    let hash = utils::cas::sha256_hex(data);
    let chunk = lifelog_proto::Chunk {
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        session_id,
        offset: 0,
        data: data.to_vec(),
        hash: hash.clone(),
    };

    // 1. Upload chunk. Default backend has indexed=false.
    // Expect acked_offset = 0 (chunk received but not indexed).
    let stream = tokio_stream::iter(vec![chunk.clone()]);
    let response = client.upload_chunks(stream).await.expect("Upload failed").into_inner();
    
    assert_eq!(response.acked_offset, 0, "ACK should be 0 because indexing is pending");

    // 2. Simulate "Indexer" completing work.
    // We manually flip 'indexed' to true in SurrealDB for this chunk.
    let db = surrealdb::Surreal::new::<surrealdb::engine::remote::ws::Ws>(&ctx.db_addr).await.expect("DB Connect failed");
    db.signin(surrealdb::opt::auth::Root {
        username: "root",
        password: "root",
    }).await.expect("DB Signin failed");
    db.use_ns("lifelog").use_db("test_db").await.expect("DB Select failed");

    let id = format!("{}-{}-{}-{}", collector_id, stream_id, session_id, 0);
    
    #[derive(serde::Deserialize)]
    struct ChunkRec {
        indexed: bool,
    }

    // Check it exists first
    let result: Option<ChunkRec> = db.select(("upload_chunks", &id)).await.expect("Select failed");
    assert!(result.is_some(), "Chunk record should exist");

    // Update to indexed=true
    let _updated: Option<ChunkRec> = db.query("UPDATE type::thing('upload_chunks', $id) SET indexed = true")
        .bind(("id", id.clone()))
        .await
        .expect("Update failed")
        .take(0)
        .expect("Take failed");

    // 3. Upload next chunk (or same chunk to probe ACK).
    // If we re-send the same chunk, apply_chunk is idempotent, and it should check indexing again.
    let stream = tokio_stream::iter(vec![chunk]);
    let response = client.upload_chunks(stream).await.expect("Upload failed").into_inner();

    assert_eq!(response.acked_offset, data.len() as u64, "ACK should advance after indexing is complete");
}

#[test]
#[ignore = "IT-100 (VALIDATION_SUITE.md): requires blob separation DB vs CAS"]
fn it_100_blob_separation() {}

#[test]
#[ignore = "IT-110 (VALIDATION_SUITE.md): requires OCR transform + derived stream + indexing"]
fn it_110_ocr_transform_pipeline() {}

#[test]
#[ignore = "IT-130 (VALIDATION_SUITE.md): requires query cancellation support"]
fn it_130_ui_query_cancellation() {}

#[test]
#[ignore = "IT-140 (VALIDATION_SUITE.md): requires TLS + pairing enforcement in gRPC"]
fn it_140_tls_and_pairing_enforcement() {}

#[test]
#[ignore = "IT-150 (VALIDATION_SUITE.md): requires system state observability surface"]
fn it_150_observability_surface() {}

#[test]
#[ignore = "IT-160 (VALIDATION_SUITE.md): requires performance suite entrypoint"]
fn it_160_performance_suite_smoke() {}

