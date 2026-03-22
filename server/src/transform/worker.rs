use std::sync::Arc;

use chrono::{DateTime, Utc};
use lifelog_core::{DataOrigin, DataOriginType, LifelogError, LifelogFrameKey};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use utils::cas::FsCas;

use crate::postgres::PostgresPool;

use super::dag::TransformDag;
use super::watermark::WatermarkStore;
use super::writer::{extract_source_timestamps, write_transform_output};
use super::TransformExecutor;

pub struct PipelineWorker {
    dag: Arc<TransformDag>,
    watermarks: Arc<dyn WatermarkStore>,
    db: Surreal<Client>,
    postgres_pool: Option<PostgresPool>,
    cas: FsCas,
    http_client: reqwest::Client,
    batch_size: usize,
}

impl PipelineWorker {
    pub fn new(
        dag: Arc<TransformDag>,
        watermarks: Arc<dyn WatermarkStore>,
        db: Surreal<Client>,
        postgres_pool: Option<PostgresPool>,
        cas: FsCas,
        http_client: reqwest::Client,
        batch_size: usize,
    ) -> Self {
        Self {
            dag,
            watermarks,
            db,
            postgres_pool,
            cas,
            http_client,
            batch_size,
        }
    }

    pub async fn poll_once(&self) -> Result<(), LifelogError> {
        let available_origins = crate::db::get_origins_from_db(&self.db).await?;

        // Two-pass approach: first run all primary transforms, then run
        // downstream transforms that may have new input from the first pass.
        // This avoids async recursion while still supporting one level of chaining
        // (e.g., Audio → STT → Transcription, then Transcription → LLM → CleanedTranscription).
        let mut produced_modalities: Vec<String> = Vec::new();

        for transform in self.dag.all_transforms() {
            let did_work = self
                .run_single_transform(transform, &available_origins)
                .await;
            if did_work {
                produced_modalities.push(transform.destination_modality().to_string());
            }
        }

        // Second pass: process any downstream transforms triggered by first-pass outputs.
        for modality in &produced_modalities {
            for downstream in self.dag.transforms_for_modality(modality) {
                // Re-fetch origins since first pass may have written new tables.
                let origins = crate::db::get_origins_from_db(&self.db).await?;
                self.run_single_transform(downstream, &origins).await;
            }
        }

        Ok(())
    }

    async fn run_single_transform(
        &self,
        transform: &Arc<dyn TransformExecutor>,
        available_origins: &[DataOrigin],
    ) -> bool {
        let id = transform.id().to_string();
        let watermark = match self.watermarks.get(&id, "*").await {
            Ok(w) => w,
            Err(e) => {
                tracing::error!(transform_id = %id, error = %e, "Failed to get watermark");
                return false;
            }
        };

        let targets = resolve_targets(&transform.source(), available_origins);
        let mut any_work = false;

        for target_origin in targets {
            let keys = match crate::data_retrieval::get_keys_after(
                &self.db,
                self.postgres_pool.as_ref(),
                &target_origin,
                watermark,
                self.batch_size,
            )
            .await
            {
                Ok(k) => k,
                Err(e) => {
                    tracing::error!(
                        transform_id = %id,
                        origin = %target_origin,
                        error = %e,
                        "Failed to get keys after watermark"
                    );
                    continue;
                }
            };

            if keys.is_empty() {
                continue;
            }

            any_work = true;
            let last_ts = self.process_batch(transform, &keys).await;

            if let Some(ts) = last_ts {
                if let Err(e) = self.watermarks.set(&id, "*", ts).await {
                    tracing::error!(transform_id = %id, error = %e, "Failed to update watermark");
                }
            }
        }

        any_work
    }

    async fn process_batch(
        &self,
        transform: &Arc<dyn TransformExecutor>,
        keys: &[LifelogFrameKey],
    ) -> Option<DateTime<Utc>> {
        let mut last_ts: Option<DateTime<Utc>> = None;

        for key in keys {
            let data = match crate::data_retrieval::get_data_by_key(
                &self.db,
                self.postgres_pool.as_ref(),
                &self.cas,
                key,
            )
            .await
            {
                Ok(d) => d,
                Err(e) => {
                    tracing::error!(uuid = %key.uuid, error = %e, "Failed to load data for transform");
                    continue;
                }
            };

            if !transform.matches_origin(&key.origin) {
                continue;
            }

            let source_timestamps = extract_source_timestamps(&data);

            let output = match transform.execute(&self.http_client, &data, key).await {
                Ok(o) => o,
                Err(e) => {
                    tracing::error!(
                        uuid = %key.uuid,
                        transform = %transform.id(),
                        error = %e,
                        "Transform execution failed"
                    );
                    continue;
                }
            };

            let destination = transform.destination();

            match write_transform_output(
                &self.db,
                self.postgres_pool.as_ref(),
                output,
                &destination,
                &source_timestamps,
            )
            .await
            {
                Ok(Some(ts)) => {
                    last_ts = Some(ts);
                    tracing::debug!(
                        uuid = %key.uuid,
                        transform = %transform.id(),
                        "Transform output written"
                    );
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::error!(
                        uuid = %key.uuid,
                        transform = %transform.id(),
                        error = %e,
                        "Failed to write transform output"
                    );
                }
            }
        }

        last_ts
    }
}

fn resolve_targets(
    source_pattern: &DataOrigin,
    available_origins: &[DataOrigin],
) -> Vec<DataOrigin> {
    if let DataOriginType::DeviceId(ref id) = source_pattern.origin {
        if id == "*" {
            return available_origins
                .iter()
                .filter(|o| o.modality_name == source_pattern.modality_name)
                .cloned()
                .collect();
        }
    }
    vec![source_pattern.clone()]
}
