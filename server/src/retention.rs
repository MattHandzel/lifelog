use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, Utc};
use lifelog_core::LifelogError;
use utils::cas::FsCas;

use crate::postgres::PostgresPool;

#[derive(Debug, Default, Clone)]
pub struct RetentionRunSummary {
    pub deleted_records: u64,
    pub deleted_blobs: u64,
}

pub async fn prune_once(
    pool: &PostgresPool,
    cas: &FsCas,
    retention_policy_days: &HashMap<String, u32>,
    now: DateTime<Utc>,
) -> Result<RetentionRunSummary, LifelogError> {
    let normalized = normalize_policy_map(retention_policy_days);
    if normalized.is_empty() {
        return Ok(RetentionRunSummary::default());
    }

    let client = pool
        .get()
        .await
        .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

    let modality_rows = client
        .query("SELECT DISTINCT modality FROM frames", &[])
        .await
        .map_err(|e| LifelogError::Database(format!("retention modality query: {e}")))?;

    let mut candidate_hashes = HashSet::new();
    let mut summary = RetentionRunSummary::default();

    for row in modality_rows {
        let modality: String = row.get(0);
        let lower_modality = modality.to_lowercase();
        let ttl_days = match ttl_days_for_modality(&normalized, &lower_modality) {
            Some(days) if days > 0 => days,
            _ => continue,
        };

        let cutoff = now - Duration::days(i64::from(ttl_days));

        let count_row = client
            .query_one(
                "SELECT COUNT(*) AS count FROM frames WHERE modality = $1 AND t_canonical < $2",
                &[&modality, &cutoff],
            )
            .await
            .map_err(|e| LifelogError::Database(format!("retention count: {e}")))?;
        let stale_count: i64 = count_row.get(0);
        if stale_count == 0 {
            continue;
        }

        let blob_rows = client
            .query(
                "SELECT blob_hash FROM frames WHERE modality = $1 AND t_canonical < $2 AND blob_hash IS NOT NULL",
                &[&modality, &cutoff],
            )
            .await
            .map_err(|e| LifelogError::Database(format!("retention blob query: {e}")))?;

        for brow in blob_rows {
            let hash: String = brow.get(0);
            if !hash.is_empty() {
                candidate_hashes.insert(hash);
            }
        }

        client
            .execute(
                "DELETE FROM frames WHERE modality = $1 AND t_canonical < $2",
                &[&modality, &cutoff],
            )
            .await
            .map_err(|e| LifelogError::Database(format!("retention delete: {e}")))?;

        summary.deleted_records = summary.deleted_records.saturating_add(stale_count as u64);
    }

    for hash in candidate_hashes {
        let ref_row = client
            .query_one("SELECT COUNT(*) FROM frames WHERE blob_hash = $1", &[&hash])
            .await
            .map_err(|e| LifelogError::Database(format!("retention ref check: {e}")))?;
        let ref_count: i64 = ref_row.get(0);
        if ref_count > 0 {
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
