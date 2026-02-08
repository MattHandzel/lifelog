use super::planner::ExecutionPlan;
use lifelog_core::LifelogFrameKey;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

pub async fn execute(
    db: &Surreal<Client>,
    plan: ExecutionPlan,
) -> Result<Vec<LifelogFrameKey>, anyhow::Error> {
    match plan {
        ExecutionPlan::TableQuery { table, origin, sql } => {
            tracing::debug!(sql = %sql, table = %table, "Executing table query");

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
                    keys.push(LifelogFrameKey {
                        uuid,
                        origin: origin.clone(),
                    });
                }
            }
            Ok(keys)
        }
        ExecutionPlan::MultiQuery(plans) => {
            let mut all_keys = Vec::new();
            for subplan in plans {
                let keys = Box::pin(execute(db, subplan)).await?;
                all_keys.extend(keys);
            }
            Ok(all_keys)
        }
        ExecutionPlan::Unsupported(msg) => Err(anyhow::anyhow!("Unsupported query plan: {}", msg)),
    }
}
