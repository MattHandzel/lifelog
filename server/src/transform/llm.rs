use async_trait::async_trait;
use lifelog_core::{DataOrigin, DataOriginType, LifelogFrameKey};
use lifelog_types::{DataModality, LifelogData};

use super::{TransformExecutor, TransformOutput, TransformPipelineError};

pub struct LlmExecutor {
    id: String,
    source: DataOrigin,
    endpoint: String,
    model: String,
    system_prompt: String,
    timeout_secs: u64,
}

impl LlmExecutor {
    pub fn new(
        id: String,
        source: DataOrigin,
        endpoint: String,
        params: &std::collections::HashMap<String, String>,
    ) -> Self {
        Self {
            id,
            source,
            endpoint,
            model: params
                .get("model")
                .cloned()
                .unwrap_or_else(|| "gemma3:4b-it-qat".to_string()),
            system_prompt: params
                .get("system_prompt")
                .cloned()
                .unwrap_or_else(|| {
                    "Clean up this speech-to-text transcription. Fix grammar, remove filler words, maintain the original meaning. Output only the cleaned text.".to_string()
                }),
            timeout_secs: params
                .get("timeout_secs")
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
        }
    }
}

#[async_trait]
impl TransformExecutor for LlmExecutor {
    fn id(&self) -> &str {
        &self.id
    }
    fn source_modality(&self) -> &str {
        "Transcription"
    }
    fn destination_modality(&self) -> &str {
        "Transcription"
    }
    fn priority(&self) -> u8 {
        2
    }
    fn is_async(&self) -> bool {
        true
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
        http: &reqwest::Client,
        data: &LifelogData,
        key: &LifelogFrameKey,
    ) -> Result<TransformOutput, TransformPipelineError> {
        let payload = data
            .payload
            .as_ref()
            .ok_or_else(|| TransformPipelineError::DataError("missing payload".to_string()))?;

        let transcription = match payload {
            lifelog_types::lifelog_data::Payload::Transcriptionframe(f) => f,
            _ => {
                return Err(TransformPipelineError::UnsupportedModality {
                    transform: self.id.clone(),
                    modality: format!("{:?}", payload),
                });
            }
        };

        if transcription.text.is_empty() {
            return Ok(TransformOutput::Transcription(transcription.clone()));
        }

        let url = format!("{}/api/chat", self.endpoint.trim_end_matches('/'));

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": self.system_prompt },
                { "role": "user", "content": &transcription.text }
            ],
            "stream": false
        });

        let resp = http
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .send()
            .await
            .map_err(|e| TransformPipelineError::ServiceUnavailable {
                endpoint: format!("{url}: {e}"),
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(TransformPipelineError::ServiceError(format!(
                "ollama {status}: {text}"
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| TransformPipelineError::ServiceError(format!("json: {e}")))?;

        let cleaned_text = json["message"]["content"]
            .as_str()
            .unwrap_or(&transcription.text)
            .to_string();

        let frame = lifelog_types::TranscriptionFrame {
            uuid: key.uuid.to_string(),
            text: cleaned_text,
            source_uuid: transcription.source_uuid.clone(),
            model: self.model.clone(),
            timestamp: transcription.timestamp,
            confidence: 0.0,
            t_device: transcription.t_device,
            t_ingest: None,
            t_canonical: transcription.t_canonical,
            t_end: transcription.t_end,
            time_quality: transcription.time_quality,
            record_type: transcription.record_type,
        };

        Ok(TransformOutput::Transcription(frame))
    }
}
