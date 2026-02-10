use lifelog_core::*;

use lifelog_core::{DataOrigin, DataOriginType, LifelogImage, Transform, TransformError};
use lifelog_types::DataModality;
pub use lifelog_types::OcrFrame;
use rusty_tesseract::{Args, Image};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OCRTransformError {
    #[error("Tesseract initialization failed")]
    TesseractInitError,
    #[error("Image conversion failed")]
    ImageConversionError,
    #[error("OCR processing failed: {0}")]
    ProcessingError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrTransform {
    source: DataOrigin,
    destination: DataOrigin,
    config: OcrConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrConfig {
    pub language: String,
    pub engine_path: Option<String>,
}

impl Transform for OcrTransform {
    type Input = LifelogImage;
    type Output = OcrFrame;
    type Config = OcrConfig;

    fn new(source: DataOrigin, config: Self::Config) -> Self {
        Self {
            source: source.clone(),
            destination: DataOrigin::new(
                DataOriginType::DataOrigin(Box::new(source)),
                DataModality::Ocr.as_str_name().to_string(),
            ),
            config,
        }
    }

    fn apply(&self, input: Self::Input) -> Result<Self::Output, TransformError> {
        let img = match Image::from_dynamic_image(&input.image) {
            Ok(image) => image,
            Err(_e) => {
                tracing::warn!("Failed to create rusty_tesseract::Image from dynamic_image");
                let ts = Some(::pbjson_types::Timestamp {
                    seconds: input.timestamp.timestamp(),
                    nanos: input.timestamp.timestamp_subsec_nanos() as i32,
                });
                return Ok(OcrFrame {
                    text: String::new(),
                    uuid: input.uuid.to_string(),
                    timestamp: ts,
                    t_device: ts,
                    t_canonical: ts,
                    t_end: ts,
                    ..Default::default()
                });
            }
        };
        let args = Args {
            lang: self.config.language.clone(),
            ..Default::default()
        };

        let data_output = match rusty_tesseract::image_to_string(&img, &args) {
            Ok(text) => text,
            Err(e) => {
                tracing::warn!(error = ?e, "Tesseract processing error");
                String::new()
            }
        };

        let ts = Some(::pbjson_types::Timestamp {
            seconds: input.timestamp.timestamp(),
            nanos: input.timestamp.timestamp_subsec_nanos() as i32,
        });

        Ok(OcrFrame {
            text: data_output,
            uuid: input.uuid.to_string(),
            timestamp: ts,
            t_device: ts,
            t_canonical: ts,
            t_end: ts,
            ..Default::default()
        })
    }

    fn priority(&self) -> u8 {
        2
    }

    fn config(&self) -> Self::Config {
        self.config.clone()
    }

    fn source(&self) -> DataOrigin {
        self.source.clone()
    }
    fn destination(&self) -> DataOrigin {
        self.destination.clone()
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn ocr_transform_smoke_blank_image() {
        let config = OcrConfig {
            language: "eng".to_string(),
            engine_path: None,
        };
        let source = DataOrigin::new(
            DataOriginType::DeviceId("test-device".to_string()),
            DataModality::Screen.as_str_name().to_string(),
        );
        let transform = OcrTransform::new(source, config);

        let uuid = Uuid::new_v4();
        let timestamp = Utc::now();
        let image = image::DynamicImage::new_rgb8(64, 64);
        let input = LifelogImage {
            uuid,
            timestamp,
            image,
        };

        let out = transform
            .apply(input)
            .expect("OCR transform should not error");
        assert_eq!(out.uuid, uuid.to_string());
        assert_eq!(out.timestamp().timestamp(), timestamp.timestamp());
    }
}
