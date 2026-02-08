use super::ast::*;
use lifelog_core::DataOrigin;

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
    /// Placeholder for multi-stage plans.
    #[allow(dead_code)]
    Unsupported(String),
}

pub struct Planner;

impl Planner {
    pub fn plan(query: &Query, available_origins: &[DataOrigin]) -> ExecutionPlan {
        let origins = match &query.target {
            StreamSelector::All => available_origins.to_vec(),
            StreamSelector::Modality(m) => available_origins
                .iter()
                .filter(|o| o.modality_name == *m)
                .cloned()
                .collect(),
            StreamSelector::StreamId(id) => {
                // Try to match exactly by table name first
                if let Some(origin) = available_origins.iter().find(|o| o.get_table_name() == *id) {
                    vec![origin.clone()]
                } else {
                    // Fallback: search for origins that match this as a suffix or exact modality
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
                        // Final fallback: try to parse it
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
        };

        if origins.is_empty() {
            return ExecutionPlan::MultiQuery(vec![]);
        }

        let plans = origins
            .into_iter()
            .map(|origin| {
                let table = origin.get_table_name();
                let where_clause = Self::compile_expression(&query.filter);
                let sql = format!("SELECT uuid FROM `{}` WHERE {};", table, where_clause);
                ExecutionPlan::TableQuery { table, origin, sql }
            })
            .collect();

        ExecutionPlan::MultiQuery(plans)
    }

    pub fn compile_expression(expr: &Expression) -> String {
        match expr {
            Expression::And(left, right) => {
                format!(
                    "({}) AND ({})",
                    Self::compile_expression(left),
                    Self::compile_expression(right)
                )
            }
            Expression::Or(left, right) => {
                format!(
                    "({}) OR ({})",
                    Self::compile_expression(left),
                    Self::compile_expression(right)
                )
            }
            Expression::Not(inner) => {
                format!("!({})", Self::compile_expression(inner))
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
                    "timestamp >= '{}' AND timestamp < '{}'",
                    start.to_rfc3339(),
                    end.to_rfc3339()
                )
            }
            Expression::Within { .. } => {
                "false /* WITHIN operator requires multi-stage planning */".to_string()
            }
            Expression::During { .. } => {
                "false /* DURING operator requires multi-stage planning */".to_string()
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
