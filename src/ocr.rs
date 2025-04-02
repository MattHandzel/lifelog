use image::DynamicImage;
use rusty_tesseract::{Args, Image};

// Stolen from: https://github.com/nanovin/gaze/blob/main/src-tauri/src/gaze/ocr.rs
pub fn image_to_text(img: &DynamicImage) -> String {
    rusty_tesseract::image_to_string(
        &Image::from_dynamic_image(&img).unwrap(),
        &Args {
            lang: "eng".into(),
            ..Default::default()
        },
    )
    .expect("Failed to perform OCR")
}
