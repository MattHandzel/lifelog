//! Validation suite skeleton.
//!
//! This maps directly to `VALIDATION_SUITE.md` integration test IDs.
//! Tests are `#[ignore]` until the underlying end-to-end plumbing exists.

#[test]
#[ignore = "IT-010 (VALIDATION_SUITE.md): requires deterministic ingest + query engine + indexes"]
fn it_010_cross_modal_query_end_to_end() {}

#[test]
#[ignore = "IT-060 (VALIDATION_SUITE.md): requires multi-collector canonical time + time_quality"]
fn it_060_canonical_time_across_devices() {}

#[test]
#[ignore = "IT-080 (VALIDATION_SUITE.md): requires disk WAL + crash/restart + upload protocol"]
fn it_080_crash_restart_durability() {}

#[test]
#[ignore = "IT-090 (VALIDATION_SUITE.md): requires resumable upload by byte offsets"]
fn it_090_resume_upload_with_byte_offsets() {}

#[test]
#[ignore = "IT-081 (VALIDATION_SUITE.md): requires ack gating on index completion"]
fn it_081_ack_implies_queryable() {}

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

