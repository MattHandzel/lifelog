pub mod dag;
pub mod llm;
pub mod ocr;
pub mod stt;
pub mod watermark;
pub mod worker;
pub mod writer;

use async_trait::async_trait;
use lifelog_core::{DataOrigin, LifelogFrameKey};
use lifelog_types::LifelogData;

#[derive(Debug, thiserror::Error)]
pub enum TransformPipelineError {
    #[error("service unavailable: {endpoint}")]
    ServiceUnavailable { endpoint: String },
    #[error("service error: {0}")]
    ServiceError(String),
    #[error("data error: {0}")]
    DataError(String),
    #[error("unsupported input modality for transform {transform}: {modality}")]
    UnsupportedModality { transform: String, modality: String },
    #[error("cycle detected in transform DAG: {0}")]
    CycleDetected(String),
}

pub enum TransformOutput {
    Ocr(lifelog_types::OcrFrame),
    Transcription(lifelog_types::TranscriptionFrame),
    Embedding(lifelog_types::EmbeddingFrame),
}

#[async_trait]
pub trait TransformExecutor: Send + Sync {
    fn id(&self) -> &str;
    fn source_modality(&self) -> &str;
    fn destination_modality(&self) -> &str;
    fn priority(&self) -> u8;
    fn is_async(&self) -> bool;
    fn matches_origin(&self, key_origin: &DataOrigin) -> bool;
    fn source(&self) -> DataOrigin;
    fn destination(&self) -> DataOrigin;

    async fn execute(
        &self,
        http: &reqwest::Client,
        data: &LifelogData,
        key: &LifelogFrameKey,
    ) -> Result<TransformOutput, TransformPipelineError>;
}
