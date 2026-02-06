//! Pre-built scenario definitions for common sync test patterns.

use super::*;

/// Happy path: N devices upload without any faults.
pub fn happy_path_n_devices(n: usize) -> SyncScenario {
    let mut builder = SyncScenario::builder("happy_path")
        .seed(42)
        .invariants(vec![
            Invariant::AllDataArrived,
            Invariant::CorrectFinalOffsets,
            Invariant::IsolatedDeviceStreams,
        ]);

    for i in 0..n {
        builder = builder.device(
            format!("happy-{i}"),
            vec![stream("main", 1000 + i as u64, 5, 128)],
        );
    }

    builder.build()
}

/// Duplicate chunk idempotency: uploads the same chunks multiple times.
/// Offsets should still be correct (no double-counting).
pub fn duplicate_chunk_idempotency() -> SyncScenario {
    SyncScenario::builder("duplicate_chunk_idempotency")
        .seed(77)
        .device("dup-device", vec![stream("main", 300, 3, 64)])
        .invariants(vec![
            Invariant::CorrectFinalOffsets,
            Invariant::NoDataDuplication,
        ])
        .build()
}

/// Network flakiness: every 3rd request is dropped.
/// Devices should retry and eventually succeed.
pub fn network_flakiness(drop_rate: u64) -> SyncScenario {
    SyncScenario::builder("network_flakiness")
        .seed(99)
        .device("flaky-a", vec![stream("main", 400, 4, 64)])
        .device("flaky-b", vec![stream("main", 401, 4, 64)])
        .fault(FaultSchedule::DropEveryN { n: drop_rate })
        .invariants(vec![
            Invariant::AllDataArrived,
            Invariant::CorrectFinalOffsets,
            Invariant::IsolatedDeviceStreams,
        ])
        .build()
}

/// Hash corruption detection: tested at the unit level (Tier 0).
/// At the integration level, we ensure valid hashes pass and the
/// scenario completes correctly.
pub fn hash_verification() -> SyncScenario {
    SyncScenario::builder("hash_verification")
        .seed(111)
        .device("hash-dev", vec![stream("verified", 500, 6, 256)])
        .invariants(vec![
            Invariant::AllDataArrived,
            Invariant::CorrectFinalOffsets,
        ])
        .build()
}

/// Interleaved multi-stream: each device uploads to multiple streams concurrently.
pub fn interleaved_multi_stream() -> SyncScenario {
    SyncScenario::builder("interleaved_multi_stream")
        .seed(222)
        .device(
            "multi-a",
            vec![
                stream("screen", 600, 3, 128),
                stream("audio", 601, 4, 64),
                stream("browser", 602, 2, 256),
            ],
        )
        .device(
            "multi-b",
            vec![stream("screen", 700, 3, 128), stream("audio", 701, 4, 64)],
        )
        .invariants(vec![
            Invariant::AllDataArrived,
            Invariant::CorrectFinalOffsets,
            Invariant::IsolatedDeviceStreams,
        ])
        .build()
}

/// Staggered registration: devices register at different times.
pub fn staggered_registration() -> SyncScenario {
    SyncScenario::builder("staggered_registration")
        .seed(333)
        .device("early-dev", vec![stream("main", 800, 5, 64)])
        .device_with_delay(
            "late-dev",
            vec![stream("main", 801, 5, 64)],
            Duration::from_millis(500),
        )
        .invariants(vec![
            Invariant::AllDataArrived,
            Invariant::CorrectFinalOffsets,
            Invariant::IsolatedDeviceStreams,
        ])
        .build()
}
