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
        let img = Image::from_dynamic_image(&input.image).unwrap();
        let args = Args {
            lang: self.config.language.clone(), // TODO: Make it so users can type language (like en, eng,
            // English, etc)
            ..Default::default()
        };

        // TODO: Refactor OCR frame to use the boxes so we can better show data to the user.

        let data_output = rusty_tesseract::image_to_string(&img, &args).unwrap();
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
    use std::path::PathBuf;

    // Test helper function
    fn get_test_image_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_data")
            .join(name)
    }

    #[test]
    fn test_load_valid_image() {
        let path = get_test_image_path("clear_text.png");
        let result = load_image(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_invalid_image() {
        let result = load_image("this_image_doesn't_exist.jpg");
        assert!(result.is_err());
    }

    #[test]
    fn test_ocr_success() {
        let config = OcrConfig {
            language: "eng".to_string(),
            engine_path: None,
        };
        let transform = OcrTransform::new(config);

        let image = load_image(get_test_image_path("clear_text.png")).unwrap();
        let result = transform.apply(image);

        assert!(result.is_ok());
        let Data::Text(text) = result.unwrap() else {
            panic!("Expected text output");
        };
        assert_eq!(text.trim(), "TEST OCR SAMPLE");
    }

    #[test]
    fn test_ocr_invalid_input() {
        let config = OcrConfig {
            language: "eng".to_string(),
            engine_path: None,
        };
        let transform = OcrTransform::new(config);

        let result = transform.apply(Data::Text("invalid input".to_string()));

        assert!(matches!(
            result,
            Err(TransformError::InvalidInputType {
                transform: TransformType::OCR
            })
        ));
    }

    #[test]
    fn test_ocr_low_quality_image() {
        let config = OcrConfig {
            language: "eng".to_string(),
            engine_path: None,
        };
        let transform = OcrTransform::new(config);

        let image = load_image(get_test_image_path("blurry_text.jpg")).unwrap();
        let result = transform.apply(image);

        assert!(result.is_ok());
        let Data::Text(text) = result.unwrap() else {
            panic!("Expected text output");
        };
        // Test approximate match for low-quality image
        assert!(text.contains("SAMPLE"));
    }
}
