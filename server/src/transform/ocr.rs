use async_trait::async_trait;
use data_modalities::ocr::{OcrConfig, OcrTransform};
use lifelog_core::{DataOrigin, DataOriginType, LifelogFrameKey, LifelogImage, Transform};
use lifelog_types::LifelogData;

use super::{TransformExecutor, TransformOutput, TransformPipelineError};

pub struct OcrExecutor {
    inner: OcrTransform,
    id: String,
}

impl OcrExecutor {
    pub fn new(source: DataOrigin, config: OcrConfig) -> Self {
        Self {
            inner: OcrTransform::new(source, config),
            id: "ocr".to_string(),
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }
}

#[async_trait]
impl TransformExecutor for OcrExecutor {
    fn id(&self) -> &str {
        &self.id
    }

    fn source_modality(&self) -> &str {
        "Screen"
    }

    fn destination_modality(&self) -> &str {
        "Ocr"
    }

    fn priority(&self) -> u8 {
        2
    }

    fn is_async(&self) -> bool {
        false
    }

    fn matches_origin(&self, key_origin: &DataOrigin) -> bool {
        let src = self.inner.source();
        if src.modality_name != key_origin.modality_name {
            return false;
        }
        match &src.origin {
            DataOriginType::DeviceId(device_id) if device_id == "*" => true,
            _ => src == *key_origin,
        }
    }

    fn source(&self) -> DataOrigin {
        self.inner.source()
    }

    fn destination(&self) -> DataOrigin {
        self.inner.destination()
    }

    async fn execute(
        &self,
        _http: &reqwest::Client,
        data: &LifelogData,
        key: &LifelogFrameKey,
    ) -> Result<TransformOutput, TransformPipelineError> {
        let payload = data
            .payload
            .as_ref()
            .ok_or_else(|| TransformPipelineError::DataError("missing payload".to_string()))?;

        let screen_frame = match payload {
            lifelog_types::lifelog_data::Payload::Screenframe(f) => f,
            _ => {
                return Err(TransformPipelineError::UnsupportedModality {
                    transform: self.id.clone(),
                    modality: format!("{:?}", payload),
                });
            }
        };

        let image: LifelogImage = screen_frame.clone().into();
        let inner = self.inner.clone();

        let mut result = tokio::task::spawn_blocking(move || inner.apply(image))
            .await
            .map_err(|e| TransformPipelineError::ServiceError(format!("spawn_blocking: {e}")))?
            .map_err(|e| TransformPipelineError::ServiceError(format!("OCR: {e}")))?;

        result.uuid = key.uuid.to_string();
        Ok(TransformOutput::Ocr(result))
    }
}
