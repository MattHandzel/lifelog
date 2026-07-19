//! Integration tests for the Postgres migration runner (MAT-1500).
//!
//! Boots a fresh Postgres via testcontainers and exercises `run_migrations`
//! end-to-end plus the startup drift assertion's failure path — the runtime
//! coverage the compile-time `EMBEDDED_MIGRATIONS` unit tests can't provide.

use lifelog_server::postgres::{connect_pool, run_migrations, verify_all_migrations_recorded};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;

/// Starts an isolated Postgres container and returns it (kept alive for the
/// test's lifetime) plus a connection URL. Never touches any local/prod DB.
///
/// Pinned to Postgres 16 to match prod (16.13); the testcontainers-modules
/// default tag is 11-alpine, which predates the `GENERATED ... STORED` columns
/// the schema relies on and would fail the very first migration.
async fn fresh_postgres() -> (ContainerAsync<Postgres>, String) {
    let container = Postgres::default()
        .with_tag("16-alpine")
        .start()
        .await
        .expect("start postgres testcontainer (is Docker running?)");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("get testcontainer host port");
    let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    (container, url)
}

/// The migration files on disk, sorted (= the set the runner must apply).
fn migration_files_on_disk() -> Vec<String> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/migrations");
    let mut files: Vec<String> = std::fs::read_dir(dir)
        .expect("read migrations dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".sql"))
        .collect();
    files.sort();
    files
}

/// Minimum bar: the runtime path actually applies every migration file to a
/// fresh DB and records it, and the startup assertion passes afterward.
#[tokio::test]
async fn run_migrations_applies_all_on_fresh_db() {
    let (_container, url) = fresh_postgres().await;
    let pool = connect_pool(&url, 4).await.expect("connect pool");

    run_migrations(&pool)
        .await
        .expect("run_migrations should apply every migration cleanly on a fresh DB");

    let client = pool.get().await.expect("get client");
    let rows = client
        .query(
            "SELECT version FROM schema_migrations ORDER BY version",
            &[],
        )
        .await
        .expect("query schema_migrations");
    let recorded: Vec<String> = rows.iter().map(|r| r.get::<_, String>(0)).collect();

    assert_eq!(
        recorded,
        migration_files_on_disk(),
        "every migrations/*.sql file must be applied and recorded on a fresh DB"
    );

    // The startup backstop must pass on a correctly-migrated DB (happy path).
    verify_all_migrations_recorded(&client)
        .await
        .expect("verify_all_migrations_recorded should pass after a full run");
}

/// Failure path: with a migration the runner knows about missing from
/// `schema_migrations`, the startup assertion returns an Err that names it.
#[tokio::test]
async fn verify_reports_missing_migration() {
    let (_container, url) = fresh_postgres().await;
    let pool = connect_pool(&url, 4).await.expect("connect pool");
    run_migrations(&pool).await.expect("run_migrations");

    // Simulate drift by removing one recorded version.
    let client = pool.get().await.expect("get client");
    let victim = "20260324100000_smart_search_doc.sql";
    let deleted = client
        .execute(
            "DELETE FROM schema_migrations WHERE version = $1",
            &[&victim],
        )
        .await
        .expect("delete a recorded migration");
    assert_eq!(
        deleted, 1,
        "victim migration should have been recorded first"
    );

    let err = verify_all_migrations_recorded(&client)
        .await
        .expect_err("verify must fail when a known migration is not recorded");
    let msg = err.to_string();
    assert!(
        msg.contains(victim),
        "error must name the missing migration; got: {msg}"
    );
}
