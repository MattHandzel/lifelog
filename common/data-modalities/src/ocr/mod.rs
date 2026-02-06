use lifelog_core::*;

use lifelog_macros::lifelog_type;
use lifelog_proto;
use lifelog_types::{DataModality, DataOrigin, DataOriginType, Modality, Transform};
use lifelog_types::{LifelogImage, TransformError};
use rusty_tesseract::{Args, Image};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[lifelog_type(Data)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrFrame {
    pub uuid: ::lifelog_core::uuid::Uuid,
    pub timestamp: ::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc>,
    pub text: String,
}

// TODO: Make this be automatically created
impl Modality for OcrFrame {
    fn into_payload(self) -> lifelog_proto::lifelog_data::Payload {
        lifelog_proto::lifelog_data::Payload::Ocrframe(self.into())
    }

    fn get_table_name() -> &'static str {
        "ocr" // TODO: automatically generate this based on folder name
    }
    /// This returns the surrealdb schema with the table's needed to be filled out.
    /// NOTE: `{table}` is a placeholder for the table name. The ` are required because the table will
    /// contain special characters like ":"
    // TODO: Add searching?
    fn get_surrealdb_schema() -> &'static str {
        r#"
        DEFINE FIELD timestamp ON `{table}` TYPE datetime;
        DEFINE FIELD text ON `{table}` TYPE string;"#
    }
}

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
                DataModality::Ocr,
            ),
            config,
        }
    }

    fn apply(&self, input: Self::Input) -> Result<Self::Output, TransformError> {
        let img = match Image::from_dynamic_image(&input.image) {
            Ok(image) => image,
            Err(_e) => {
                // If image conversion itself fails, we might want a specific error.
                // For now, this will likely lead to Tesseract error or empty output.
                // This path should ideally not be hit if LifelogImage is valid.
                // Consider returning TransformError::ImageConversionFailed or similar.
                eprintln!(
                    "[OCR TRANSFORM] Failed to create rusty_tesseract::Image from dynamic_image"
                );
                // Let's return an empty OcrFrame to avoid panicking here
                return Ok(OcrFrame {
                    text: String::new(),
                    uuid: input.uuid,
                    timestamp: input.timestamp,
                });
            }
        };
        let args = Args {
            lang: self.config.language.clone(), // TODO: Make it so users can type language (like en, eng,
            // English, etc)
            ..Default::default()
        };

        // TODO: Refactor OCR frame to use the boxes so we can better show data to the user.

        let data_output = match rusty_tesseract::image_to_string(&img, &args) {
            Ok(text) => text,
            Err(e) => {
                eprintln!("[OCR TRANSFORM] Tesseract processing error: {:?}", e);
                // Return empty string, allowing the transform to "succeed" with no text.
                // Depending on requirements, one might want to propagate this as an error.
                String::new()
            }
        };
        //let data_output = rusty_tesseract::image_to_data(&img, &args).unwrap();
        //for line in data_output.data {
        //    println!(
        //        "[OCR TRANSFORM] Text: '{}', Confidence: {}, Bounding Box: {:?}",
        //        line.text, line.conf, line.bbox
        //    );
        //}

        Ok(OcrFrame {
            text: data_output,
            uuid: input.uuid,
            timestamp: input.timestamp,
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
            DataModality::Screen,
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

        // We don't assert on specific text output here; OCR output varies by environment.
        let out = transform
            .apply(input)
            .expect("OCR transform should not error");
        assert_eq!(out.uuid, uuid);
        assert_eq!(out.timestamp, timestamp);
    }
}
