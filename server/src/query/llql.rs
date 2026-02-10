use super::ast;
use chrono::{DateTime, Utc};
use lifelog_core::LifelogError;

/// Minimal “LLQL” (lifelog query language) parsing.
///
/// Current goal: allow the UI (via existing `Query.text`) to submit a fully-typed
/// cross-modal query (WITHIN/DURING) without changing the protobuf surface.
///
/// Usage: set `Query.text = ["llql:{...json...}"]` or `["llql-json:{...json...}"]`.
pub fn try_parse_llql(text_terms: &[String]) -> Result<Option<ast::Query>, LifelogError> {
    let Some(first) = text_terms.first() else {
        return Ok(None);
    };

    let s = first.trim_start();
    let json = if let Some(rest) = s.strip_prefix("llql-json:") {
        rest
    } else if let Some(rest) = s.strip_prefix("llql:") {
        rest
    } else {
        return Ok(None);
    };

    let llql: LlqlQuery =
        serde_json::from_str(json.trim()).map_err(|e| LifelogError::Validation {
            field: "query.text".to_string(),
            reason: format!("failed to parse llql json: {e}"),
        })?;

    llql.try_into_ast()
        .map(Some)
        .map_err(|reason| LifelogError::Validation {
            field: "query.text".to_string(),
            reason,
        })
}

#[derive(Debug, serde::Deserialize)]
struct LlqlQuery {
    target: LlqlSelector,
    filter: LlqlExpr,
}

impl LlqlQuery {
    fn try_into_ast(self) -> Result<ast::Query, String> {
        Ok(ast::Query {
            target: self.target.into_ast(),
            filter: self.filter.try_into_ast()?,
        })
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LlqlSelector {
    All,
    Modality { modality: String },
    StreamId { stream_id: String },
}

impl LlqlSelector {
    fn into_ast(self) -> ast::StreamSelector {
        match self {
            LlqlSelector::All => ast::StreamSelector::All,
            LlqlSelector::Modality { modality } => ast::StreamSelector::Modality(modality),
            LlqlSelector::StreamId { stream_id } => ast::StreamSelector::StreamId(stream_id),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum LlqlExpr {
    And {
        terms: Vec<LlqlExpr>,
    },
    Or {
        terms: Vec<LlqlExpr>,
    },
    Not {
        term: Box<LlqlExpr>,
    },

    Eq {
        field: String,
        value: LlqlValue,
    },
    Contains {
        field: String,
        text: String,
    },
    TimeRange {
        start: String,
        end: String,
    },

    Within {
        stream: LlqlSelector,
        predicate: Box<LlqlExpr>,
        window: String,
    },
    During {
        stream: LlqlSelector,
        predicate: Box<LlqlExpr>,
        window: String,
    },
    Overlaps {
        stream: LlqlSelector,
        predicate: Box<LlqlExpr>,
        window: String,
    },
}

impl LlqlExpr {
    fn try_into_ast(self) -> Result<ast::Expression, String> {
        match self {
            LlqlExpr::And { terms } => {
                if terms.is_empty() {
                    return Ok(ast::Expression::TimeRange(
                        chrono::DateTime::<Utc>::MIN_UTC,
                        chrono::DateTime::<Utc>::MAX_UTC,
                    ));
                }
                let mut iter = terms.into_iter();
                let mut acc = iter
                    .next()
                    .ok_or_else(|| "AND terms cannot be empty".to_string())?
                    .try_into_ast()?;
                for t in iter {
                    acc = ast::Expression::And(Box::new(acc), Box::new(t.try_into_ast()?));
                }
                Ok(acc)
            }
            LlqlExpr::Or { terms } => {
                if terms.is_empty() {
                    return Ok(ast::Expression::TimeRange(
                        chrono::DateTime::<Utc>::MIN_UTC,
                        chrono::DateTime::<Utc>::MAX_UTC,
                    ));
                }
                let mut iter = terms.into_iter();
                let mut acc = iter
                    .next()
                    .ok_or_else(|| "OR terms cannot be empty".to_string())?
                    .try_into_ast()?;
                for t in iter {
                    acc = ast::Expression::Or(Box::new(acc), Box::new(t.try_into_ast()?));
                }
                Ok(acc)
            }
            LlqlExpr::Not { term } => Ok(ast::Expression::Not(Box::new(term.try_into_ast()?))),
            LlqlExpr::Eq { field, value } => Ok(ast::Expression::Eq(field, value.into_ast())),
            LlqlExpr::Contains { field, text } => Ok(ast::Expression::Contains(field, text)),
            LlqlExpr::TimeRange { start, end } => {
                let start = parse_rfc3339_utc(&start)?;
                let end = parse_rfc3339_utc(&end)?;
                Ok(ast::Expression::TimeRange(start, end))
            }
            LlqlExpr::Within {
                stream,
                predicate,
                window,
            } => Ok(ast::Expression::Within {
                stream: stream.into_ast(),
                predicate: Box::new(predicate.try_into_ast()?),
                window: parse_duration(&window)?,
            }),
            LlqlExpr::During {
                stream,
                predicate,
                window,
            } => Ok(ast::Expression::During {
                stream: stream.into_ast(),
                predicate: Box::new(predicate.try_into_ast()?),
                window: parse_duration(&window)?,
            }),
            LlqlExpr::Overlaps {
                stream,
                predicate,
                window,
            } => Ok(ast::Expression::Overlaps {
                stream: stream.into_ast(),
                predicate: Box::new(predicate.try_into_ast()?),
                window: parse_duration(&window)?,
            }),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
enum LlqlValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl LlqlValue {
    fn into_ast(self) -> ast::Value {
        match self {
            LlqlValue::String(s) => ast::Value::String(s),
            LlqlValue::Int(i) => ast::Value::Int(i),
            LlqlValue::Float(f) => ast::Value::Float(f),
            LlqlValue::Bool(b) => ast::Value::Bool(b),
        }
    }
}

fn parse_rfc3339_utc(s: &str) -> Result<DateTime<Utc>, String> {
    let dt = DateTime::parse_from_rfc3339(s.trim())
        .map_err(|e| format!("invalid rfc3339 datetime '{s}': {e}"))?;
    Ok(dt.with_timezone(&Utc))
}

fn parse_duration(s: &str) -> Result<chrono::Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("duration cannot be empty".to_string());
    }

    // Support a small, explicit set of suffixes to keep this unambiguous.
    // Examples: "250ms", "30s", "5m", "1h"
    let (num, unit) = if let Some(v) = s.strip_suffix("ms") {
        (v, "ms")
    } else if let Some(v) = s.strip_suffix('s') {
        (v, "s")
    } else if let Some(v) = s.strip_suffix('m') {
        (v, "m")
    } else if let Some(v) = s.strip_suffix('h') {
        (v, "h")
    } else {
        return Err(format!("invalid duration '{s}': expected suffix ms|s|m|h"));
    };

    let n: i64 = num.trim().parse().map_err(|e| {
        format!(
            "invalid duration '{s}': failed to parse integer '{}': {e}",
            num.trim()
        )
    })?;

    if n < 0 {
        return Err(format!("invalid duration '{s}': must be non-negative"));
    }

    match unit {
        "ms" => Ok(chrono::Duration::milliseconds(n)),
        "s" => Ok(chrono::Duration::seconds(n)),
        "m" => Ok(chrono::Duration::minutes(n)),
        "h" => Ok(chrono::Duration::hours(n)),
        _ => Err(format!("invalid duration unit in '{s}'")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn parses_duration_units() {
        assert!(matches!(
            parse_duration("250ms"),
            Ok(d) if d == Duration::milliseconds(250)
        ));
        assert!(matches!(
            parse_duration("30s"),
            Ok(d) if d == Duration::seconds(30)
        ));
        assert!(matches!(
            parse_duration("5m"),
            Ok(d) if d == Duration::minutes(5)
        ));
        assert!(matches!(
            parse_duration("1h"),
            Ok(d) if d == Duration::hours(1)
        ));
        assert!(parse_duration("10").is_err());
    }

    #[test]
    fn parses_llql_json_canonical_example_shape() {
        let input = r#"llql:{
          "target": {"type":"modality","modality":"Audio"},
          "filter": {
            "op":"and",
            "terms":[
              {
                "op":"during",
                "stream": {"type":"modality","modality":"Browser"},
                "predicate": {"op":"contains","field":"url","text":"youtube"},
                "window":"30s"
              },
              {
                "op":"during",
                "stream": {"type":"modality","modality":"Ocr"},
                "predicate": {"op":"contains","field":"text","text":"3Blue1Brown"},
                "window":"30s"
              }
            ]
          }
        }"#;

        let parsed = try_parse_llql(&[input.to_string()]);
        assert!(matches!(parsed, Ok(Some(_))));
        let q = if let Ok(Some(q)) = parsed { q } else { return };

        assert_eq!(q.target, ast::StreamSelector::Modality("Audio".to_string()));
        assert!(matches!(q.filter, ast::Expression::And(_, _)));
    }
}
