use super::ast::{Expression, Value};
use super::planner::ExecutionPlan;
use crate::postgres::PostgresPool;
use anyhow::anyhow;
use lifelog_core::{DataOrigin, DataOriginType, LifelogFrameKey};
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
        ExecutionPlan::TableQuery {
            table, origin, sql, ..
        } => {
            tracing::debug!(sql = %sql, table = %table, "Executing table query");

            let mut response = timeout(DEFAULT_DB_QUERY_TIMEOUT, db.query(sql)).await??;

            #[derive(serde::Deserialize, Debug)]
            struct UuidResult {
                uuid: String,
            }

            let results: Vec<UuidResult> = response.take(0)?;

            let mut keys = Vec::new();
            for res in results {
                if let Ok(uuid) = res.uuid.parse::<lifelog_core::uuid::Uuid>() {
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
            ..
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

            let mut clauses = Vec::with_capacity(intersection.len());
            for (start, end) in intersection {
                clauses.push(format!(
                    "(t_canonical <= d'{}' AND (t_end >= d'{}' OR t_canonical >= d'{}'))",
                    end.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
                    start.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
                    start.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
                ));
            }
            let time_where = clauses.join(" OR ");

            let sql = format!(
                "SELECT uuid FROM `{}` WHERE ({}) AND ({}) LIMIT {};",
                target_table, target_base_where, time_where, DEFAULT_MAX_TARGET_UUIDS
            );

            let mut response = timeout(DEFAULT_DB_QUERY_TIMEOUT, db.query(sql)).await??;

            #[derive(serde::Deserialize, Debug)]
            struct UuidResult {
                uuid: String,
            }

            let results: Vec<UuidResult> = response.take(0)?;

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
        ExecutionPlan::Unsupported(msg) => Err(anyhow!("Unsupported query plan: {}", msg)),
    }
}

pub async fn execute_postgres(
    pool: &PostgresPool,
    plan: ExecutionPlan,
) -> Result<Vec<LifelogFrameKey>, anyhow::Error> {
    match plan {
        ExecutionPlan::TableQuery {
            origin,
            filter,
            limit,
            ..
        } => execute_postgres_table_query(pool, origin, filter, limit).await,
        ExecutionPlan::MultiQuery(plans) => {
            let mut all_keys = Vec::new();
            let mut seen = BTreeSet::new();
            for subplan in plans {
                let keys = Box::pin(execute_postgres(pool, subplan)).await?;
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
            target_origin,
            target_base_filter,
            during_terms,
            target_limit,
            ..
        } => {
            execute_postgres_during_query(
                pool,
                target_origin,
                target_base_filter,
                during_terms,
                target_limit,
            )
            .await
        }
        ExecutionPlan::Unsupported(msg) => Err(anyhow!("Unsupported query plan: {}", msg)),
    }
}

pub fn plan_is_postgres_compatible(plan: &ExecutionPlan) -> bool {
    match plan {
        ExecutionPlan::TableQuery { origin, .. } => pg_table_for_origin(origin).is_some(),
        ExecutionPlan::MultiQuery(plans) => plans.iter().all(plan_is_postgres_compatible),
        ExecutionPlan::DuringQuery {
            target_origin,
            during_terms,
            ..
        } => {
            pg_table_for_origin(target_origin).is_some()
                && during_terms.iter().all(|term| {
                    term.source_plans
                        .iter()
                        .all(|sp| pg_table_for_origin(&sp.source_origin).is_some())
                })
        }
        ExecutionPlan::Unsupported(_) => false,
    }
}

async fn execute_postgres_table_query(
    pool: &PostgresPool,
    origin: DataOrigin,
    filter: Option<Expression>,
    limit: usize,
) -> Result<Vec<LifelogFrameKey>, anyhow::Error> {
    if limit == 0 {
        return Ok(vec![]);
    }

    let table = pg_table_for_origin(&origin).ok_or_else(|| {
        anyhow!(
            "postgres table not available for modality {}",
            origin.modality_name
        )
    })?;

    let origin_scope = compile_origin_scope_sql("t", &origin);
    let filter_sql = filter
        .as_ref()
        .map(|f| compile_expression_pg_sql(f, "t", table))
        .unwrap_or_else(|| "TRUE".to_string());

    let sql = format!(
        "SELECT t.id::text AS id FROM {table} t WHERE ({origin_scope}) AND ({filter_sql}) ORDER BY lower(t.time_range) ASC LIMIT {limit}"
    );

    let client = pool.get().await?;
    let rows = timeout(DEFAULT_DB_QUERY_TIMEOUT, client.query(sql.as_str(), &[])).await??;

    let mut keys = Vec::new();
    for row in rows {
        let id_str: String = row.get("id");
        if let Ok(uuid) = id_str.parse::<lifelog_core::uuid::Uuid>() {
            keys.push(LifelogFrameKey {
                uuid,
                origin: origin.clone(),
            });
        }
    }

    Ok(keys)
}

async fn execute_postgres_during_query(
    pool: &PostgresPool,
    target_origin: DataOrigin,
    target_base_filter: Option<Expression>,
    during_terms: Vec<super::planner::DuringTermPlan>,
    target_limit: usize,
) -> Result<Vec<LifelogFrameKey>, anyhow::Error> {
    if target_limit == 0 {
        return Ok(vec![]);
    }

    let target_table = pg_table_for_origin(&target_origin).ok_or_else(|| {
        anyhow!(
            "postgres table not available for target modality {}",
            target_origin.modality_name
        )
    })?;

    let mut where_clauses = Vec::new();
    where_clauses.push(compile_origin_scope_sql("t", &target_origin));
    if let Some(filter) = target_base_filter.as_ref() {
        where_clauses.push(compile_expression_pg_sql(filter, "t", target_table));
    }

    for term in during_terms {
        let window_ms = term.window.num_milliseconds();
        let mut source_exists_terms = Vec::new();

        for source_plan in term.source_plans {
            let Some(source_table) = pg_table_for_origin(&source_plan.source_origin) else {
                continue;
            };

            let source_scope = compile_origin_scope_sql("s", &source_plan.source_origin);
            let source_filter = compile_expression_pg_sql(&source_plan.filter, "s", source_table);
            let expanded_overlap = format!(
                "t.time_range && tstzrange(lower(s.time_range) - interval '{window_ms} milliseconds', upper(s.time_range) + interval '{window_ms} milliseconds', '[]')"
            );

            source_exists_terms.push(format!(
                "EXISTS (SELECT 1 FROM {source_table} s WHERE ({source_scope}) AND ({source_filter}) AND ({expanded_overlap}))"
            ));
        }

        if source_exists_terms.is_empty() {
            return Ok(vec![]);
        }

        where_clauses.push(format!("({})", source_exists_terms.join(" OR ")));
    }

    let where_sql = where_clauses.join(" AND ");
    let sql = format!(
        "SELECT DISTINCT t.id::text AS id FROM {target_table} t WHERE {where_sql} ORDER BY lower(t.time_range) ASC LIMIT {target_limit}"
    );

    let client = pool.get().await?;
    let rows = timeout(DEFAULT_DB_QUERY_TIMEOUT, client.query(sql.as_str(), &[])).await??;

    let mut keys = Vec::new();
    let mut seen = BTreeSet::new();
    for row in rows {
        let id_str: String = row.get("id");
        if let Ok(uuid) = id_str.parse::<lifelog_core::uuid::Uuid>() {
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

fn compile_expression_pg_sql(expr: &Expression, alias: &str, table: &str) -> String {
    match expr {
        Expression::And(left, right) => format!(
            "({}) AND ({})",
            compile_expression_pg_sql(left, alias, table),
            compile_expression_pg_sql(right, alias, table)
        ),
        Expression::Or(left, right) => format!(
            "({}) OR ({})",
            compile_expression_pg_sql(left, alias, table),
            compile_expression_pg_sql(right, alias, table)
        ),
        Expression::Not(inner) => {
            format!("NOT ({})", compile_expression_pg_sql(inner, alias, table))
        }
        Expression::Eq(field, value) => {
            let field_ref = format!("{alias}.{}", sanitize_identifier(field));
            format!("{field_ref} = {}", compile_pg_value(value))
        }
        Expression::Contains(field, text) => {
            if table_supports_search_document(table) {
                format!(
                    "{alias}.search_document @@ websearch_to_tsquery('english', {})",
                    quote_string(text)
                )
            } else {
                let field_ref = format!("{alias}.{}", sanitize_identifier(field));
                format!(
                    "COALESCE({field_ref}::text, '') ILIKE '%' || {} || '%'",
                    quote_string(text)
                )
            }
        }
        Expression::TimeRange(start, end) => format!(
            "{alias}.time_range && tstzrange({}, {}, '[)')",
            quote_string(&start.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)),
            quote_string(&end.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
        ),
        Expression::Within { .. } | Expression::During { .. } | Expression::Overlaps { .. } => {
            "FALSE".to_string()
        }
    }
}

fn compile_origin_scope_sql(alias: &str, origin: &DataOrigin) -> String {
    if let Some(collector_id) = extract_collector_id(origin) {
        format!("{alias}.collector_id = {}", quote_string(collector_id))
    } else {
        "TRUE".to_string()
    }
}

fn extract_collector_id(origin: &DataOrigin) -> Option<&str> {
    match &origin.origin {
        DataOriginType::DeviceId(id) => Some(id.as_str()),
        DataOriginType::DataOrigin(parent) => extract_collector_id(parent),
    }
}

fn pg_table_for_origin(origin: &DataOrigin) -> Option<&'static str> {
    match origin.modality_name.to_lowercase().as_str() {
        "screen" => Some("screen_records"),
        "browser" => Some("browser_records"),
        "ocr" => Some("ocr_records"),
        "audio" | "microphone" => Some("audio_records"),
        "clipboard" => Some("clipboard_records"),
        "shell_history" | "shellhistory" => Some("shell_history_records"),
        "keystrokes" | "keyboard" => Some("keystroke_records"),
        _ => None,
    }
}

fn table_supports_search_document(table: &str) -> bool {
    matches!(
        table,
        "browser_records"
            | "ocr_records"
            | "clipboard_records"
            | "shell_history_records"
            | "keystroke_records"
    )
}

fn compile_pg_value(value: &Value) -> String {
    match value {
        Value::String(s) => quote_string(s),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
    }
}

fn quote_string(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

fn sanitize_identifier(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect()
}
