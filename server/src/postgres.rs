use lifelog_core::LifelogError;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio_postgres::NoTls;

pub type PostgresPool = deadpool_postgres::Pool;

pub fn is_postgres_uri(endpoint: &str) -> bool {
    let trimmed = endpoint.trim();
    trimmed.starts_with("postgres://") || trimmed.starts_with("postgresql://")
}

pub fn migrations_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
}

pub async fn connect_pool(
    database_url: &str,
    max_connections: usize,
) -> Result<PostgresPool, LifelogError> {
    let cfg =
        tokio_postgres::Config::from_str(database_url).map_err(|e| LifelogError::Validation {
            field: "database_endpoint".to_string(),
            reason: format!("invalid postgres uri: {e}"),
        })?;
    let mgr_cfg = deadpool_postgres::ManagerConfig {
        recycling_method: deadpool_postgres::RecyclingMethod::Fast,
    };
    let mgr = deadpool_postgres::Manager::from_config(cfg, NoTls, mgr_cfg);
    deadpool_postgres::Pool::builder(mgr)
        .max_size(max_connections)
        .build()
        .map_err(|e| LifelogError::Database(format!("postgres pool build failed: {e}")))
}

pub async fn run_migrations(pool: &PostgresPool) -> Result<(), LifelogError> {
    let mut client = pool
        .get()
        .await
        .map_err(|e| LifelogError::Database(format!("postgres pool get failed: {e}")))?;
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version TEXT PRIMARY KEY,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );",
        )
        .await
        .map_err(|e| LifelogError::Database(format!("postgres migration bootstrap failed: {e}")))?;

    let mut files = std::fs::read_dir(migrations_dir())
        .map_err(|e| LifelogError::Database(format!("read migrations dir failed: {e}")))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension() == Some(OsStr::new("sql")))
        .collect::<Vec<_>>();
    files.sort();

    for path in files {
        let version = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| LifelogError::Database("invalid migration filename".to_string()))?
            .to_string();
        let already = client
            .query_opt(
                "SELECT version FROM schema_migrations WHERE version = $1",
                &[&version],
            )
            .await
            .map_err(|e| LifelogError::Database(format!("migration query failed: {e}")))?;
        if already.is_some() {
            continue;
        }

        let sql = std::fs::read_to_string(&path).map_err(|e| {
            LifelogError::Database(format!("read migration {} failed: {e}", path.display()))
        })?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| LifelogError::Database(format!("migration tx begin failed: {e}")))?;
        tx.batch_execute(&sql).await.map_err(|e| {
            LifelogError::Database(format!("apply migration {version} failed: {e}"))
        })?;
        tx.execute(
            "INSERT INTO schema_migrations(version) VALUES ($1)",
            &[&version],
        )
        .await
        .map_err(|e| LifelogError::Database(format!("record migration {version} failed: {e}")))?;
        tx.commit().await.map_err(|e| {
            LifelogError::Database(format!("commit migration {version} failed: {e}"))
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::is_postgres_uri;

    #[test]
    fn detects_postgres_uris() {
        assert!(is_postgres_uri("postgres://user:pass@localhost/lifelog"));
        assert!(is_postgres_uri("postgresql://user:pass@localhost/lifelog"));
    }

    #[test]
    fn rejects_non_postgres_uris() {
        assert!(!is_postgres_uri("127.0.0.1:7183"));
        assert!(!is_postgres_uri("ws://127.0.0.1:7183"));
        assert!(!is_postgres_uri("http://localhost:5432"));
    }
}
