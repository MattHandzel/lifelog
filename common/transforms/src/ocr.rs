use definitions::*;
use tesseract_rs::{LeptonicaPix, Tesseract};
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

pub struct TesseractOCRTransform {
    tesseract: Tesseract,
    language: String,
}

impl TesseractOCRTransform {
    pub fn new(language: &str) -> Result<Self, OCRTransformError> {
        let tesseract = Tesseract::new(None, Some(language))
            .map_err(|_| OCRTransformError::TesseractInitError)?;

        Ok(Self {
            tesseract,
            language: language.to_string(),
        })
    }

    pub fn with_config(
        language: &str,
        config_variables: &[(&str, &str)],
    ) -> Result<Self, OCRTransformError> {
        let mut tesseract = Tesseract::new(None, Some(language))
            .map_err(|_| OCRTransformError::TesseractInitError)?;

        for (var, value) in config_variables {
            tesseract
                .set_variable(var, value)
                .map_err(|e| OCRTransformError::ProcessingError(e.to_string()))?;
        }

        Ok(Self {
            tesseract,
            language: language.to_string(),
        })
    }
}

impl Transform for TesseractOCRTransform {
    fn name(&self) -> &str {
        "tesseract_ocr"
    }
}

impl OCRTransform for TesseractOCRTransform {
    fn apply(&self, input: Image) -> Result<Text, TransformError> {
        // Convert your Image type to LeptonicaPix
        let pix = self.image_to_pix(&input)?;

        self.tesseract
            .set_image(&pix)
            .map_err(|e| TransformError::OCR(OCRTransformError::ProcessingError(e.to_string())))?;

        let text = self
            .tesseract
            .get_text()
            .map_err(|e| TransformError::OCR(OCRTransformError::ProcessingError(e.to_string())))?;

        Ok(Text::new(text))
    }
}

impl TesseractOCRTransform {
    fn image_to_pix(&self, image: &Image) -> Result<LeptonicaPix, OCRTransformError> {
        // Convert your Image type to LeptonicaPix
        // This will depend on your specific Image type implementation
        // Here's a simplified example assuming Image contains raw pixel data:

        let (width, height) = image.dimensions();
        let data = image.to_rgba8(); // Convert to RGBA format

        LeptonicaPix::new_from_memory(
            data.as_raw(),
            width as i32,
            height as i32,
            8, // depth
            tesseract_rs::LeptonicaColorType::Rgba,
        )
        .map_err(|_| OCRTransformError::ImageConversionError)
    }
}
