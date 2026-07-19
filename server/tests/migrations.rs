//! Integration tests for the Postgres migration runner (MAT-1500).
//!
//! Boots a fresh Postgres via testcontainers and exercises `run_migrations`
//! end-to-end plus the startup drift assertion's failure path — the runtime
//! coverage the compile-time `EMBEDDED_MIGRATIONS` unit tests can't provide.

use lifelog_server::postgres::{connect_pool, run_migrations, verify_migration_consistency};
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

    // The startup consistency check must pass on a correctly-migrated DB.
    verify_migration_consistency(&client)
        .await
        .expect("verify_migration_consistency should pass after a full run");
}

/// Reachable failure path: the DB is ahead of this binary — `schema_migrations`
/// holds a version this build doesn't know (a newer build migrated it). This is
/// the real condition the check guards against (e.g. a bad rollback), not an
/// artificial one — the apply loop can never leave an embedded migration
/// unrecorded, so that direction is unreachable at runtime. The check must return
/// an Err naming the unknown version.
#[tokio::test]
async fn verify_detects_db_ahead_of_binary() {
    let (_container, url) = fresh_postgres().await;
    let pool = connect_pool(&url, 4).await.expect("connect pool");
    run_migrations(&pool).await.expect("run_migrations");

    // A future build's migration this binary has never heard of.
    let client = pool.get().await.expect("get client");
    let from_the_future = "29990101000000_added_by_a_newer_build.sql";
    let inserted = client
        .execute(
            "INSERT INTO schema_migrations(version) VALUES ($1)",
            &[&from_the_future],
        )
        .await
        .expect("insert an unknown recorded migration");
    assert_eq!(inserted, 1, "should have recorded the phantom migration");

    let err = verify_migration_consistency(&client)
        .await
        .expect_err("verify must fail when the DB has a migration unknown to this build");
    let msg = err.to_string();
    assert!(
        msg.contains(from_the_future),
        "error must name the unknown migration; got: {msg}"
    );
}
