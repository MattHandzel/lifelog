use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lifelog_core::LifelogError;

use crate::postgres::PostgresPool;

#[async_trait]
pub trait WatermarkStore: Send + Sync {
    async fn get(&self, transform_id: &str, origin: &str) -> Result<DateTime<Utc>, LifelogError>;
    async fn set(
        &self,
        transform_id: &str,
        origin: &str,
        ts: DateTime<Utc>,
    ) -> Result<(), LifelogError>;
}

pub struct PostgresWatermarkStore {
    pool: PostgresPool,
}

impl PostgresWatermarkStore {
    pub fn new(pool: PostgresPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WatermarkStore for PostgresWatermarkStore {
    async fn get(&self, transform_id: &str, origin: &str) -> Result<DateTime<Utc>, LifelogError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

        let row = client
            .query_opt(
                "SELECT cursor_value FROM transform_watermarks WHERE transform_id = $1 AND origin = $2",
                &[&transform_id, &origin],
            )
            .await
            .map_err(|e| LifelogError::Database(format!("watermark get: {e}")))?;

        match row {
            Some(r) => {
                let val: String = r.get(0);
                val.parse::<DateTime<Utc>>()
                    .map_err(|e| LifelogError::Database(format!("watermark parse: {e}")))
            }
            None => Ok(DateTime::<Utc>::from_timestamp(0, 0).unwrap_or_default()),
        }
    }

    async fn set(
        &self,
        transform_id: &str,
        origin: &str,
        ts: DateTime<Utc>,
    ) -> Result<(), LifelogError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

        let ts_str = ts.to_rfc3339();
        client
            .execute(
                "INSERT INTO transform_watermarks (transform_id, origin, cursor_value, updated_at)
                 VALUES ($1, $2, $3, NOW())
                 ON CONFLICT (transform_id, origin) DO UPDATE SET cursor_value = $3, updated_at = NOW()",
                &[&transform_id, &origin, &ts_str],
            )
            .await
            .map_err(|e| LifelogError::Database(format!("watermark set: {e}")))?;

        Ok(())
    }
}
