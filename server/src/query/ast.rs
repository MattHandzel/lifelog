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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}
