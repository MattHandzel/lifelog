#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod harness;
mod scenarios;

use scenarios::generators;
use scenarios::ScenarioRunner;

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn scenario_happy_path_3_devices() {
    let scenario = generators::happy_path_n_devices(3);
    ScenarioRunner::run(&scenario).await;
}

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn scenario_happy_path_5_devices() {
    let scenario = generators::happy_path_n_devices(5);
    ScenarioRunner::run(&scenario).await;
}

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn scenario_network_flakiness() {
    let scenario = generators::network_flakiness(3);
    ScenarioRunner::run(&scenario).await;
}

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn scenario_duplicate_chunk_idempotency() {
    let scenario = generators::duplicate_chunk_idempotency();
    ScenarioRunner::run(&scenario).await;
}

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn scenario_hash_verification() {
    let scenario = generators::hash_verification();
    ScenarioRunner::run(&scenario).await;
}

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn scenario_interleaved_multi_stream() {
    let scenario = generators::interleaved_multi_stream();
    ScenarioRunner::run(&scenario).await;
}

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn scenario_staggered_registration() {
    let scenario = generators::staggered_registration();
    ScenarioRunner::run(&scenario).await;
}
