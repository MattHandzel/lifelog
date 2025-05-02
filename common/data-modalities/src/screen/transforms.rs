use super::data::ScreenFrame;
use crate::core::{Transform, TransformResult};

#[derive(Default)]
pub struct ScreenTransforms;

impl Transform for ScreenTransforms {
    type Item = ScreenFrame;

    fn available_transforms(&self) -> Vec<&'static TransformTypes> {
        vec!["ocr", "image_embedding", "sensitive_content_detection"]
    }

    fn apply_transform(
        &self,
        transform_name: &str,
        frame: ScreenFrame,
    ) -> TransformResult<ScreenFrame> {
        match transform_name {
            "ocr" => self.apply_ocr(frame),
            "image_embedding" => self.apply_embedding(frame),
            _ => Err(format!("Unknown transform: {}", transform_name).into()),
        }
    }
}

impl ScreenTransforms {
    fn apply_ocr(&self, mut frame: ScreenFrame) -> TransformResult<ScreenFrame> {
        // OCR implementation
        frame.metadata.contains_sensitive = Some(false);
        Ok(frame)
    }

    fn apply_embedding(&self, frame: ScreenFrame) -> TransformResult<ScreenFrame> {
        // Embedding implementation
        Ok(frame)
    }
}
