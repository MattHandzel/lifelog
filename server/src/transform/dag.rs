use std::collections::HashMap;
use std::sync::Arc;

use super::{TransformExecutor, TransformPipelineError};

pub struct TransformDag {
    by_source_modality: HashMap<String, Vec<usize>>,
    transforms: Vec<Arc<dyn TransformExecutor>>,
}

impl std::fmt::Debug for TransformDag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ids: Vec<&str> = self.transforms.iter().map(|t| t.id()).collect();
        f.debug_struct("TransformDag")
            .field("transforms", &ids)
            .field("edges", &self.by_source_modality)
            .finish()
    }
}

impl TransformDag {
    pub fn new(
        transforms: Vec<Arc<dyn TransformExecutor>>,
    ) -> Result<Self, TransformPipelineError> {
        let mut by_source_modality: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, t) in transforms.iter().enumerate() {
            by_source_modality
                .entry(t.source_modality().to_string())
                .or_default()
                .push(idx);
        }

        let dag = Self {
            by_source_modality,
            transforms,
        };
        dag.validate_no_cycles()?;
        Ok(dag)
    }

    pub fn transforms_for_modality(&self, modality: &str) -> Vec<&Arc<dyn TransformExecutor>> {
        self.by_source_modality
            .get(modality)
            .map(|indices| indices.iter().map(|&i| &self.transforms[i]).collect())
            .unwrap_or_default()
    }

    pub fn downstream(&self, transform_idx: usize) -> Vec<usize> {
        let dest = self.transforms[transform_idx].destination_modality();
        self.by_source_modality
            .get(dest)
            .cloned()
            .unwrap_or_default()
    }

    pub fn all_transforms(&self) -> &[Arc<dyn TransformExecutor>] {
        &self.transforms
    }

    fn validate_no_cycles(&self) -> Result<(), TransformPipelineError> {
        let mut visited: HashMap<String, u8> = HashMap::new(); // 0=unvisited, 1=in-progress, 2=done

        for modality in self.by_source_modality.keys() {
            self.dfs_check(modality, &mut visited)?;
        }
        Ok(())
    }

    fn dfs_check(
        &self,
        modality: &str,
        visited: &mut HashMap<String, u8>,
    ) -> Result<(), TransformPipelineError> {
        let state = *visited.get(modality).unwrap_or(&0);
        if state == 2 {
            return Ok(());
        }
        if state == 1 {
            return Err(TransformPipelineError::CycleDetected(modality.to_string()));
        }

        visited.insert(modality.to_string(), 1);

        if let Some(indices) = self.by_source_modality.get(modality) {
            for &idx in indices {
                let dest = self.transforms[idx].destination_modality();
                if dest != modality {
                    self.dfs_check(dest, visited)?;
                }
            }
        }

        visited.insert(modality.to_string(), 2);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform::{TransformOutput, TransformPipelineError};
    use async_trait::async_trait;
    use lifelog_core::{DataOrigin, DataOriginType, LifelogFrameKey};
    use lifelog_types::LifelogData;

    struct MockTransform {
        id: String,
        src: String,
        dst: String,
    }

    #[async_trait]
    impl TransformExecutor for MockTransform {
        fn id(&self) -> &str {
            &self.id
        }
        fn source_modality(&self) -> &str {
            &self.src
        }
        fn destination_modality(&self) -> &str {
            &self.dst
        }
        fn priority(&self) -> u8 {
            0
        }
        fn is_async(&self) -> bool {
            false
        }
        fn matches_origin(&self, _: &DataOrigin) -> bool {
            true
        }
        fn source(&self) -> DataOrigin {
            DataOrigin::new(DataOriginType::DeviceId("*".into()), self.src.clone())
        }
        fn destination(&self) -> DataOrigin {
            DataOrigin::new(DataOriginType::DeviceId("*".into()), self.dst.clone())
        }
        async fn execute(
            &self,
            _: &reqwest::Client,
            _: &LifelogData,
            _: &LifelogFrameKey,
        ) -> Result<TransformOutput, TransformPipelineError> {
            unimplemented!()
        }
    }

    #[test]
    fn valid_dag_no_cycles() {
        let transforms: Vec<Arc<dyn TransformExecutor>> = vec![
            Arc::new(MockTransform {
                id: "ocr".into(),
                src: "Screen".into(),
                dst: "Ocr".into(),
            }),
            Arc::new(MockTransform {
                id: "stt".into(),
                src: "Audio".into(),
                dst: "Transcription".into(),
            }),
            Arc::new(MockTransform {
                id: "llm".into(),
                src: "Transcription".into(),
                dst: "CleanedTranscription".into(),
            }),
        ];
        assert!(TransformDag::new(transforms).is_ok());
    }

    #[test]
    fn detects_cycle() {
        let transforms: Vec<Arc<dyn TransformExecutor>> = vec![
            Arc::new(MockTransform {
                id: "a".into(),
                src: "X".into(),
                dst: "Y".into(),
            }),
            Arc::new(MockTransform {
                id: "b".into(),
                src: "Y".into(),
                dst: "X".into(),
            }),
        ];
        let result = TransformDag::new(transforms);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cycle"));
    }

    #[test]
    fn self_loop_allowed_for_same_modality_refinement() {
        // A transform from Transcription -> Transcription (LLM cleanup) is a self-loop on the modality.
        // This is intentionally allowed — the DAG cycle check skips edges where src == dst.
        let transforms: Vec<Arc<dyn TransformExecutor>> = vec![Arc::new(MockTransform {
            id: "cleanup".into(),
            src: "Transcription".into(),
            dst: "Transcription".into(),
        })];
        assert!(TransformDag::new(transforms).is_ok());
    }
}
