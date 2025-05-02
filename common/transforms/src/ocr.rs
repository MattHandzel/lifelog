use lifelog-core;:*;
use tesseract_rs::{LeptonicaPix, Tesseract};
use thiserror::Error;
use utils::load_image;

#[derive(Debug, Error)]
pub enum OCRTransformError {
    #[error("Tesseract initialization failed")]
    TesseractInitError,
    #[error("Image conversion failed")]
    ImageConversionError,
    #[error("OCR processing failed: {0}")]
    ProcessingError(String),
}

pub struct OcrTransform {
    config: OcrConfig,
}

#[derive(Clone)]
pub struct OcrConfig {
    pub language: String,
    pub engine_path: Option<String>,
}

impl Transform for OcrTransform {
    type Input = Image::DynamicImage;
    type Output = String;
    type Config = OcrConfig;

    fn apply(&self, input: Input) -> Result<Output, TransformError> {
        let text = perform_ocr(input, &self.config).map_err(|e| TransformError::General {
            transform: TransformType::OCR,
            message: format!("OCR failed: {}", e),
        })?;

        Ok(Data::Text(text))
    }

    fn transform_type(&self) -> TransformType {
        TransformType::OCR
    }

    fn priority(&self) -> u8 {
        2
    }
}

impl OcrTransform {
    pub fn new(config: OcrConfig) -> Self {
        OcrTransform { config }
    }
}

// Helper function for OCR processing
fn perform_ocr(image: image::DynamicImage, config: &OcrConfig) -> Result<String, String> {
    let mut engine = tesseract::Tesseract::new(config.engine_path.as_deref(), &config.language)
        .map_err(|e| format!("Engine initialization failed: {}", e))?;

    engine
        .set_image_from_mem(&image.to_rgb8())
        .map_err(|e| format!("Image processing failed: {}", e))?;

    engine
        .get_text()
        .map_err(|e| format!("Text extraction failed: {}", e))
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
