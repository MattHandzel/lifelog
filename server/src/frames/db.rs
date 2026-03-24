use chrono::{DateTime, Utc};

use super::to_lifelog_data;
use super::FrameRow;
use crate::postgres::PostgresPool;
use lifelog_core::{DataOrigin, DataOriginType, LifelogError, LifelogFrameKey};
use utils::cas::FsCas;

fn row_to_frame_row(row: &tokio_postgres::Row) -> Result<FrameRow, LifelogError> {
    let t_canonical: DateTime<Utc> = row.get("t_canonical");
    Ok(FrameRow {
        id: row.get("id"),
        collector_id: row.get("collector_id"),
        stream_id: row.get("stream_id"),
        modality: row.get("modality"),
        t_device: row.get("t_device"),
        t_ingest: row.get("t_ingest"),
        t_canonical,
        t_end: row.get("t_end"),
        time_quality: row.get("time_quality"),
        blob_hash: row.get("blob_hash"),
        blob_size: row.get("blob_size"),
        indexed: row.get("indexed"),
        source_frame_id: row.get("source_frame_id"),
        payload: row.get("payload"),
    })
}

fn extract_collector_id(origin: &DataOrigin) -> Option<&str> {
    match &origin.origin {
        DataOriginType::DeviceId(id) => Some(id.as_str()),
        DataOriginType::DataOrigin(parent) => extract_collector_id(parent),
    }
}

pub async fn get_by_id(
    pool: &PostgresPool,
    cas: &FsCas,
    id: uuid::Uuid,
) -> Result<lifelog_types::LifelogData, LifelogError> {
    let client = pool
        .get()
        .await
        .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

    let row = client
        .query_opt(
            "SELECT id, collector_id, stream_id, modality, t_device, t_ingest, t_canonical, t_end,
                    time_quality, blob_hash, blob_size, indexed, source_frame_id, payload
             FROM frames WHERE id = $1",
            &[&id],
        )
        .await
        .map_err(|e| LifelogError::Database(format!("frames select: {e}")))?
        .ok_or_else(|| LifelogError::Database(format!("frame not found: {id}")))?;

    let frame_row = row_to_frame_row(&row)?;
    to_lifelog_data(&frame_row, cas)
        .map_err(|e| LifelogError::Database(format!("frame conversion: {e}")))
}

pub async fn get_by_ids(
    pool: &PostgresPool,
    cas: &FsCas,
    ids: &[uuid::Uuid],
) -> Result<Vec<lifelog_types::LifelogData>, LifelogError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let client = pool
        .get()
        .await
        .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

    let rows = client
        .query(
            "SELECT id, collector_id, stream_id, modality, t_device, t_ingest, t_canonical, t_end,
                    time_quality, blob_hash, blob_size, indexed, source_frame_id, payload
             FROM frames WHERE id = ANY($1)",
            &[&ids],
        )
        .await
        .map_err(|e| LifelogError::Database(format!("frames batch select: {e}")))?;

    let mut results = Vec::with_capacity(rows.len());
    for row in &rows {
        match row_to_frame_row(row) {
            Ok(frame_row) => match to_lifelog_data(&frame_row, cas) {
                Ok(data) => results.push(data),
                Err(e) => tracing::error!(error = %e, "Failed to convert frame row"),
            },
            Err(e) => tracing::error!(error = %e, "Failed to parse frame row"),
        }
    }
    Ok(results)
}

pub async fn get_keys_after(
    pool: &PostgresPool,
    origin: &DataOrigin,
    after: DateTime<Utc>,
    limit: usize,
) -> Result<Vec<LifelogFrameKey>, LifelogError> {
    get_keys_after_filtered(pool, origin, after, limit, false).await
}

pub async fn get_keys_after_filtered(
    pool: &PostgresPool,
    origin: &DataOrigin,
    after: DateTime<Utc>,
    limit: usize,
    exclude_derived: bool,
) -> Result<Vec<LifelogFrameKey>, LifelogError> {
    let client = pool
        .get()
        .await
        .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

    let collector_id = extract_collector_id(origin);
    let modality = &origin.modality_name;
    let derived_filter = if exclude_derived {
        " AND source_frame_id IS NULL"
    } else {
        ""
    };

    let rows = if let Some(cid) = collector_id {
        client
            .query(
                &format!(
                    "SELECT id FROM frames WHERE modality = $1 AND collector_id = $2 AND t_canonical > $3{derived_filter} ORDER BY t_canonical ASC LIMIT $4"
                ),
                &[&modality, &cid, &after, &(limit as i64)],
            )
            .await
    } else {
        client
            .query(
                &format!(
                    "SELECT id FROM frames WHERE modality = $1 AND t_canonical > $2{derived_filter} ORDER BY t_canonical ASC LIMIT $3"
                ),
                &[&modality, &after, &(limit as i64)],
            )
            .await
    }
    .map_err(|e| LifelogError::Database(format!("frames keys query: {e}")))?;

    let keys = rows
        .iter()
        .filter_map(|row| {
            let uuid: uuid::Uuid = row.get(0);
            Some(LifelogFrameKey::new(
                lifelog_core::Uuid::from_bytes(*uuid.as_bytes()),
                origin.clone(),
            ))
        })
        .collect();

    Ok(keys)
}

pub async fn count_keys_after(
    pool: &PostgresPool,
    origin: &DataOrigin,
    after: DateTime<Utc>,
) -> Result<i64, LifelogError> {
    let client = pool
        .get()
        .await
        .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

    let collector_id = extract_collector_id(origin);
    let modality = &origin.modality_name;

    let count: i64 = if let Some(cid) = collector_id {
        client
            .query_one(
                "SELECT COUNT(*) FROM frames WHERE modality = $1 AND collector_id = $2 AND t_canonical > $3",
                &[&modality, &cid, &after],
            )
            .await
    } else {
        client
            .query_one(
                "SELECT COUNT(*) FROM frames WHERE modality = $1 AND t_canonical > $2",
                &[&modality, &after],
            )
            .await
    }
    .map_err(|e| LifelogError::Database(format!("frames count query: {e}")))?
    .get(0);

    Ok(count)
}

pub async fn get_origins(pool: &PostgresPool) -> Result<Vec<DataOrigin>, LifelogError> {
    let client = pool
        .get()
        .await
        .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

    let rows = client
        .query("SELECT DISTINCT collector_id, modality FROM frames", &[])
        .await
        .map_err(|e| LifelogError::Database(format!("frames origins query: {e}")))?;

    let origins = rows
        .iter()
        .map(|row| {
            let collector_id: String = row.get(0);
            let modality: String = row.get(1);
            DataOrigin::new(DataOriginType::DeviceId(collector_id), modality)
        })
        .collect();

    Ok(origins)
}

pub async fn insert_transform_output(
    pool: &PostgresPool,
    row: &FrameRow,
) -> Result<bool, LifelogError> {
    let client = pool
        .get()
        .await
        .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

    let sql = "INSERT INTO frames (
            id, collector_id, stream_id, modality, time_range,
            t_device, t_ingest, t_canonical, t_end, time_quality,
            blob_hash, blob_size, indexed, source_frame_id, payload
        ) VALUES (
            $1, $2, $3, $4, tstzrange($5, $6, '[]'),
            $7, $8, $9, $10, $11,
            $12, $13, $14, $15, $16
        )
        ON CONFLICT (source_frame_id, stream_id, modality)
            WHERE source_frame_id IS NOT NULL
        DO NOTHING";

    let params = row.insert_params();
    let rows_affected = client
        .execute(sql, &params)
        .await
        .map_err(|e| LifelogError::Database(format!("transform output insert: {e}")))?;

    if rows_affected == 0 {
        tracing::debug!(
            source_frame_id = ?row.source_frame_id,
            stream_id = %row.stream_id,
            modality = %row.modality,
            "Transform output already exists; skipped duplicate"
        );
    }

    Ok(rows_affected > 0)
}

pub async fn upsert(pool: &PostgresPool, row: &FrameRow) -> Result<(), LifelogError> {
    let client = pool
        .get()
        .await
        .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

    let sql = "INSERT INTO frames (
            id, collector_id, stream_id, modality, time_range,
            t_device, t_ingest, t_canonical, t_end, time_quality,
            blob_hash, blob_size, indexed, source_frame_id, payload
        ) VALUES (
            $1, $2, $3, $4, tstzrange($5, $6, '[]'),
            $7, $8, $9, $10, $11,
            $12, $13, $14, $15, $16
        )
        ON CONFLICT (id) DO UPDATE SET
            payload = EXCLUDED.payload,
            t_ingest = EXCLUDED.t_ingest,
            t_canonical = EXCLUDED.t_canonical,
            t_end = EXCLUDED.t_end,
            time_quality = EXCLUDED.time_quality,
            indexed = EXCLUDED.indexed,
            time_range = EXCLUDED.time_range";

    let params = row.insert_params();
    client
        .execute(sql, &params)
        .await
        .map_err(|e| LifelogError::Database(format!("frames upsert: {e}")))?;

    Ok(())
}
