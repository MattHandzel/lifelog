use std::sync::Arc;

use chrono::{DateTime, Utc};
use lifelog_core::{DataOrigin, DataOriginType, LifelogError, LifelogFrameKey, PrivacyTier};
use tokio::task::JoinSet;
use utils::cas::FsCas;

use crate::postgres::PostgresPool;

use super::dag::TransformDag;
use super::watermark::WatermarkStore;
use super::writer::{extract_source_timestamps, write_transform_output};
use super::TransformExecutor;

pub struct PipelineWorker {
    dag: Arc<TransformDag>,
    watermarks: Arc<dyn WatermarkStore>,
    postgres_pool: PostgresPool,
    cas: FsCas,
    http_client: reqwest::Client,
    batch_size: usize,
}

impl PipelineWorker {
    pub fn new(
        dag: Arc<TransformDag>,
        watermarks: Arc<dyn WatermarkStore>,
        postgres_pool: PostgresPool,
        cas: FsCas,
        http_client: reqwest::Client,
        batch_size: usize,
    ) -> Self {
        Self {
            dag,
            watermarks,
            postgres_pool,
            cas,
            http_client,
            batch_size,
        }
    }

    pub async fn poll_once(self: &Arc<Self>) -> Result<(), LifelogError> {
        let available_origins = crate::frames::get_origins(&self.postgres_pool).await?;

        let mut join_set = JoinSet::new();

        for transform in self.dag.all_transforms() {
            let worker = Arc::clone(self);
            let transform = Arc::clone(transform);
            let origins = available_origins.clone();
            join_set.spawn(async move {
                let did_work = worker.run_single_transform(&transform, &origins).await;
                if did_work {
                    Some(transform.destination_modality().to_string())
                } else {
                    None
                }
            });
        }

        let mut produced_modalities: Vec<String> = Vec::new();
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Some(modality)) => produced_modalities.push(modality),
                Ok(None) => {}
                Err(e) => {
                    tracing::error!(error = %e, "Transform task panicked");
                }
            }
        }

        let mut downstream_set = JoinSet::new();
        for modality in &produced_modalities {
            for downstream in self.dag.transforms_for_modality(modality) {
                let worker = Arc::clone(self);
                let downstream = Arc::clone(downstream);
                let pool = self.postgres_pool.clone();
                downstream_set.spawn(async move {
                    let origins = match crate::frames::get_origins(&pool).await {
                        Ok(o) => o,
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to get origins for downstream");
                            return;
                        }
                    };
                    worker.run_single_transform(&downstream, &origins).await;
                });
            }
        }
        while let Some(result) = downstream_set.join_next().await {
            if let Err(e) = result {
                tracing::error!(error = %e, "Downstream transform task panicked");
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

        let source_tier = PrivacyTier::for_modality(transform.source_modality());
        if !transform.privacy_level().can_process(source_tier) {
            tracing::warn!(
                transform_id = %id,
                source_modality = %transform.source_modality(),
                privacy_tier = %source_tier,
                privacy_level = %transform.privacy_level(),
                "Transform privacy level cannot process source modality tier; skipping"
            );
            return false;
        }

        let targets = resolve_targets(&transform.source(), available_origins);

        for target_origin in &targets {
            match crate::frames::count_keys_after(&self.postgres_pool, target_origin, watermark)
                .await
            {
                Ok(backlog) => {
                    if backlog > 10_000 {
                        tracing::error!(
                            transform_id = %id,
                            origin = %target_origin,
                            backlog = backlog,
                            "Severe backlog — consider disabling expensive transforms"
                        );
                    } else if backlog > 1_000 {
                        tracing::warn!(
                            transform_id = %id,
                            origin = %target_origin,
                            backlog = backlog,
                            "Transform backlog exceeds threshold"
                        );
                    } else if backlog > 0 {
                        tracing::debug!(
                            transform_id = %id,
                            origin = %target_origin,
                            backlog = backlog,
                            "Pending frames for transform"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        transform_id = %id,
                        error = %e,
                        "Failed to query backlog count"
                    );
                }
            }
        }

        let mut any_work = false;

        let same_modality = transform.source_modality() == transform.destination_modality();

        for target_origin in targets {
            let keys = match crate::frames::get_keys_after_filtered(
                &self.postgres_pool,
                &target_origin,
                watermark,
                self.batch_size,
                same_modality,
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
            let batch_start = std::time::Instant::now();
            let batch_len = keys.len();
            let last_ts = self.process_batch(transform, &keys).await;
            let elapsed = batch_start.elapsed();
            let rate = if elapsed.as_secs() > 0 {
                (batch_len as f64 / elapsed.as_secs_f64()) * 60.0
            } else {
                0.0
            };
            tracing::info!(
                transform_id = %id,
                frames_processed = batch_len,
                elapsed_ms = elapsed.as_millis() as u64,
                frames_per_min = format_args!("{rate:.1}"),
                "Batch complete"
            );

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
            let data = match crate::frames::get_by_id(
                &self.postgres_pool,
                &self.cas,
                uuid::Uuid::from_bytes(key.uuid.into_bytes()),
            )
            .await
            {
                Ok(d) => d,
                Err(e) => {
                    tracing::error!(uuid = %key.uuid, error = %e, "Failed to load data for transform");
                    break;
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
                    break;
                }
            };

            let destination = transform.destination();

            match write_transform_output(
                &self.postgres_pool,
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
                    break;
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
