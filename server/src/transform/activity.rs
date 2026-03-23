use async_trait::async_trait;
use lifelog_core::{DataOrigin, DataOriginType, LifelogFrameKey};
use lifelog_types::{DataModality, LifelogData};

use super::{TransformExecutor, TransformOutput, TransformPipelineError};

pub struct ActivityClassifierExecutor {
    id: String,
    source: DataOrigin,
    endpoint: String,
    model: String,
    system_prompt: String,
    timeout_secs: u64,
}

impl ActivityClassifierExecutor {
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
                    "Classify the user's current activity based on this screen text. Respond with ONLY a short category label from: coding, browsing, email, documentation, social-media, video, chat, terminal, file-management, design, writing, meeting, idle, other. Then a colon and a one-sentence description. Example: 'coding: editing Rust server code in neovim'".to_string()
                }),
            timeout_secs: params
                .get("timeout_secs")
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
        }
    }
}

#[async_trait]
impl TransformExecutor for ActivityClassifierExecutor {
    fn id(&self) -> &str {
        &self.id
    }
    fn source_modality(&self) -> &str {
        "Ocr"
    }
    fn destination_modality(&self) -> &str {
        "Transcription"
    }
    fn priority(&self) -> u8 {
        3
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

        let ocr_frame = match payload {
            lifelog_types::lifelog_data::Payload::Ocrframe(f) => f,
            _ => {
                return Err(TransformPipelineError::UnsupportedModality {
                    transform: self.id.clone(),
                    modality: format!("{:?}", payload),
                });
            }
        };

        if ocr_frame.text.is_empty() {
            return Ok(TransformOutput::Transcription(
                lifelog_types::TranscriptionFrame {
                    uuid: key.uuid.to_string(),
                    text: "idle".to_string(),
                    source_uuid: ocr_frame.uuid.clone(),
                    model: self.model.clone(),
                    timestamp: ocr_frame.timestamp,
                    confidence: 0.0,
                    t_device: ocr_frame.t_device,
                    t_ingest: None,
                    t_canonical: ocr_frame.t_canonical,
                    t_end: ocr_frame.t_end,
                    time_quality: ocr_frame.time_quality,
                    record_type: ocr_frame.record_type,
                },
            ));
        }

        let url = format!("{}/api/chat", self.endpoint.trim_end_matches('/'));

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": self.system_prompt },
                { "role": "user", "content": &ocr_frame.text }
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

        let classification = json["message"]["content"]
            .as_str()
            .unwrap_or("other")
            .to_string();

        let frame = lifelog_types::TranscriptionFrame {
            uuid: key.uuid.to_string(),
            text: classification,
            source_uuid: ocr_frame.uuid.clone(),
            model: self.model.clone(),
            timestamp: ocr_frame.timestamp,
            confidence: 0.0,
            t_device: ocr_frame.t_device,
            t_ingest: None,
            t_canonical: ocr_frame.t_canonical,
            t_end: ocr_frame.t_end,
            time_quality: ocr_frame.time_quality,
            record_type: ocr_frame.record_type,
        };

        Ok(TransformOutput::Transcription(frame))
    }
}
