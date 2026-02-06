#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod harness;

use harness::event_gen::{collect_hashes, expected_final_offset, generate_chunk_sequence};
use harness::TestContext;
use std::sync::Arc;

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn test_multi_device_simulation() {
    let ctx = Arc::new(TestContext::new().await);
    let num_devices = 5;
    let chunks_per_device = 3;
    let chunk_size = 64;

    let mut handles = Vec::new();

    for i in 0..num_devices {
        let ctx = Arc::clone(&ctx);
        let handle = tokio::spawn(async move {
            let mut device =
                harness::device_client::DeviceClient::new(format!("device-{i}"), ctx.client());
            device.register().await.expect("register failed");

            let session_id = 1000 + i as u64;
            let chunks = generate_chunk_sequence(
                &device.device_id,
                "main",
                session_id,
                chunks_per_device,
                chunk_size,
                i as u64,
            );

            device.upload_chunks(chunks).await.expect("upload failed");

            let offset = device
                .get_offset("main", session_id)
                .await
                .expect("get_offset failed");
            assert_eq!(offset, expected_final_offset(chunks_per_device, chunk_size));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Device task failed");
    }
}

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn test_10_devices_concurrent() {
    let ctx = Arc::new(TestContext::new().await);
    let num_devices = 10;
    let chunks_per_device = 5;
    let chunk_size = 128;

    let mut handles = Vec::new();

    for i in 0..num_devices {
        let ctx = Arc::clone(&ctx);
        let handle = tokio::spawn(async move {
            let mut device =
                harness::device_client::DeviceClient::new(format!("concurrent-{i}"), ctx.client());
            device.register().await.expect("register failed");

            let session_id = 2000 + i as u64;
            let chunks = generate_chunk_sequence(
                &device.device_id,
                "stream-0",
                session_id,
                chunks_per_device,
                chunk_size,
                100 + i as u64,
            );
            let expected_hashes = collect_hashes(&chunks);

            device.upload_chunks(chunks).await.expect("upload failed");

            let offset = device
                .get_offset("stream-0", session_id)
                .await
                .expect("get_offset failed");
            assert_eq!(offset, expected_final_offset(chunks_per_device, chunk_size));

            // Verify CAS contains all our chunks
            harness::assertions::assert_cas_contains(&ctx.cas(), &expected_hashes);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Device task failed");
    }
}

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn test_device_isolation() {
    let ctx = TestContext::new().await;
    let chunk_size = 64;

    // Upload data for two devices with different sessions
    let mut dev_a = harness::device_client::DeviceClient::new("iso-device-a", ctx.client());
    let mut dev_b = harness::device_client::DeviceClient::new("iso-device-b", ctx.client());

    dev_a.register().await.expect("register A");
    dev_b.register().await.expect("register B");

    let chunks_a = generate_chunk_sequence("iso-device-a", "s1", 100, 3, chunk_size, 10);
    let chunks_b = generate_chunk_sequence("iso-device-b", "s1", 200, 3, chunk_size, 20);

    dev_a.upload_chunks(chunks_a).await.expect("upload A");
    dev_b.upload_chunks(chunks_b).await.expect("upload B");

    // Device A's offset should not include device B's data and vice versa
    let mut client = ctx.client();
    harness::assertions::assert_final_offset(
        &mut client,
        "iso-device-a",
        "s1",
        100,
        expected_final_offset(3, chunk_size),
    )
    .await;
    harness::assertions::assert_final_offset(
        &mut client,
        "iso-device-b",
        "s1",
        200,
        expected_final_offset(3, chunk_size),
    )
    .await;

    // Cross-contamination check
    let specs = vec![
        ("iso-device-a".to_string(), "s1".to_string(), 100u64),
        ("iso-device-b".to_string(), "s1".to_string(), 200u64),
    ];
    harness::assertions::assert_no_cross_contamination(&mut client, &specs).await;
}

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn test_registration_before_upload() {
    let ctx = TestContext::new().await;
    let mut device = harness::device_client::DeviceClient::new("unregistered", ctx.client());

    // Upload without registering first — should still work at the gRPC level
    // (the server accepts chunks even without explicit registration via ControlStream)
    let chunks = generate_chunk_sequence("unregistered", "s1", 500, 2, 32, 42);
    let result = device.upload_chunks(chunks).await;

    // The upload itself should succeed (chunks are stored regardless of registration)
    assert!(result.is_ok());
}
