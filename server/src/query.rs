use chrono::{DateTime, Utc};
use lifelog_core::uuid::Uuid;
use lifelog_proto::{Query, Timerange};
use lifelog_types::{DataModality, DataOrigin, LifelogError, LifelogFrameKey};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

pub(crate) async fn execute_query(
    db: &Surreal<Client>,
    query: &Query,
    all_origins: &[DataOrigin],
) -> Result<Vec<LifelogFrameKey>, LifelogError> {
    // 1. Filter origins based on search_origins
    let search_origins: Vec<&DataOrigin> = if query.search_origins.is_empty() {
        all_origins.iter().collect()
    } else {
        all_origins
            .iter()
            .filter(|origin| {
                // Origin format: {device_id}:{modality_name} or just check if string representation matches
                // query.search_origins contains strings. DataOrigin string representation should match.
                let origin_str = format!("{}", origin);
                query.search_origins.contains(&origin_str)
            })
            .collect()
    };

    let mut all_keys = Vec::new();

    for origin in search_origins {
        let table = origin.get_table_name();

        // Skip if modality has no text fields and query has text
        let text_fields = text_search_fields(origin.modality.clone());
        if !query.text.is_empty() && text_fields.is_empty() {
            continue;
        }

        let sql = build_query_sql(&table, query, origin.modality.clone());

        // We select the ID (which is the UUID) from the table
        let mut response = db
            .query(sql)
            .await
            .map_err(|e| LifelogError::Database(e.to_string()))?;

        // Assuming the query returns objects with "uuid" field or we parse the ID
        // The query should return `SELECT VALUE id FROM ...` or similar.
        // But `id` in SurrealDB is `table:uuid`. We need to extract the UUID part.
        // Let's select `record::id(id) as uuid`.

        let uuids: Vec<String> = response
            .take("uuid")
            .map_err(|e| LifelogError::Database(e.to_string()))?;

        for uuid_str in uuids {
            if let Ok(uuid) = uuid_str.parse::<Uuid>() {
                all_keys.push(LifelogFrameKey {
                    uuid,
                    origin: origin.clone(),
                });
            }
        }
    }

    // 2. Filter results based on return_origins (post-filter)
    let final_keys = if query.return_origins.is_empty() {
        all_keys
    } else {
        all_keys
            .into_iter()
            .filter(|key| {
                let origin_str = format!("{}", key.origin);
                query.return_origins.contains(&origin_str)
            })
            .collect()
    };

    Ok(final_keys)
}

fn sanitize_surql_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

fn text_search_fields(modality: DataModality) -> Vec<&'static str> {
    match modality {
        DataModality::Browser => vec!["url", "title"],
        DataModality::Ocr => vec!["text"],
        // DataModality::Keystrokes => vec!["text", "application", "window_title"],
        // DataModality::Clipboard => vec!["text"],
        // DataModality::ShellHistory => vec!["command", "working_dir"],
        // DataModality::WindowActivity => vec!["application", "window_title"],
        // DataModality::Screen | DataModality::Audio | _ => vec![],
        _ => vec![],
    }
}

fn proto_ts_to_chrono(ts: &prost_types::Timestamp) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
}

fn build_time_clause(time_ranges: &[Timerange]) -> String {
    if time_ranges.is_empty() {
        return "true".to_string();
    }

    let clauses: Vec<String> = time_ranges
        .iter()
        .filter_map(|tr| {
            let start = tr.start.as_ref().and_then(proto_ts_to_chrono);
            let end = tr.end.as_ref().and_then(proto_ts_to_chrono);

            match (start, end) {
                (Some(s), Some(e)) => Some(format!(
                    "(timestamp >= '{}' AND timestamp <= '{}')",
                    s.to_rfc3339(),
                    e.to_rfc3339()
                )),
                (Some(s), None) => Some(format!("(timestamp >= '{}')", s.to_rfc3339())),
                (None, Some(e)) => Some(format!("(timestamp <= '{}')", e.to_rfc3339())),
                (None, None) => None,
            }
        })
        .collect();

    if clauses.is_empty() {
        "true".to_string()
    } else {
        format!("({})", clauses.join(" OR "))
    }
}

fn build_text_clause(text_terms: &[String], modality: DataModality) -> String {
    if text_terms.is_empty() {
        return "true".to_string();
    }

    let fields = text_search_fields(modality);
    if fields.is_empty() {
        return "false".to_string(); // Should be skipped earlier, but safe fallback
    }

    let term_clauses: Vec<String> = text_terms
        .iter()
        .map(|term| {
            let sanitized_term = sanitize_surql_string(term);
            // string::lowercase(field) contains string::lowercase('term')
            let field_clauses: Vec<String> = fields
                .iter()
                .map(|field| {
                    format!(
                        "string::contains(string::lowercase({}), string::lowercase('{}'))",
                        field, sanitized_term
                    )
                })
                .collect();
            format!("({})", field_clauses.join(" OR "))
        })
        .collect();

    format!("({})", term_clauses.join(" OR "))
}

fn build_query_sql(table: &str, query: &Query, modality: DataModality) -> String {
    let time_clause = build_time_clause(&query.time_ranges);
    let text_clause = build_text_clause(&query.text, modality);

    format!(
        "SELECT VALUE record::id(id) as uuid FROM `{}` WHERE {} AND {}",
        table, time_clause, text_clause
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use lifelog_types::{DataModality, DataOriginType};
    use std::str::FromStr;

    #[test]
    fn test_sanitize_surql_string() {
        assert_eq!(sanitize_surql_string("normal"), "normal");
        assert_eq!(sanitize_surql_string("with ' quote"), "with \\' quote");
        assert_eq!(
            sanitize_surql_string(r"with \ backslash"),
            r"with \\ backslash"
        );
    }

    #[test]
    fn test_text_search_fields() {
        assert_eq!(
            text_search_fields(DataModality::Browser),
            vec!["url", "title"]
        );
        assert_eq!(text_search_fields(DataModality::Screen), Vec::<&str>::new());
    }

    #[test]
    fn test_build_time_clause() {
        let ranges = vec![
            Timerange {
                start: Some(prost_types::Timestamp {
                    seconds: 1000,
                    nanos: 0,
                }),
                end: Some(prost_types::Timestamp {
                    seconds: 2000,
                    nanos: 0,
                }),
            },
            Timerange {
                start: Some(prost_types::Timestamp {
                    seconds: 3000,
                    nanos: 0,
                }),
                end: None,
            },
        ];
        let clause = build_time_clause(&ranges);
        assert!(clause.contains("timestamp >= '1970-01-01T00:16:40+00:00'"));
        assert!(clause.contains("timestamp <= '1970-01-01T00:33:20+00:00'"));
        assert!(clause.contains("OR"));
    }

    #[test]
    fn test_build_text_clause() {
        let terms = vec!["hello".to_string(), "world".to_string()];
        let clause = build_text_clause(&terms, DataModality::Browser);
        // Should contain checks for url and title
        assert!(
            clause.contains("string::contains(string::lowercase(url), string::lowercase('hello'))")
        );
        assert!(clause
            .contains("string::contains(string::lowercase(title), string::lowercase('hello'))"));
        assert!(
            clause.contains("string::contains(string::lowercase(url), string::lowercase('world'))")
        );
    }

    #[test]
    fn test_build_query_sql() {
        let query = Query {
            search_origins: vec![],
            return_origins: vec![],
            time_ranges: vec![],
            image_embedding: None,
            text_embedding: None,
            text: vec!["foo".to_string()],
        };
        let sql = build_query_sql("test_table", &query, DataModality::Ocr);
        assert!(sql.contains("SELECT VALUE record::id(id) as uuid FROM `test_table`"));
        assert!(sql.contains("WHERE true AND")); // time clause is true
        assert!(sql.contains("string::contains(string::lowercase(text), string::lowercase('foo'))"));
    }
}
