use super::ast::*;
use lifelog_core::DataOrigin;
use std::collections::VecDeque;

#[derive(Debug, PartialEq)]
pub enum ExecutionPlan {
    /// A query targeted at a specific table/origin.
    TableQuery {
        table: String,
        origin: DataOrigin,
        sql: String,
    },
    /// Multiple queries to be executed.
    MultiQuery(Vec<ExecutionPlan>),
    /// Two-stage temporal join for `DURING(...)` / `OVERLAPS(...)` / `WITHIN(...)`.
    ///
    /// Phase 1: query the `source_*` tables for candidate intervals.
    /// Phase 2: query the target table for UUIDs whose timestamps fall within any interval,
    /// in addition to the target base predicate.
    DuringQuery {
        target_table: String,
        target_origin: DataOrigin,
        target_base_where: String,
        during_terms: Vec<DuringTermPlan>,
        max_source_intervals: usize,
        max_time_clauses: usize,
    },
    /// Placeholder for multi-stage plans.
    #[allow(dead_code)]
    Unsupported(String),
}

#[derive(Debug, PartialEq)]
pub struct DuringSourcePlan {
    pub source_table: String,
    pub source_origin: DataOrigin,
    pub sql: String,
}

#[derive(Debug, PartialEq)]
pub struct DuringTermPlan {
    pub source_plans: Vec<DuringSourcePlan>,
    pub window: chrono::Duration,
}

pub struct Planner;

impl Planner {
    pub fn plan(query: &Query, available_origins: &[DataOrigin]) -> ExecutionPlan {
        let origins = Self::resolve_selector(&query.target, available_origins);

        if origins.is_empty() {
            return ExecutionPlan::MultiQuery(vec![]);
        }

        const DEFAULT_MAX_SOURCE_INTERVALS: usize = 200;
        const DEFAULT_MAX_TIME_CLAUSES: usize = 50;
        const DEFAULT_MAX_TARGET_UUIDS: usize = 1_000;

        let plans = origins
            .into_iter()
            .map(|origin| {
                let table = origin.get_table_name();
                match Self::compile_conjunctive(&query.filter) {
                    Ok((sql_terms, temporal_terms)) => {
                        let target_base_where = if sql_terms.is_empty() {
                            "true".to_string()
                        } else {
                            sql_terms.join(" AND ")
                        };

                        if temporal_terms.is_empty() {
                            let sql = format!(
                                "SELECT uuid FROM `{}` WHERE {} LIMIT {};",
                                table, target_base_where, DEFAULT_MAX_TARGET_UUIDS
                            );
                            return ExecutionPlan::TableQuery { table, origin, sql };
                        };

                        // Temporal terms: build one interval set per term, then intersect at execution time.
                        //
                        // Note: `WITHIN(...)` is treated as a point-interval term; if the source stream is
                        // interval-valued (has `t_end`), the whole interval is expanded by `window` and used.
                        let mut during_terms = Vec::new();
                        for t in temporal_terms {
                            let term = match t {
                                TemporalTerm::Within(d)
                                | TemporalTerm::During(d)
                                | TemporalTerm::Overlaps(d) => d,
                            };

                            let source_origins =
                                Self::resolve_selector(&term.stream, available_origins);
                            if source_origins.is_empty() {
                                let sql = format!("SELECT uuid FROM `{}` WHERE false;", table);
                                return ExecutionPlan::TableQuery { table, origin, sql };
                            }

                            if Self::contains_temporal_ops(&term.predicate) {
                                return ExecutionPlan::Unsupported(
                                    "Nested temporal operators inside temporal predicates are not supported yet"
                                        .to_string(),
                                );
                            }

                            let source_where = Self::compile_expression_sql(&term.predicate);
                            let source_plans = source_origins
                                .into_iter()
                                .map(|source_origin| {
                                    let source_table = source_origin.get_table_name();
                                    let sql = format!(
                                        "SELECT t_canonical, t_end FROM `{}` WHERE {} ORDER BY t_canonical DESC LIMIT {};",
                                        source_table, source_where, DEFAULT_MAX_SOURCE_INTERVALS
                                    );
                                    DuringSourcePlan {
                                        source_table,
                                        source_origin,
                                        sql,
                                    }
                                })
                                .collect();

                            during_terms.push(DuringTermPlan {
                                source_plans,
                                window: term.window,
                            });
                        }

                        ExecutionPlan::DuringQuery {
                            target_table: table,
                            target_origin: origin,
                            target_base_where,
                            during_terms,
                            max_source_intervals: DEFAULT_MAX_SOURCE_INTERVALS,
                            max_time_clauses: DEFAULT_MAX_TIME_CLAUSES,
                        }
                    }
                    Err(msg) => ExecutionPlan::Unsupported(msg),
                }
            })
            .collect();

        ExecutionPlan::MultiQuery(plans)
    }

    /// Compiles an expression that contains *no* temporal join operators (`WITHIN`, `DURING`) to SQL.
    pub fn compile_expression_sql(expr: &Expression) -> String {
        match expr {
            Expression::And(left, right) => {
                format!(
                    "({}) AND ({})",
                    Self::compile_expression_sql(left),
                    Self::compile_expression_sql(right)
                )
            }
            Expression::Or(left, right) => {
                format!(
                    "({}) OR ({})",
                    Self::compile_expression_sql(left),
                    Self::compile_expression_sql(right)
                )
            }
            Expression::Not(inner) => {
                format!("!({})", Self::compile_expression_sql(inner))
            }
            Expression::Eq(field, value) => {
                format!("{} = {}", field, Self::compile_value(value))
            }
            Expression::Contains(field, text) => {
                // Use @@ for BM25 full-text search on indexed fields
                format!("{} @@ {}", field, Self::quote_string(text))
            }
            Expression::TimeRange(start, end) => {
                // SurrealDB datetime format
                format!(
                    "t_canonical >= d'{}' AND t_canonical < d'{}'",
                    start.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
                    end.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
                )
            }
            Expression::Within { .. } => "false /* WITHIN handled at plan level */".to_string(),
            Expression::During { .. } => "false /* DURING handled at plan level */".to_string(),
            Expression::Overlaps { .. } => "false /* OVERLAPS handled at plan level */".to_string(),
        }
    }

    /// Attempts to decompose `expr` into a conjunction of SQL-compilable terms plus
    /// one or more top-level temporal join terms (`WITHIN(...)` and/or `DURING(...)`).
    ///
    /// Current limitation (intentional): `WITHIN` cannot appear under `OR` / `NOT`.
    fn compile_conjunctive(expr: &Expression) -> Result<(Vec<String>, Vec<TemporalTerm>), String> {
        let mut sql_terms: Vec<String> = Vec::new();
        let mut temporal_terms: Vec<TemporalTerm> = Vec::new();

        // Flatten nested ANDs iteratively to keep stack shallow.
        let mut queue: VecDeque<&Expression> = VecDeque::new();
        queue.push_back(expr);

        while let Some(node) = queue.pop_front() {
            match node {
                Expression::And(l, r) => {
                    queue.push_back(l);
                    queue.push_back(r);
                }
                Expression::Within {
                    stream,
                    predicate,
                    window,
                } => temporal_terms.push(TemporalTerm::Within(DuringTerm {
                    stream: stream.clone(),
                    predicate: (**predicate).clone(),
                    window: *window,
                })),
                Expression::During {
                    stream,
                    predicate,
                    window,
                } => {
                    temporal_terms.push(TemporalTerm::During(DuringTerm {
                        stream: stream.clone(),
                        predicate: (**predicate).clone(),
                        window: *window,
                    }));
                }
                Expression::Overlaps {
                    stream,
                    predicate,
                    window,
                } => {
                    temporal_terms.push(TemporalTerm::Overlaps(DuringTerm {
                        stream: stream.clone(),
                        predicate: (**predicate).clone(),
                        window: *window,
                    }));
                }
                Expression::Or(..) | Expression::Not(..) => {
                    if Self::contains_temporal_ops(node) {
                        return Err(
                            "Temporal joins (WITHIN/DURING/OVERLAPS) are only supported under conjunctions (AND), not OR/NOT"
                                .to_string(),
                        );
                    }
                    sql_terms.push(Self::compile_expression_sql(node));
                }
                _ => sql_terms.push(Self::compile_expression_sql(node)),
            }
        }

        Ok((sql_terms, temporal_terms))
    }

    fn contains_temporal_ops(expr: &Expression) -> bool {
        match expr {
            Expression::Within { .. } | Expression::During { .. } | Expression::Overlaps { .. } => {
                true
            }
            Expression::And(l, r) | Expression::Or(l, r) => {
                Self::contains_temporal_ops(l) || Self::contains_temporal_ops(r)
            }
            Expression::Not(inner) => Self::contains_temporal_ops(inner),
            _ => false,
        }
    }

    fn resolve_selector(
        selector: &StreamSelector,
        available_origins: &[DataOrigin],
    ) -> Vec<DataOrigin> {
        match selector {
            StreamSelector::All => available_origins.to_vec(),
            StreamSelector::Modality(m) => available_origins
                .iter()
                .filter(|o| o.modality_name == *m)
                .cloned()
                .collect(),
            StreamSelector::StreamId(id) => {
                // Try to match exactly by table name first.
                if let Some(origin) = available_origins.iter().find(|o| o.get_table_name() == *id) {
                    vec![origin.clone()]
                } else {
                    // Fallback: search for origins that match this as a suffix or exact modality.
                    let matches: Vec<_> = available_origins
                        .iter()
                        .filter(|o| {
                            o.get_table_name() == *id
                                || o.modality_name == *id
                                || o.get_table_name().ends_with(&format!(":{}", id))
                        })
                        .cloned()
                        .collect();

                    if matches.is_empty() {
                        // Final fallback: try to parse it.
                        if let Ok(origin) = lifelog_core::DataOrigin::tryfrom_string(id.clone()) {
                            vec![origin]
                        } else {
                            vec![]
                        }
                    } else {
                        matches
                    }
                }
            }
        }
    }

    fn compile_value(val: &Value) -> String {
        match val {
            Value::String(s) => Self::quote_string(s),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
        }
    }

    fn quote_string(s: &str) -> String {
        format!("'{}'", s.replace('\'', "\\'"))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct DuringTerm {
    stream: StreamSelector,
    predicate: Expression,
    window: chrono::Duration,
}

#[derive(Debug, Clone, PartialEq)]
enum TemporalTerm {
    Within(DuringTerm),
    During(DuringTerm),
    Overlaps(DuringTerm),
}

#[cfg(test)]
mod tests {
    use super::*;
    use lifelog_core::{DataOrigin, DataOriginType};

    #[test]
    #[allow(clippy::panic)]
    fn test_planner_origin_resolution() {
        let origins = vec![
            DataOrigin::new(DataOriginType::DeviceId("laptop".into()), "Screen".into()),
            DataOrigin::new(DataOriginType::DeviceId("phone".into()), "Browser".into()),
            DataOrigin::new(DataOriginType::DeviceId("laptop".into()), "Browser".into()),
        ];

        // 1. Exact match by table name
        let query = Query {
            target: StreamSelector::StreamId("laptop:Screen".into()),
            filter: Expression::Eq("a".into(), Value::Int(1)),
        };
        let plan = Planner::plan(&query, &origins);
        if let ExecutionPlan::MultiQuery(plans) = plan {
            assert_eq!(plans.len(), 1);
            if let ExecutionPlan::TableQuery { table, .. } = &plans[0] {
                assert_eq!(table, "laptop:Screen");
            } else {
                panic!("Expected TableQuery");
            }
        } else {
            panic!("Expected MultiQuery");
        }

        // 2. Match by suffix (e.g. just "Browser" -> matches both browsers)
        let query = Query {
            target: StreamSelector::StreamId("Browser".into()),
            filter: Expression::Eq("a".into(), Value::Int(1)),
        };
        let plan = Planner::plan(&query, &origins);
        if let ExecutionPlan::MultiQuery(plans) = plan {
            assert_eq!(plans.len(), 2);
            // Verify both laptop:Browser and phone:Browser are included
            let tables: Vec<_> = plans
                .iter()
                .map(|p| match p {
                    ExecutionPlan::TableQuery { table, .. } => table.clone(),
                    _ => "".to_string(),
                })
                .collect();
            assert!(tables.contains(&"laptop:Browser".to_string()));
            assert!(tables.contains(&"phone:Browser".to_string()));
        } else {
            panic!("Expected MultiQuery");
        }

        // 3. Fallback to direct parsing if no match found
        let query = Query {
            target: StreamSelector::StreamId("unknown:device".into()),
            filter: Expression::Eq("a".into(), Value::Int(1)),
        };
        let plan = Planner::plan(&query, &origins);
        if let ExecutionPlan::MultiQuery(plans) = plan {
            assert_eq!(plans.len(), 1);
            if let ExecutionPlan::TableQuery { table, .. } = &plans[0] {
                assert_eq!(table, "unknown:device");
            } else {
                panic!("Expected TableQuery");
            }
        }
    }
}
