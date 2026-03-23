use crate::postgres::PostgresPool;
use chrono::Utc;
use lifelog_core::LifelogError;
use serde_json::json;
use utils::cas::FsCas;
use uuid::Uuid;

pub async fn generate_daily_summary(
    pool: &PostgresPool,
    _cas: &FsCas,
    http: &reqwest::Client,
    endpoint: &str,
    model: &str,
) -> Result<(), LifelogError> {
    let client = pool
        .get()
        .await
        .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

    let rows = client
        .query(
            "SELECT modality, substring(payload::text, 1, 300) as snippet FROM frames
             WHERE t_canonical > NOW() - INTERVAL '24 hours'
             ORDER BY t_canonical DESC LIMIT 100",
            &[],
        )
        .await
        .map_err(|e| LifelogError::Database(format!("summary query: {e}")))?;

    let mut data_snippets = Vec::new();

    for row in rows {
        let modality: String = row.get(0);
        let snippet: String = row.get(1);

        let truncated = if snippet.len() > 200 {
            format!("{}...", &snippet[..200])
        } else {
            snippet
        };

        data_snippets.push(format!("[{}] {}", modality, truncated));
    }

    if data_snippets.is_empty() {
        tracing::debug!("no data found for daily summary");
        return Ok(());
    }

    let data_str = data_snippets[..data_snippets.len().min(20)].join("\n");

    let prompt = format!(
        "Summarize this person's day based on their computer activity:\n\n{}",
        data_str
    );

    let url = format!("{}/api/chat", endpoint.trim_end_matches('/'));

    let body = json!({
        "model": model,
        "messages": [
            { "role": "user", "content": prompt }
        ],
        "stream": false
    });

    let resp = http
        .post(&url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| LifelogError::Database(format!("summary request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text: String = resp.text().await.unwrap_or_default();
        return Err(LifelogError::Database(format!(
            "ollama summary {status}: {text}"
        )));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| LifelogError::Database(format!("summary json parse: {e}")))?;

    let summary_text = json["message"]["content"]
        .as_str()
        .unwrap_or("unable to generate summary")
        .to_string();

    let today = Utc::now().format("%Y-%m-%d").to_string();

    let frame_row = crate::frames::FrameRow {
        id: Uuid::new_v4(),
        collector_id: "system".to_string(),
        stream_id: "daily-summary".to_string(),
        modality: "Summary".to_string(),
        t_device: None,
        t_ingest: Utc::now(),
        t_canonical: Utc::now(),
        t_end: None,
        time_quality: "system".to_string(),
        blob_hash: None,
        blob_size: None,
        indexed: true,
        source_frame_id: None,
        payload: json!({
            "text": summary_text,
            "date": today,
        }),
    };

    crate::frames::upsert(pool, &frame_row)
        .await
        .map_err(|e| LifelogError::Database(format!("summary upsert failed: {e}")))?;

    tracing::info!("daily summary generated and stored");

    Ok(())
}
