//! Assertion helpers for multi-device sync integration tests.
//!
//! These check invariants against the database and CAS after scenarios complete.

#![allow(clippy::expect_used, dead_code)]

use lifelog_types::lifelog_server_service_client::LifelogServerServiceClient;
use lifelog_types::GetUploadOffsetRequest;
use tonic::transport::Channel;
use utils::cas::FsCas;

/// Assert that the upload offset for a given device/stream/session matches the expected value.
pub async fn assert_final_offset(
    client: &mut LifelogServerServiceClient<Channel>,
    collector_id: &str,
    stream_id: &str,
    session_id: u64,
    expected_offset: u64,
) {
    let resp = client
        .get_upload_offset(GetUploadOffsetRequest {
            stream: Some(lifelog_types::StreamIdentity {
                collector_id: collector_id.to_string(),
                stream_id: stream_id.to_string(),
                session_id,
            }),
        })
        .await
        .expect("get_upload_offset failed")
        .into_inner();

    pretty_assertions::assert_eq!(
        resp.offset,
        expected_offset,
        "Final offset mismatch for {collector_id}/{stream_id}/{session_id}"
    );
}

/// Assert that all provided hashes exist in the CAS.
pub fn assert_cas_contains(cas: &FsCas, hashes: &[String]) {
    for hash in hashes {
        assert!(
            cas.contains(hash)
                .unwrap_or_else(|e| panic!("CAS lookup failed for {hash}: {e}")),
            "CAS missing blob: {hash}"
        );
    }
}

/// Assert that no device's data appears under another device's collector_id.
/// This queries the offset for each device_id against all other devices' session_ids.
/// If cross-contamination exists, offsets would be non-zero for mismatched pairs.
pub async fn assert_no_cross_contamination(
    client: &mut LifelogServerServiceClient<Channel>,
    device_specs: &[(String, String, u64)], // (collector_id, stream_id, session_id)
) {
    for (i, (cid_a, sid_a, _sess_a)) in device_specs.iter().enumerate() {
        for (j, (_cid_b, _sid_b, sess_b)) in device_specs.iter().enumerate() {
            if i == j {
                continue;
            }
            // Query device A's collector_id with device B's session_id
            let resp = client
                .get_upload_offset(GetUploadOffsetRequest {
                    stream: Some(lifelog_types::StreamIdentity {
                        collector_id: cid_a.clone(),
                        stream_id: sid_a.clone(),
                        session_id: *sess_b,
                    }),
                })
                .await
                .expect("get_upload_offset failed")
                .into_inner();

            assert_eq!(
                resp.offset, 0,
                "Cross-contamination: device {cid_a} has data for session {sess_b}"
            );
        }
    }
}
