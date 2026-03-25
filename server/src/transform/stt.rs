use async_trait::async_trait;
use lifelog_core::{DataOrigin, DataOriginType, LifelogFrameKey, PrivacyLevel};
use lifelog_types::{DataModality, LifelogData};

use super::{TransformExecutor, TransformOutput, TransformPipelineError};

pub struct SttExecutor {
    id: String,
    source: DataOrigin,
    endpoint: String,
    model: String,
    timeout_secs: u64,
    privacy_level: PrivacyLevel,
}

impl SttExecutor {
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
                .unwrap_or_else(|| "whisper-1".to_string()),
            timeout_secs: params
                .get("timeout_secs")
                .and_then(|v| v.parse().ok())
                .unwrap_or(120),
            privacy_level: {
                let level = PrivacyLevel::from_params(params);
                tracing::info!(privacy_level = %level, params_keys = ?params.keys().collect::<Vec<_>>(), "STT privacy level");
                level
            },
        }
    }
}

#[async_trait]
impl TransformExecutor for SttExecutor {
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
        1
    }
    fn is_async(&self) -> bool {
        true
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
        http: &reqwest::Client,
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

        let audio_bytes = audio_frame.audio_bytes.clone();
        if audio_bytes.is_empty() {
            return Err(TransformPipelineError::DataError(
                "audio frame has no bytes".to_string(),
            ));
        }

        let codec = if audio_frame.codec.is_empty() {
            "wav"
        } else {
            &audio_frame.codec
        };
        let filename = format!("{}.{}", key.uuid, codec);
        let mime = format!("audio/{}", codec);
        let url = format!(
            "{}/v1/audio/transcriptions",
            self.endpoint.trim_end_matches('/')
        );

        let mut last_err = None;
        for attempt in 0..3 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(1 << attempt)).await;
            }
            let part = reqwest::multipart::Part::bytes(audio_bytes.clone())
                .file_name(filename.clone())
                .mime_str(&mime)
                .map_err(|e| TransformPipelineError::ServiceError(format!("mime: {e}")))?;
            let form = reqwest::multipart::Form::new()
                .part("file", part)
                .text("model", self.model.clone());
            match http
                .post(&url)
                .multipart(form)
                .timeout(std::time::Duration::from_secs(self.timeout_secs))
                .send()
                .await
            {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        let status = resp.status();
                        let body = resp.text().await.unwrap_or_default();
                        last_err = Some(TransformPipelineError::ServiceError(format!(
                            "whisper {status}: {body}"
                        )));
                        continue;
                    }
                    let json: serde_json::Value = resp
                        .json()
                        .await
                        .map_err(|e| TransformPipelineError::ServiceError(format!("json: {e}")))?;

                    let text = json["text"].as_str().unwrap_or("").to_string();

                    let frame = lifelog_types::TranscriptionFrame {
                        uuid: key.uuid.to_string(),
                        timestamp: audio_frame.timestamp,
                        text,
                        source_uuid: audio_frame.uuid.clone(),
                        model: self.model.clone(),
                        confidence: 0.0,
                        t_device: audio_frame.t_device,
                        t_ingest: None,
                        t_canonical: audio_frame.t_canonical.or(audio_frame.timestamp),
                        t_end: audio_frame
                            .t_end
                            .or(audio_frame.t_canonical)
                            .or(audio_frame.timestamp),
                        time_quality: audio_frame.time_quality,
                        record_type: lifelog_types::RecordType::Interval as i32,
                    };
                    return Ok(TransformOutput::Transcription(frame));
                }
                Err(e) => {
                    last_err = Some(TransformPipelineError::ServiceUnavailable {
                        endpoint: format!("{url}: {e}"),
                    });
                }
            }
        }
        Err(last_err
            .unwrap_or_else(|| TransformPipelineError::ServiceError("unknown error".to_string())))
    }
}
