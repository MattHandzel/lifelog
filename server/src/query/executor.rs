use super::planner::ExecutionPlan;
use lifelog_core::uuid::Uuid;
use lifelog_core::{DataOrigin, DataOriginType, LifelogFrameKey};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

pub async fn execute(
    db: &Surreal<Client>,
    plan: ExecutionPlan,
) -> Result<Vec<LifelogFrameKey>, anyhow::Error> {
    match plan {
        ExecutionPlan::SimpleQuery(sql) => {
            tracing::debug!(sql = %sql, "Executing query");
            let mut response = db.query(sql).await?;

            // We assume the query is "SELECT * FROM table ..."
            // We want to extract 'id'.
            // response.take(0) gives the first result set.
            let results: Vec<surrealdb::sql::Thing> = response.take("id")?;

            let mut keys = Vec::new();
            for thing in results {
                // thing.tb is table name, thing.id is the ID part.
                // Assuming ID part is UUID string.
                if let Ok(uuid) = thing.id.to_string().parse::<Uuid>() {
                    let origin = lifelog_core::DataOrigin {
                        origin: lifelog_core::DataOriginType::DeviceId("unknown".to_string()),
                        modality_name: thing.tb,
                    };
                    keys.push(LifelogFrameKey { uuid, origin });
                }
            }
            Ok(keys)
        }
        ExecutionPlan::Unsupported(msg) => Err(anyhow::anyhow!("Unsupported query plan: {}", msg)),
    }
}
