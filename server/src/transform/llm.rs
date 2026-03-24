use std::sync::Mutex;

use async_trait::async_trait;
use lifelog_core::{DataOrigin, DataOriginType, LifelogFrameKey, PrivacyLevel};
use lifelog_types::{DataModality, LifelogData};

use super::{TransformExecutor, TransformOutput, TransformPipelineError};

struct RateState {
    call_timestamps: Vec<chrono::DateTime<chrono::Utc>>,
}

pub struct LlmExecutor {
    id: String,
    source: DataOrigin,
    endpoint: String,
    model: String,
    system_prompt: String,
    timeout_secs: u64,
    privacy_level: PrivacyLevel,
    max_calls_per_hour: Option<u32>,
    rate_state: Mutex<RateState>,
}

impl LlmExecutor {
    pub fn new(
        id: String,
        source: DataOrigin,
        endpoint: String,
        params: &std::collections::HashMap<String, String>,
    ) -> Self {
        let max_calls_per_hour = params
            .get("max_calls_per_hour")
            .and_then(|v| v.parse().ok());

        if let Some(limit) = max_calls_per_hour {
            tracing::info!(
                transform_id = %id,
                max_calls_per_hour = limit,
                "LLM transform rate limit configured"
            );
        }

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
            privacy_level: PrivacyLevel::from_params(params),
            max_calls_per_hour,
            rate_state: Mutex::new(RateState {
                call_timestamps: Vec::new(),
            }),
        }
    }

    fn check_rate_limit(&self) -> Result<(), TransformPipelineError> {
        let limit = match self.max_calls_per_hour {
            Some(l) => l,
            None => return Ok(()),
        };

        let now = chrono::Utc::now();
        let one_hour_ago = now - chrono::Duration::hours(1);

        let mut state = self.rate_state.lock().unwrap_or_else(|e| e.into_inner());
        state.call_timestamps.retain(|ts| *ts > one_hour_ago);

        if state.call_timestamps.len() >= limit as usize {
            return Err(TransformPipelineError::ServiceError(format!(
                "rate limit exceeded: {} calls in the last hour (limit: {})",
                state.call_timestamps.len(),
                limit
            )));
        }

        state.call_timestamps.push(now);
        Ok(())
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
        self.check_rate_limit()?;

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

        let sanitized_input = sanitize_llm_input(&transcription.text, &self.id);

        let url = format!("{}/api/chat", self.endpoint.trim_end_matches('/'));

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": self.system_prompt },
                { "role": "user", "content": sanitized_input }
            ],
            "stream": false
        });

        let json = {
            let mut last_err = None;
            let mut result = None;
            for attempt in 0..3u32 {
                if attempt > 0 {
                    tokio::time::sleep(std::time::Duration::from_secs(1 << attempt)).await;
                }

                let resp = match http
                    .post(&url)
                    .json(&body)
                    .timeout(std::time::Duration::from_secs(self.timeout_secs))
                    .send()
                    .await
                {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::warn!(
                            attempt = attempt + 1,
                            error = %e,
                            "LLM request failed, retrying"
                        );
                        last_err = Some(TransformPipelineError::ServiceUnavailable {
                            endpoint: format!("{url}: {e}"),
                        });
                        continue;
                    }
                };

                if !resp.status().is_success() {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    tracing::warn!(
                        attempt = attempt + 1,
                        status = %status,
                        "LLM request returned error, retrying"
                    );
                    last_err = Some(TransformPipelineError::ServiceError(format!(
                        "ollama {status}: {text}"
                    )));
                    continue;
                }

                match resp.json::<serde_json::Value>().await {
                    Ok(j) => {
                        result = Some(j);
                        break;
                    }
                    Err(e) => {
                        last_err = Some(TransformPipelineError::ServiceError(format!("json: {e}")));
                        continue;
                    }
                }
            }
            match result {
                Some(j) => j,
                None => {
                    return Err(last_err.unwrap_or_else(|| {
                        TransformPipelineError::ServiceError(
                            "LLM request failed after 3 attempts".into(),
                        )
                    }));
                }
            }
        };

        let raw_text = json["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let cleaned_text = match validate_llm_output(&raw_text, &transcription.text) {
            Ok(text) => text,
            Err(reason) => {
                tracing::warn!(
                    transform_id = %self.id,
                    uuid = %key.uuid,
                    reason = %reason,
                    "LLM output rejected; keeping original text"
                );
                return Ok(TransformOutput::Transcription(transcription.clone()));
            }
        };

        let confidence = compute_llm_confidence(&cleaned_text, &transcription.text);

        let frame = lifelog_types::TranscriptionFrame {
            uuid: key.uuid.to_string(),
            text: cleaned_text,
            source_uuid: key.uuid.to_string(),
            model: self.model.clone(),
            timestamp: transcription.timestamp,
            confidence,
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

const MAX_INPUT_LEN: usize = 10_000;

const MIN_OUTPUT_LEN: usize = 2;
const MAX_OUTPUT_LEN: usize = 50_000;

const REFUSAL_PHRASES: &[&str] = &[
    "i cannot",
    "i can't",
    "i'm sorry",
    "i am sorry",
    "as an ai",
    "as a language model",
    "i'm unable to",
    "i am unable to",
    "i apologize",
];

const INJECTION_PATTERNS: &[&str] = &[
    "ignore previous instructions",
    "ignore all previous",
    "disregard previous",
    "forget your instructions",
    "new instructions:",
    "system prompt:",
    "you are now",
    "act as",
    "pretend you are",
    "override:",
    "jailbreak",
];

fn sanitize_llm_input(input: &str, transform_id: &str) -> String {
    let lower = input.to_lowercase();
    for pattern in INJECTION_PATTERNS {
        if lower.contains(pattern) {
            tracing::warn!(
                transform_id = %transform_id,
                pattern = %pattern,
                input_len = input.len(),
                "Suspicious content detected in LLM input (possible prompt injection)"
            );
            break;
        }
    }

    let sanitized = if input.len() > MAX_INPUT_LEN {
        tracing::info!(
            transform_id = %transform_id,
            original_len = input.len(),
            truncated_to = MAX_INPUT_LEN,
            "Truncating oversized LLM input"
        );
        &input[..MAX_INPUT_LEN]
    } else {
        input
    };

    sanitized
        .replace("```system", "``` system")
        .replace("<|im_start|>", "")
        .replace("<|im_end|>", "")
}

fn compute_llm_confidence(cleaned: &str, original: &str) -> f32 {
    if original.is_empty() || cleaned.is_empty() {
        return 0.0;
    }
    let orig_len = original.len() as f32;
    let clean_len = cleaned.len() as f32;
    let length_ratio = clean_len.min(orig_len) / clean_len.max(orig_len);

    let orig_words: std::collections::HashSet<&str> = original
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .collect();
    let clean_words: std::collections::HashSet<&str> = cleaned
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .collect();

    let overlap = orig_words.intersection(&clean_words).count() as f32;
    let total = orig_words.union(&clean_words).count() as f32;
    let word_overlap = if total > 0.0 { overlap / total } else { 0.0 };

    (0.4 * length_ratio + 0.6 * word_overlap).clamp(0.0, 1.0)
}

fn validate_llm_output(output: &str, original: &str) -> Result<String, String> {
    let trimmed = output.trim();

    if trimmed.is_empty() {
        return Err("empty response".to_string());
    }

    if trimmed.len() < MIN_OUTPUT_LEN {
        return Err(format!("too short ({} chars)", trimmed.len()));
    }

    if trimmed.len() > MAX_OUTPUT_LEN {
        return Err(format!("too long ({} chars)", trimmed.len()));
    }

    let lower = trimmed.to_lowercase();
    for phrase in REFUSAL_PHRASES {
        if lower.starts_with(phrase) {
            return Err(format!("refusal detected: starts with '{phrase}'"));
        }
    }

    if trimmed.len() > original.len() * 5 && original.len() > 10 {
        return Err(format!(
            "output suspiciously long ({}x original)",
            trimmed.len() / original.len()
        ));
    }

    Ok(trimmed.to_string())
}
