use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// A fully typed query that can be executed by the engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Query {
    /// The stream(s) to return records from.
    pub target: StreamSelector,
    /// The filtering condition.
    pub filter: Expression,
}

/// Selects which stream(s) are the primary subject of the query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StreamSelector {
    /// Select all streams (unlikely to be used often but good for completeness).
    All,
    /// Select a specific modality (e.g., "Screen").
    Modality(String),
    /// Select a specific stream by ID (e.g., "laptop-1:screen").
    StreamId(String),
}

/// Boolean and temporal expressions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    // --- Boolean Logic ---
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),

    // --- Leaf Predicates ---
    /// Matches if a field equals a value.
    Eq(String, Value),
    /// Matches if a field contains a string (for text search).
    Contains(String, String),
    /// Matches if the record's timestamp is within [start, end).
    TimeRange(DateTime<Utc>, DateTime<Utc>),

    // --- Cross-Stream Correlation (Section 10) ---
    /// `WITHIN(Target, Condition, Window)`: Matches records in Target if there exists
    /// a record in `stream` matching `predicate` within `±window`.
    Within {
        stream: StreamSelector,
        predicate: Box<Expression>,
        window: Duration,
    },

    /// `DURING(Target, Condition)`: Matches records in Target that temporally overlap
    /// with intervals in `stream` where `predicate` is true.
    During {
        stream: StreamSelector,
        predicate: Box<Expression>,
        /// Expansion window applied to source intervals. For point source records (no duration),
        /// this acts as a ±window around the timestamp.
        window: Duration,
    },

    /// `OVERLAPS(Target, Condition)`: Like `DURING`, but explicitly expresses interval overlap
    /// semantics for interval targets (e.g. Audio chunks).
    ///
    /// The engine treats point targets as zero-length intervals.
    Overlaps {
        stream: StreamSelector,
        predicate: Box<Expression>,
        /// Expansion window applied to source intervals. For point source records (no duration),
        /// this acts as a ±window around the timestamp.
        window: Duration,
    },
}

impl Expression {
    /// Replace any zero windows on temporal operators with the provided global default.
    ///
    /// Callers can construct queries that omit explicit windows by setting the operator window to
    /// `Duration::zero()` (LLQL supports omitting the field).
    pub fn with_default_temporal_windows(self, default_window: Duration) -> Self {
        match self {
            Expression::And(a, b) => Expression::And(
                Box::new(a.with_default_temporal_windows(default_window)),
                Box::new(b.with_default_temporal_windows(default_window)),
            ),
            Expression::Or(a, b) => Expression::Or(
                Box::new(a.with_default_temporal_windows(default_window)),
                Box::new(b.with_default_temporal_windows(default_window)),
            ),
            Expression::Not(e) => {
                Expression::Not(Box::new(e.with_default_temporal_windows(default_window)))
            }

            Expression::Within {
                stream,
                predicate,
                window,
            } => Expression::Within {
                stream,
                predicate: Box::new(predicate.with_default_temporal_windows(default_window)),
                window: if window <= Duration::zero() {
                    default_window
                } else {
                    window
                },
            },

            Expression::During {
                stream,
                predicate,
                window,
            } => Expression::During {
                stream,
                predicate: Box::new(predicate.with_default_temporal_windows(default_window)),
                window: if window <= Duration::zero() {
                    default_window
                } else {
                    window
                },
            },

            Expression::Overlaps {
                stream,
                predicate,
                window,
            } => Expression::Overlaps {
                stream,
                predicate: Box::new(predicate.with_default_temporal_windows(default_window)),
                window: if window <= Duration::zero() {
                    default_window
                } else {
                    window
                },
            },

            other => other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_temporal_windows_replaces_zero() {
        let q = Query {
            target: StreamSelector::All,
            filter: Expression::Within {
                stream: StreamSelector::Modality("Browser".to_string()),
                predicate: Box::new(Expression::Contains(
                    "url".to_string(),
                    "youtube".to_string(),
                )),
                window: Duration::zero(),
            },
        };

        let default_window = Duration::seconds(30);
        let out = q.filter.with_default_temporal_windows(default_window);
        assert!(matches!(
            out,
            Expression::Within { window, .. } if window == default_window
        ));
    }
}
