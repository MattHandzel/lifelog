use super::ast::*;

#[derive(Debug, PartialEq)]
pub enum ExecutionPlan {
    /// A single SurrealQL query string.
    SimpleQuery(String),
    /// Placeholder for multi-stage plans.
    Unsupported(String),
}

pub struct Planner;

impl Planner {
    pub fn plan(query: &Query) -> ExecutionPlan {
        let table = match &query.target {
            StreamSelector::Modality(m) => m.clone(),
            StreamSelector::StreamId(id) => {
                // Assuming StreamId implies table name directly or needs lookup.
                // For now treat as table name.
                id.clone()
            }
            StreamSelector::All => {
                return ExecutionPlan::Unsupported("Querying ALL not supported".into())
            }
        };

        let where_clause = Self::compile_expression(&query.filter);

        let sql = format!("SELECT uuid FROM `{}` WHERE {};", table, where_clause);
        ExecutionPlan::SimpleQuery(sql)
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
                // Using '~' operator for case-insensitive substring matching
                format!("{} ~ {}", field, Self::quote_string(text))
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
