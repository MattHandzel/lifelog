use super::planner::ExecutionPlan;
use lifelog_core::LifelogFrameKey;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

pub async fn execute(
    db: &Surreal<Client>,
    plan: ExecutionPlan,
) -> Result<Vec<LifelogFrameKey>, anyhow::Error> {
    match plan {
        ExecutionPlan::SimpleQuery(sql) => {
            tracing::debug!(sql = %sql, "Executing query");

            // Extract table from SQL query string before moving sql
            let table = sql.split('`').nth(1).unwrap_or("unknown").to_string();

            let mut response = db.query(sql).await?;

            #[derive(serde::Deserialize, Debug)]
            struct UuidResult {
                uuid: String,
            }

            // Extract record UUIDs as strings
            let results: Vec<UuidResult> = response.take(0)?;

            let mut keys = Vec::new();
            for res in results {
                let id_str = res.uuid;

                if let Ok(uuid) = id_str.parse::<lifelog_core::uuid::Uuid>() {
                    if let Ok(origin) = lifelog_core::DataOrigin::tryfrom_string(table.clone()) {
                        keys.push(LifelogFrameKey { uuid, origin });
                    }
                }
            }
            Ok(keys)
        }
        ExecutionPlan::Unsupported(msg) => Err(anyhow::anyhow!("Unsupported query plan: {}", msg)),
    }
}
