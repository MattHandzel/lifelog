use super::planner::ExecutionPlan;
use lifelog_core::LifelogFrameKey;
use std::collections::BTreeSet;
use std::time::Duration;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tokio::time::timeout;

const DEFAULT_DB_QUERY_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_MAX_TARGET_UUIDS: usize = 1_000;

pub async fn execute(
    db: &Surreal<Client>,
    plan: ExecutionPlan,
) -> Result<Vec<LifelogFrameKey>, anyhow::Error> {
    match plan {
        ExecutionPlan::TableQuery { table, origin, sql } => {
            tracing::debug!(sql = %sql, table = %table, "Executing table query");

            let mut response = timeout(DEFAULT_DB_QUERY_TIMEOUT, db.query(sql)).await??;

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
            let mut seen = BTreeSet::new();
            for subplan in plans {
                let keys = Box::pin(execute(db, subplan)).await?;
                for k in keys {
                    let key = format!("{}:{}", k.origin.get_table_name(), k.uuid);
                    if seen.insert(key) {
                        all_keys.push(k);
                    }
                }
            }
            Ok(all_keys)
        }
        ExecutionPlan::DuringQuery {
            target_table,
            target_origin,
            target_base_where,
            during_terms,
            max_source_intervals,
            max_time_clauses,
        } => {
            #[derive(serde::Deserialize, Debug)]
            struct IntervalRow {
                t_canonical: Option<surrealdb::sql::Datetime>,
                t_end: Option<surrealdb::sql::Datetime>,
            }

            fn merge_intervals(
                mut intervals: Vec<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
                max_clauses: usize,
            ) -> Vec<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> {
                if intervals.is_empty() {
                    return intervals;
                }

                intervals.sort_by_key(|(s, _e)| *s);
                let mut merged = Vec::new();
                for (s, e) in intervals {
                    let (start, end) = if e < s { (s, s) } else { (s, e) };
                    match merged.last_mut() {
                        Some((_cur_start, cur_end)) => {
                            if start <= *cur_end {
                                if end > *cur_end {
                                    *cur_end = end;
                                }
                            } else {
                                merged.push((start, end));
                            }
                        }
                        None => merged.push((start, end)),
                    }
                    if merged.len() >= max_clauses {
                        break;
                    }
                }
                merged
            }

            fn intersect_intervals(
                a: &[(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)],
                b: &[(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)],
                max_clauses: usize,
            ) -> Vec<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> {
                let mut out = Vec::new();
                let mut i = 0usize;
                let mut j = 0usize;

                while i < a.len() && j < b.len() {
                    let (a0, a1) = a[i];
                    let (b0, b1) = b[j];
                    let start = core::cmp::max(a0, b0);
                    let end = core::cmp::min(a1, b1);
                    if start <= end {
                        out.push((start, end));
                        if out.len() >= max_clauses {
                            break;
                        }
                    }

                    if a1 <= b1 {
                        i += 1;
                    } else {
                        j += 1;
                    }
                }

                out
            }

            // Build and merge intervals per DURING term, then intersect them.
            let mut merged_terms: Vec<
                Vec<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
            > = Vec::new();

            for term in &during_terms {
                let mut intervals: Vec<(
                    chrono::DateTime<chrono::Utc>,
                    chrono::DateTime<chrono::Utc>,
                )> = Vec::new();

                for sp in &term.source_plans {
                    tracing::debug!(
                        source_table = %sp.source_table,
                        "Executing DURING source query"
                    );
                    let mut resp =
                        timeout(DEFAULT_DB_QUERY_TIMEOUT, db.query(sp.sql.clone())).await??;
                    let rows: Vec<IntervalRow> = resp.take(0)?;
                    tracing::debug!(
                        source_table = %sp.source_table,
                        rows = %rows.len(),
                        "DURING source query returned rows"
                    );

                    for r in rows {
                        let Some(t0) = r.t_canonical else {
                            continue;
                        };
                        let mut start = t0.0;
                        let mut end = r.t_end.map(|dt| dt.0).unwrap_or(start);

                        // Apply expansion window. For point source records, this becomes ±window.
                        start -= term.window;
                        end += term.window;
                        intervals.push((start, end));

                        if intervals.len() >= max_source_intervals {
                            break;
                        }
                    }

                    if intervals.len() >= max_source_intervals {
                        intervals.truncate(max_source_intervals);
                        break;
                    }
                }

                let merged = merge_intervals(intervals, max_time_clauses);
                if merged.is_empty() {
                    tracing::debug!("DURING: a term produced 0 intervals; returning 0 results");
                    return Ok(vec![]);
                }
                merged_terms.push(merged);
            }

            let mut iter = merged_terms.into_iter();
            let mut intersection = iter.next().unwrap_or_default();
            for other in iter {
                intersection = intersect_intervals(&intersection, &other, max_time_clauses);
                if intersection.is_empty() {
                    tracing::debug!("DURING: interval intersection is empty; returning 0 results");
                    return Ok(vec![]);
                }
            }

            tracing::debug!(
                intersection_intervals = %intersection.len(),
                "DURING: intersected intervals"
            );

            let mut clauses = Vec::with_capacity(intersection.len());
            for (start, end) in intersection {
                clauses.push(format!(
                    // Interval overlap: [t_canonical, t_end] overlaps [start, end]
                    "(t_canonical <= d'{}' AND t_end >= d'{}')",
                    end.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
                    start.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
                ));
            }
            let time_where = clauses.join(" OR ");

            let sql = format!(
                "SELECT uuid FROM `{}` WHERE ({}) AND ({}) LIMIT {};",
                target_table, target_base_where, time_where, DEFAULT_MAX_TARGET_UUIDS
            );

            tracing::info!(
                target_table = %target_table,
                during_terms = %during_terms.len(),
                time_clauses = %clauses.len(),
                sql_len = %sql.len(),
                "Executing DURING target query"
            );

            let mut response = timeout(DEFAULT_DB_QUERY_TIMEOUT, db.query(sql)).await??;

            #[derive(serde::Deserialize, Debug)]
            struct UuidResult {
                uuid: String,
            }

            let results: Vec<UuidResult> = response.take(0)?;
            tracing::info!(rows = %results.len(), "DURING target query returned uuids");

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
