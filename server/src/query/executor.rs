use super::planner::ExecutionPlan;
use lifelog_core::LifelogFrameKey;
use std::collections::BTreeSet;
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
        ExecutionPlan::WithinQuery {
            target_table,
            target_origin,
            target_base_where,
            source_plans,
            window,
            max_source_timestamps,
            max_time_clauses,
        } => {
            #[derive(serde::Deserialize, Debug)]
            struct TsResult {
                timestamp: surrealdb::sql::Datetime,
            }

            let mut timestamps = Vec::new();
            for sp in &source_plans {
                tracing::debug!(source_table = %sp.source_table, "Executing WITHIN source query");
                let mut resp = db.query(sp.sql.clone()).await?;
                let rows: Vec<TsResult> = resp.take(0)?;
                tracing::debug!(
                    source_table = %sp.source_table,
                    rows = %rows.len(),
                    "WITHIN source query returned timestamps"
                );
                timestamps.extend(rows.into_iter().map(|r| r.timestamp.0));
                if timestamps.len() >= max_source_timestamps {
                    timestamps.truncate(max_source_timestamps);
                    break;
                }
            }

            if timestamps.is_empty() {
                tracing::debug!("WITHIN: no source timestamps; returning 0 results");
                return Ok(vec![]);
            }

            // Merge overlapping windows to keep the SQL size bounded.
            timestamps.sort();
            let mut merged: Vec<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> =
                Vec::new();
            for ts in timestamps {
                let start = ts - window;
                let end = ts + window;
                match merged.last_mut() {
                    Some((cur_start, cur_end)) => {
                        if start <= *cur_end {
                            if end > *cur_end {
                                *cur_end = end;
                            }
                            if start < *cur_start {
                                *cur_start = start;
                            }
                        } else {
                            merged.push((start, end));
                        }
                    }
                    None => merged.push((start, end)),
                }
                if merged.len() >= max_time_clauses {
                    break;
                }
            }
            tracing::debug!(
                merged_windows = %merged.len(),
                "WITHIN: merged time windows"
            );

            // Build OR'd time predicates.
            let mut clauses = Vec::with_capacity(merged.len());
            for (start, end) in merged {
                clauses.push(format!(
                    "(timestamp >= d'{}' AND timestamp <= d'{}')",
                    start.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
                    end.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
                ));
            }
            let time_where = clauses.join(" OR ");

            let sql = format!(
                "SELECT uuid FROM `{}` WHERE ({}) AND ({});",
                target_table, target_base_where, time_where
            );

            tracing::info!(
                target_table = %target_table,
                source_tables = %source_plans.len(),
                time_clauses = %clauses.len(),
                sql_len = %sql.len(),
                "Executing WITHIN target query"
            );

            let mut response = db.query(sql).await?;

            #[derive(serde::Deserialize, Debug)]
            struct UuidResult {
                uuid: String,
            }
            let results: Vec<UuidResult> = response.take(0)?;
            tracing::info!(rows = %results.len(), "WITHIN target query returned uuids");

            // Defensive: DISTINCT should remove duplicates, but keep a set anyway.
            let mut seen = BTreeSet::new();
            let mut keys = Vec::new();
            for res in results {
                if let Ok(uuid) = res.uuid.parse::<lifelog_core::uuid::Uuid>() {
                    if seen.insert(uuid) {
                        keys.push(LifelogFrameKey {
                            uuid,
                            origin: target_origin.clone(),
                        });
                    }
                }
            }
            Ok(keys)
        }
        ExecutionPlan::Unsupported(msg) => Err(anyhow::anyhow!("Unsupported query plan: {}", msg)),
    }
}
