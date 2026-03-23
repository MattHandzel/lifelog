use lifelog_core::LifelogError;
use std::str::FromStr;
use tokio_postgres::NoTls;

pub type PostgresPool = deadpool_postgres::Pool;

pub fn is_postgres_uri(endpoint: &str) -> bool {
    let trimmed = endpoint.trim();
    trimmed.starts_with("postgres://") || trimmed.starts_with("postgresql://")
}

struct EmbeddedMigration {
    version: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[EmbeddedMigration] = &[
    EmbeddedMigration {
        version: "20260303143000_init_postgres.sql",
        sql: include_str!("../migrations/20260303143000_init_postgres.sql"),
    },
    EmbeddedMigration {
        version: "20260322000000_transform_pipeline.sql",
        sql: include_str!("../migrations/20260322000000_transform_pipeline.sql"),
    },
    EmbeddedMigration {
        version: "20260323000000_unified_frames.sql",
        sql: include_str!("../migrations/20260323000000_unified_frames.sql"),
    },
    EmbeddedMigration {
        version: "20260323100000_migrate_to_frames.sql",
        sql: include_str!("../migrations/20260323100000_migrate_to_frames.sql"),
    },
    EmbeddedMigration {
        version: "20260323200000_drop_legacy_tables.sql",
        sql: include_str!("../migrations/20260323200000_drop_legacy_tables.sql"),
    },
];

pub async fn connect_pool(
    database_url: &str,
    max_connections: usize,
) -> Result<PostgresPool, LifelogError> {
    tracing::debug!("Connecting to Postgres with URI: {}", database_url);
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
    let mut client = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Postgres pool get failed during migrations: {e}");
            return Err(LifelogError::Database(format!(
                "postgres pool get failed: {e}"
            )));
        }
    };
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version TEXT PRIMARY KEY,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );",
        )
        .await
        .map_err(|e| LifelogError::Database(format!("postgres migration bootstrap failed: {e}")))?;

    for migration in MIGRATIONS {
        let already = client
            .query_opt(
                "SELECT version FROM schema_migrations WHERE version = $1",
                &[&migration.version],
            )
            .await
            .map_err(|e| LifelogError::Database(format!("migration query failed: {e}")))?;
        if already.is_some() {
            continue;
        }

        let tx = client
            .transaction()
            .await
            .map_err(|e| LifelogError::Database(format!("migration tx begin failed: {e}")))?;
        tx.batch_execute(migration.sql).await.map_err(|e| {
            LifelogError::Database(format!("apply migration {} failed: {e}", migration.version))
        })?;
        tx.execute(
            "INSERT INTO schema_migrations(version) VALUES ($1)",
            &[&migration.version],
        )
        .await
        .map_err(|e| {
            LifelogError::Database(format!(
                "record migration {} failed: {e}",
                migration.version
            ))
        })?;
        tx.commit().await.map_err(|e| {
            LifelogError::Database(format!(
                "commit migration {} failed: {e}",
                migration.version
            ))
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
