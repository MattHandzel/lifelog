use async_trait::async_trait;
use lifelog_core::{DataOrigin, DataOriginType, LifelogFrameKey, PrivacyLevel};
use lifelog_types::{DataModality, LifelogData, TranscriptionFrame};

use super::{TransformExecutor, TransformOutput, TransformPipelineError};

pub struct SoundClassifierExecutor {
    id: String,
    source: DataOrigin,
    privacy_level: PrivacyLevel,
}

impl SoundClassifierExecutor {
    pub fn new(
        id: String,
        source: DataOrigin,
        params: &std::collections::HashMap<String, String>,
    ) -> Self {
        Self {
            id,
            source,
            privacy_level: PrivacyLevel::from_params(params),
        }
    }
}

#[async_trait]
impl TransformExecutor for SoundClassifierExecutor {
    fn id(&self) -> &str {
        &self.id
    }

    fn source_modality(&self) -> &str {
        "Audio"
    }

    fn destination_modality(&self) -> &str {
        "Transcription"
    }

    fn priority(&self) -> u8 {
        3
    }

    fn is_async(&self) -> bool {
        false
    }

    fn privacy_level(&self) -> PrivacyLevel {
        self.privacy_level
    }

    fn matches_origin(&self, key_origin: &DataOrigin) -> bool {
        let src = self.source();
        if src.modality_name != key_origin.modality_name {
            return false;
        }
        match &src.origin {
            DataOriginType::DeviceId(id) if id == "*" => true,
            _ => src == *key_origin,
        }
    }

    fn source(&self) -> DataOrigin {
        self.source.clone()
    }

    fn destination(&self) -> DataOrigin {
        DataOrigin::new(
            DataOriginType::DataOrigin(Box::new(self.source.clone())),
            DataModality::Transcription.as_str_name().to_string(),
        )
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

        let audio_frame = match payload {
            lifelog_types::lifelog_data::Payload::Audioframe(f) => f,
            _ => {
                return Err(TransformPipelineError::UnsupportedModality {
                    transform: self.id.clone(),
                    modality: format!("{:?}", payload),
                });
            }
        };

        let duration_secs = audio_frame.duration_secs;

        let classification = if duration_secs < 1.0 {
            "silence".to_string()
        } else if duration_secs < 10.0 {
            "short-audio".to_string()
        } else if duration_secs < 120.0 {
            "speech-segment".to_string()
        } else {
            "long-recording".to_string()
        };

        let result = TranscriptionFrame {
            uuid: key.uuid.to_string(),
            text: classification,
            model: "sound-classifier".to_string(),
            confidence: 1.0,
            ..Default::default()
        };

        Ok(TransformOutput::Transcription(result))
    }
}
