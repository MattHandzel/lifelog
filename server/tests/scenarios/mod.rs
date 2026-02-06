//! Scenario DSL for multi-device sync integration tests.
//!
//! The builder pattern defines device specs, fault schedules, and invariants.
//! `ScenarioRunner` orchestrates execution and checks invariants.

pub mod generators;

use crate::harness::assertions;
use crate::harness::device_client::DeviceClient;
use crate::harness::event_gen::{collect_hashes, expected_final_offset, generate_chunk_sequence};
use crate::harness::fault_layer::{FaultController, FaultRule};
use crate::harness::TestContext;
use lifelog_proto::Chunk;
use std::sync::Arc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Scenario types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct StreamSpec {
    pub stream_id: String,
    pub session_id: u64,
    pub num_chunks: usize,
    pub chunk_size: usize,
}

#[derive(Clone, Debug)]
pub struct DeviceSpec {
    pub id: String,
    pub streams: Vec<StreamSpec>,
    pub registration_delay: Option<Duration>,
}

#[derive(Clone, Debug)]
pub enum FaultSchedule {
    /// Reject the next N requests globally.
    #[allow(dead_code)]
    RejectNextN { n: u64, code: tonic::Code },
    /// Drop every Nth request.
    DropEveryN { n: u64 },
}

#[derive(Clone, Debug)]
pub enum Invariant {
    /// All chunks uploaded by all devices arrived at the server.
    AllDataArrived,
    /// No duplicate data in the CAS (inherent in content-addressing, but we verify offsets).
    NoDataDuplication,
    /// Final offsets match expected values for each device/stream/session.
    CorrectFinalOffsets,
    /// Devices' data doesn't leak into other devices' namespaces.
    IsolatedDeviceStreams,
}

// ---------------------------------------------------------------------------
// Scenario builder
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub struct SyncScenario {
    pub name: String,
    pub devices: Vec<DeviceSpec>,
    pub faults: Vec<FaultSchedule>,
    pub invariants: Vec<Invariant>,
    pub seed: u64,
}

impl SyncScenario {
    pub fn builder(name: impl Into<String>) -> SyncScenarioBuilder {
        SyncScenarioBuilder {
            name: name.into(),
            devices: Vec::new(),
            faults: Vec::new(),
            invariants: Vec::new(),
            seed: 0,
        }
    }
}

pub struct SyncScenarioBuilder {
    name: String,
    devices: Vec<DeviceSpec>,
    faults: Vec<FaultSchedule>,
    invariants: Vec<Invariant>,
    seed: u64,
}

impl SyncScenarioBuilder {
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn device(mut self, id: impl Into<String>, streams: Vec<StreamSpec>) -> Self {
        self.devices.push(DeviceSpec {
            id: id.into(),
            streams,
            registration_delay: None,
        });
        self
    }

    pub fn device_with_delay(
        mut self,
        id: impl Into<String>,
        streams: Vec<StreamSpec>,
        delay: Duration,
    ) -> Self {
        self.devices.push(DeviceSpec {
            id: id.into(),
            streams,
            registration_delay: Some(delay),
        });
        self
    }

    pub fn fault(mut self, fault: FaultSchedule) -> Self {
        self.faults.push(fault);
        self
    }

    #[allow(dead_code)]
    pub fn invariant(mut self, invariant: Invariant) -> Self {
        self.invariants.push(invariant);
        self
    }

    pub fn invariants(mut self, invariants: Vec<Invariant>) -> Self {
        self.invariants.extend(invariants);
        self
    }

    pub fn build(self) -> SyncScenario {
        SyncScenario {
            name: self.name,
            devices: self.devices,
            faults: self.faults,
            invariants: self.invariants,
            seed: self.seed,
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: generate stream spec shorthand
// ---------------------------------------------------------------------------

pub fn stream(
    stream_id: &str,
    session_id: u64,
    num_chunks: usize,
    chunk_size: usize,
) -> StreamSpec {
    StreamSpec {
        stream_id: stream_id.to_string(),
        session_id,
        num_chunks,
        chunk_size,
    }
}

// ---------------------------------------------------------------------------
// Scenario runner
// ---------------------------------------------------------------------------

pub struct ScenarioRunner;

impl ScenarioRunner {
    pub async fn run(scenario: &SyncScenario) {
        // Set up fault controller from scenario
        let fault_controller = FaultController::new();
        for fault in &scenario.faults {
            match fault {
                FaultSchedule::RejectNextN { n, code } => {
                    fault_controller
                        .add_rule(FaultRule::reject_next_n(*n, *code, "scenario fault"))
                        .await;
                }
                FaultSchedule::DropEveryN { n } => {
                    fault_controller.add_rule(FaultRule::drop_every_n(*n)).await;
                }
            }
        }

        let ctx = Arc::new(TestContext::new_with_faults(fault_controller).await);

        // Pre-generate all chunks for each device/stream
        struct DeviceWork {
            spec: DeviceSpec,
            stream_chunks: Vec<(StreamSpec, Vec<Chunk>)>,
        }

        let mut device_works: Vec<DeviceWork> = Vec::new();
        for (di, device) in scenario.devices.iter().enumerate() {
            let mut stream_chunks = Vec::new();
            for (si, stream_spec) in device.streams.iter().enumerate() {
                let seed = scenario.seed.wrapping_add(di as u64 * 1000 + si as u64);
                let chunks = generate_chunk_sequence(
                    &device.id,
                    &stream_spec.stream_id,
                    stream_spec.session_id,
                    stream_spec.num_chunks,
                    stream_spec.chunk_size,
                    seed,
                );
                stream_chunks.push((stream_spec.clone(), chunks));
            }
            device_works.push(DeviceWork {
                spec: device.clone(),
                stream_chunks,
            });
        }

        // Execute each device's work concurrently
        let mut handles = Vec::new();
        for work in device_works {
            let ctx = Arc::clone(&ctx);
            let handle = tokio::spawn(async move {
                if let Some(delay) = work.spec.registration_delay {
                    tokio::time::sleep(delay).await;
                }

                let mut device = DeviceClient::new(&work.spec.id, ctx.client());
                device.register().await.expect("register failed");

                for (stream_spec, chunks) in &work.stream_chunks {
                    // Retry on fault injection failures
                    let mut attempts = 0;
                    loop {
                        let result = device.upload_chunks(chunks.clone()).await;
                        match result {
                            Ok(_) => break,
                            Err(status)
                                if status.code() == tonic::Code::Unavailable && attempts < 5 =>
                            {
                                attempts += 1;
                                tokio::time::sleep(Duration::from_millis(100 * attempts)).await;
                                // Reconnect
                                device = DeviceClient::new(&work.spec.id, ctx.client());
                                device.register().await.expect("re-register failed");
                            }
                            Err(e) => panic!(
                                "Upload failed for {}/{}: {e}",
                                work.spec.id, stream_spec.stream_id
                            ),
                        }
                    }
                }

                // Return info needed for invariant checks
                work.stream_chunks
                    .iter()
                    .map(|(spec, chunks)| {
                        (
                            work.spec.id.clone(),
                            spec.stream_id.clone(),
                            spec.session_id,
                            expected_final_offset(spec.num_chunks, spec.chunk_size),
                            collect_hashes(chunks),
                        )
                    })
                    .collect::<Vec<_>>()
            });
            handles.push(handle);
        }

        // Collect results
        let mut all_results: Vec<(String, String, u64, u64, Vec<String>)> = Vec::new();
        for handle in handles {
            let results = handle.await.expect("device task panicked");
            all_results.extend(results);
        }

        // Check invariants
        let mut client = ctx.client();
        for invariant in &scenario.invariants {
            match invariant {
                Invariant::CorrectFinalOffsets => {
                    for (cid, sid, sess, expected_offset, _) in &all_results {
                        assertions::assert_final_offset(
                            &mut client,
                            cid,
                            sid,
                            *sess,
                            *expected_offset,
                        )
                        .await;
                    }
                }
                Invariant::AllDataArrived => {
                    let cas = ctx.cas();
                    for (_, _, _, _, hashes) in &all_results {
                        assertions::assert_cas_contains(&cas, hashes);
                    }
                }
                Invariant::IsolatedDeviceStreams => {
                    let specs: Vec<_> = all_results
                        .iter()
                        .map(|(cid, sid, sess, _, _)| (cid.clone(), sid.clone(), *sess))
                        .collect();
                    assertions::assert_no_cross_contamination(&mut client, &specs).await;
                }
                Invariant::NoDataDuplication => {
                    // Offsets being correct implies no duplication at the stream level.
                    // CAS deduplication is inherent. Just verify offsets.
                    for (cid, sid, sess, expected_offset, _) in &all_results {
                        assertions::assert_final_offset(
                            &mut client,
                            cid,
                            sid,
                            *sess,
                            *expected_offset,
                        )
                        .await;
                    }
                }
            }
        }
    }
}
