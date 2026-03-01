use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, Utc};
use lifelog_core::LifelogError;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use utils::cas::FsCas;

use crate::db::get_origins_from_db;

#[derive(Debug, Default, Clone)]
pub struct RetentionRunSummary {
    pub deleted_records: u64,
    pub deleted_blobs: u64,
}

#[derive(serde::Deserialize)]
struct CountRow {
    count: u64,
}

#[derive(serde::Deserialize)]
struct BlobRow {
    blob_hash: Option<String>,
}

#[derive(serde::Deserialize)]
struct RefRow {
    id: Option<serde_json::Value>,
}

pub async fn prune_once(
    db: &Surreal<Client>,
    cas: &FsCas,
    retention_policy_days: &HashMap<String, u32>,
    now: DateTime<Utc>,
) -> Result<RetentionRunSummary, LifelogError> {
    let normalized = normalize_policy_map(retention_policy_days);
    if normalized.is_empty() {
        return Ok(RetentionRunSummary::default());
    }

    let origins = get_origins_from_db(db).await?;
    let mut candidate_hashes = HashSet::new();
    let mut summary = RetentionRunSummary::default();

    for origin in origins {
        let modality = origin.modality_name.to_lowercase();
        let ttl_days = match ttl_days_for_modality(&normalized, &modality) {
            Some(days) if days > 0 => days,
            _ => continue,
        };

        let cutoff = now - Duration::days(i64::from(ttl_days));
        let table = origin.get_table_name();

        let mut count_resp = db
            .query(format!(
                "SELECT count() AS count FROM `{}` WHERE ((t_canonical != NONE AND t_canonical < $cutoff) OR (t_canonical = NONE AND timestamp < $cutoff));",
                table
            ))
            .bind(("cutoff", cutoff))
            .await
            .map_err(|e| LifelogError::Database(e.to_string()))?;
        let counts: Vec<CountRow> = count_resp
            .take(0)
            .map_err(|e| LifelogError::Database(e.to_string()))?;
        let stale_count = counts.first().map(|r| r.count).unwrap_or(0);
        if stale_count == 0 {
            continue;
        }

        let mut blobs_resp = db
            .query(format!(
                "SELECT blob_hash FROM `{}` WHERE ((t_canonical != NONE AND t_canonical < $cutoff) OR (t_canonical = NONE AND timestamp < $cutoff)) AND blob_hash != NONE AND blob_hash != '';",
                table
            ))
            .bind(("cutoff", cutoff))
            .await
            .map_err(|e| LifelogError::Database(e.to_string()))?;
        let blob_rows: Vec<BlobRow> = blobs_resp
            .take(0)
            .map_err(|e| LifelogError::Database(e.to_string()))?;
        for row in blob_rows {
            if let Some(hash) = row.blob_hash {
                if !hash.is_empty() {
                    candidate_hashes.insert(hash);
                }
            }
        }

        db.query(format!(
            "DELETE `{}` WHERE ((t_canonical != NONE AND t_canonical < $cutoff) OR (t_canonical = NONE AND timestamp < $cutoff));",
            table
        ))
        .bind(("cutoff", cutoff))
        .await
        .map_err(|e| LifelogError::Database(e.to_string()))?;

        summary.deleted_records = summary.deleted_records.saturating_add(stale_count);
    }

    for hash in candidate_hashes {
        if has_blob_references(db, &hash).await? {
            continue;
        }
        match cas.remove(&hash) {
            Ok(()) => {
                summary.deleted_blobs = summary.deleted_blobs.saturating_add(1);
            }
            Err(e) => {
                tracing::warn!(hash = %hash, error = %e, "failed to remove orphan CAS blob");
            }
        }
    }

    Ok(summary)
}

async fn has_blob_references(db: &Surreal<Client>, hash: &str) -> Result<bool, LifelogError> {
    let origins = get_origins_from_db(db).await?;
    for origin in origins {
        let table = origin.get_table_name();
        let mut resp = db
            .query(format!(
                "SELECT id FROM `{}` WHERE blob_hash = $hash LIMIT 1;",
                table
            ))
            .bind(("hash", hash.to_string()))
            .await
            .map_err(|e| LifelogError::Database(e.to_string()))?;
        let refs: Vec<RefRow> = resp
            .take(0)
            .map_err(|e| LifelogError::Database(e.to_string()))?;
        if refs.iter().any(|r| r.id.is_some()) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn normalize_policy_map(raw: &HashMap<String, u32>) -> HashMap<String, u32> {
    raw.iter()
        .map(|(k, v)| (k.trim().to_lowercase(), *v))
        .collect()
}

fn ttl_days_for_modality(policy: &HashMap<String, u32>, modality: &str) -> Option<u32> {
    if let Some(v) = policy.get(modality) {
        return Some(*v);
    }
    if is_text_modality(modality) {
        if let Some(v) = policy.get("text") {
            return Some(*v);
        }
    }
    policy.get("all").copied()
}

fn is_text_modality(modality: &str) -> bool {
    matches!(
        modality,
        "browser" | "ocr" | "clipboard" | "shellhistory" | "keystrokes" | "windowactivity"
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::ttl_days_for_modality;

    #[test]
    fn resolves_direct_and_text_bucket() {
        let mut policy = HashMap::new();
        policy.insert("screen".to_string(), 7);
        policy.insert("text".to_string(), 30);

        assert_eq!(ttl_days_for_modality(&policy, "screen"), Some(7));
        assert_eq!(ttl_days_for_modality(&policy, "ocr"), Some(30));
        assert_eq!(ttl_days_for_modality(&policy, "audio"), None);
    }
}
