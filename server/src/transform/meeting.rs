use crate::postgres::PostgresPool;
use chrono::Utc;
use lifelog_core::LifelogError;
use serde_json::json;
use uuid::Uuid;

pub async fn detect_meetings(pool: &PostgresPool) -> Result<(), LifelogError> {
    let client = pool
        .get()
        .await
        .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

    let rows = client
        .query(
            "SELECT COUNT(*) as meeting_count FROM frames
             WHERE modality IN ('Hyprland', 'WindowActivity')
             AND t_canonical > NOW() - INTERVAL '1 hour'
             AND (
               payload::text ILIKE '%zoom%'
               OR payload::text ILIKE '%teams%'
               OR payload::text ILIKE '%meet%'
               OR payload::text ILIKE '%slack%'
               OR payload::text ILIKE '%discord%'
             )",
            &[],
        )
        .await
        .map_err(|e| LifelogError::Database(format!("meeting query: {e}")))?;

    if rows.is_empty() {
        return Ok(());
    }

    let meeting_indicators: i64 = rows[0].get(0);

    if meeting_indicators > 0 {
        let uuid = Uuid::new_v4();
        let now = Utc::now();
        let one_hour_ago = now - chrono::Duration::hours(1);

        client
            .execute(
                "INSERT INTO frames
                 (id, collector_id, stream_id, modality, time_range, t_canonical, time_quality, payload)
                 VALUES ($1, $2, $3, $4, tstzrange($5, $6, '[)'), $7, $8, $9)
                 ON CONFLICT DO NOTHING",
                &[
                    &uuid,
                    &"system",
                    &"meeting-detector",
                    &"Meeting",
                    &one_hour_ago,
                    &now,
                    &now,
                    &"inferred",
                    &json!({
                        "detected": true,
                        "indicators": meeting_indicators,
                        "source": "window_activity_detection"
                    }),
                ],
            )
            .await
            .map_err(|e| LifelogError::Database(format!("meeting insert: {e}")))?;

        tracing::debug!(
            meeting_count = meeting_indicators,
            "Meeting detected from window activity"
        );
    }

    Ok(())
}
