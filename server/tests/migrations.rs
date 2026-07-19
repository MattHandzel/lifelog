//! Integration tests for the Postgres migration runner (MAT-1500).
//!
//! Boots a fresh Postgres via testcontainers and exercises `run_migrations`
//! end-to-end plus the startup drift assertion's failure path — the runtime
//! coverage the compile-time `EMBEDDED_MIGRATIONS` unit tests can't provide.

use lifelog_server::postgres::{connect_pool, run_migrations, verify_migration_consistency};
use std::sync::{Arc, Mutex};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;

/// In-memory `tracing` writer so a test can assert what was logged. Scoped via
/// `set_default` (thread-local), which captures across awaits under the
/// current-thread `#[tokio::test]` runtime.
#[derive(Clone, Default)]
struct LogCapture(Arc<Mutex<Vec<u8>>>);

impl LogCapture {
    fn contents(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
    }
}

impl std::io::Write for LogCapture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl tracing_subscriber::fmt::MakeWriter<'_> for LogCapture {
    type Writer = LogCapture;
    fn make_writer(&self) -> Self::Writer {
        self.clone()
    }
}

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

/// DB ahead of this binary (a newer build migrated it — e.g. a rollback):
/// `schema_migrations` holds a version this build doesn't know. The check must
/// **boot anyway** (return Ok) and **log loudly** naming the unknown version —
/// refusing to boot would break emergency rollback. This is the real reachable
/// condition, not an artificial one (the apply loop can never leave an embedded
/// migration unrecorded).
#[tokio::test]
async fn verify_boots_and_warns_when_db_ahead() {
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

    // Capture logs emitted during the check.
    let capture = LogCapture::default();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(capture.clone())
        .with_max_level(tracing::Level::ERROR)
        .finish();

    let result = {
        let _guard = tracing::subscriber::set_default(subscriber);
        verify_migration_consistency(&client).await
    };

    // Boots: DB-ahead must NOT fail closed.
    result.expect("verify must NOT fail closed when the DB is merely ahead of the binary");

    // Logs: the warning must be loud and name the unknown version.
    let logs = capture.contents();
    assert!(
        logs.contains(from_the_future),
        "log must name the unknown migration; got: {logs}"
    );
    assert!(
        logs.contains("BEHIND the schema"),
        "log must flag that the server is running behind the schema; got: {logs}"
    );
}

/// Defense-in-depth failure path: if an embedded migration the binary applies is
/// somehow not recorded, the runner is broken — the check must fail closed with an
/// Err naming it. (This state is unreachable via `run_migrations`; we force it here
/// to prove the branch's behavior.)
#[tokio::test]
async fn verify_errors_when_embedded_migration_not_recorded() {
    let (_container, url) = fresh_postgres().await;
    let pool = connect_pool(&url, 4).await.expect("connect pool");
    run_migrations(&pool).await.expect("run_migrations");

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

    let err = verify_migration_consistency(&client)
        .await
        .expect_err("verify must fail closed when an embedded migration is not recorded");
    let msg = err.to_string();
    assert!(
        msg.contains(victim),
        "error must name the unrecorded migration; got: {msg}"
    );
}
