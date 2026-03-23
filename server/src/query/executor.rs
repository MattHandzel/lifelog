use super::ast::{Expression, Value};
use super::planner::ExecutionPlan;
use crate::postgres::PostgresPool;
use anyhow::anyhow;
use lifelog_core::{DataOrigin, DataOriginType, LifelogFrameKey};
use std::collections::BTreeSet;
use std::time::Duration;
use tokio::time::timeout;

const DEFAULT_DB_QUERY_TIMEOUT: Duration = Duration::from_secs(10);

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
        } => execute_table_query(pool, origin, filter, limit).await,
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
            execute_during_query(
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

async fn execute_table_query(
    pool: &PostgresPool,
    origin: DataOrigin,
    filter: Option<Expression>,
    limit: usize,
) -> Result<Vec<LifelogFrameKey>, anyhow::Error> {
    if limit == 0 {
        return Ok(vec![]);
    }

    let origin_scope = compile_origin_scope_sql("t", &origin);
    let filter_sql = filter
        .as_ref()
        .map(|f| compile_expression_pg_sql(f, "t"))
        .unwrap_or_else(|| "TRUE".to_string());

    let sql = format!(
        "SELECT t.id::text AS id FROM frames t WHERE ({origin_scope}) AND ({filter_sql}) ORDER BY t.t_canonical ASC LIMIT {limit}"
    );
    tracing::debug!("Postgres TableQuery SQL: {}", sql);

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

async fn execute_during_query(
    pool: &PostgresPool,
    target_origin: DataOrigin,
    target_base_filter: Option<Expression>,
    during_terms: Vec<super::planner::DuringTermPlan>,
    target_limit: usize,
) -> Result<Vec<LifelogFrameKey>, anyhow::Error> {
    if target_limit == 0 {
        return Ok(vec![]);
    }

    let mut where_clauses = Vec::new();
    where_clauses.push(compile_origin_scope_sql("t", &target_origin));
    if let Some(filter) = target_base_filter.as_ref() {
        where_clauses.push(compile_expression_pg_sql(filter, "t"));
    }

    for term in during_terms {
        let window_ms = term.window.num_milliseconds();
        let mut source_exists_terms = Vec::new();

        for source_plan in term.source_plans {
            let source_scope = compile_origin_scope_sql("s", &source_plan.source_origin);
            let source_filter = compile_expression_pg_sql(&source_plan.filter, "s");
            let expanded_overlap = format!(
                "t.time_range && tstzrange(lower(s.time_range) - interval '{window_ms} milliseconds', upper(s.time_range) + interval '{window_ms} milliseconds', '[]')"
            );

            source_exists_terms.push(format!(
                "EXISTS (SELECT 1 FROM frames s WHERE ({source_scope}) AND ({source_filter}) AND ({expanded_overlap}))"
            ));
        }

        if source_exists_terms.is_empty() {
            return Ok(vec![]);
        }

        where_clauses.push(format!("({})", source_exists_terms.join(" OR ")));
    }

    let where_sql = where_clauses.join(" AND ");
    let sql = format!(
        "SELECT DISTINCT t.id::text AS id FROM frames t WHERE {where_sql} ORDER BY t.t_canonical ASC LIMIT {target_limit}"
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

fn compile_expression_pg_sql(expr: &Expression, alias: &str) -> String {
    match expr {
        Expression::And(left, right) => format!(
            "({}) AND ({})",
            compile_expression_pg_sql(left, alias),
            compile_expression_pg_sql(right, alias)
        ),
        Expression::Or(left, right) => format!(
            "({}) OR ({})",
            compile_expression_pg_sql(left, alias),
            compile_expression_pg_sql(right, alias)
        ),
        Expression::Not(inner) => {
            format!("NOT ({})", compile_expression_pg_sql(inner, alias))
        }
        Expression::Eq(field, value) => {
            let field_ref = format!("{alias}.payload->>'{}'", sanitize_identifier(field));
            format!("{field_ref} = {}", compile_pg_value(value))
        }
        Expression::Contains(_field, text) => {
            format!(
                "{alias}.search_doc @@ websearch_to_tsquery('english', {})",
                quote_string(text)
            )
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
    let modality = &origin.modality_name;
    let modality_clause = format!("{alias}.modality = {}", quote_string(modality));

    if let Some(collector_id) = extract_collector_id(origin) {
        format!(
            "{modality_clause} AND {alias}.collector_id = {}",
            quote_string(collector_id)
        )
    } else {
        modality_clause
    }
}

fn extract_collector_id(origin: &DataOrigin) -> Option<&str> {
    match &origin.origin {
        DataOriginType::DeviceId(id) => Some(id.as_str()),
        DataOriginType::DataOrigin(parent) => extract_collector_id(parent),
    }
}

fn compile_pg_value(value: &Value) -> String {
    match value {
        Value::String(s) => quote_string(s),
        Value::Int(i) => quote_string(&i.to_string()),
        Value::Float(f) => quote_string(&f.to_string()),
        Value::Bool(b) => quote_string(&b.to_string()),
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
