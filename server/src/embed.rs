#[cfg(feature = "ml")]
use bincode;
#[cfg(feature = "ml")]
use image::DynamicImage;
use rusqlite::{params, Connection};
use std::fs;
#[cfg(feature = "ml")]
use tch::{CModule, Kind, Tensor};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct ImageEmbedding {
    id: i32,
    timestamp: f64,
    embedding: Vec<f32>,
    resource_uri: String,
}

#[cfg(feature = "ml")]
/// Loads the CLIP model from a `.pt` file.
pub fn load_clip_model() -> CModule {
    CModule::load("clip_model.pt").expect("Failed to load CLIP model")
}

#[cfg(feature = "ml")]
/// Reads an image from a given file path.
pub fn read_image(path: &str) -> DynamicImage {
    image::open(path).expect("Failed to read image")
}

#[cfg(feature = "ml")]
/// Embeds an image using the CLIP model.
pub fn embed_image(image: DynamicImage, model: &CModule) -> Vec<f32> {
    let resized = image.resize_exact(224, 224, image::imageops::FilterType::CatmullRom);
    let tensor = Tensor::of_slice(&(resized.to_rgb8().into_raw()))
        .view([1, 3, 224, 224])
        .to_kind(Kind::Float)
        / 255.0;

    let embedding = model.forward_ts(&[tensor]).expect("Failed to run model");
    embedding
        .contiguous()
        .view(-1)
        .try_into::<Vec<f32>>()
        .unwrap()
}

#[cfg(feature = "ml")]
/// Embeds text using the CLIP model (assuming it can handle text).
pub fn embed_text(text: &str, model: &CModule) -> Vec<f32> {
    let text_tensor = Tensor::of_slice(&text.as_bytes()).view([-1]); // Modify based on CLIP's text processing
    let embedding = model
        .forward_ts(&[text_tensor])
        .expect("Failed to run model");
    embedding
        .contiguous()
        .view(-1)
        .try_into::<Vec<f32>>()
        .unwrap()
}

#[cfg(feature = "ml")]
/// Stores an image embedding in SQLite.
pub fn store_embedding(conn: &Connection, timestamp: f64, image_path: &str, embedding: &[f32]) {
    let embedding_blob = bincode::serialize(embedding).unwrap();
    conn.execute(
        "INSERT INTO image_embeddings (timestamp, embedding, resource_uri) VALUES (?1, ?2, ?3)",
        params![timestamp, embedding_blob, image_path],
    )
    .expect("Failed to insert embedding into DB");
}

#[cfg(not(feature = "ml"))]
/// Stores an image embedding in SQLite.
pub fn store_embedding(_conn: &Connection, _timestamp: f64, _image_path: &str, _embedding: &[f32]) {
    println!("ML features not enabled on this platform");
}

/// Computes the cosine similarity between two vectors.
fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
    let dot: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
    let norm1 = vec1.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
    let norm2 = vec2.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
    dot / (norm1 * norm2)
}

#[cfg(feature = "ml")]
/// Searches for images similar to a text query.
pub fn search_images(conn: &Connection, query: &str, model: &CModule) {
    let query_embedding = embed_text(query, model);

    let mut stmt = conn.prepare("SELECT path, embedding FROM images").unwrap();
    let image_results = stmt
        .query_map([], |row| {
            let path: String = row.get(0)?;
            let embedding_blob: Vec<u8> = row.get(1)?;
            let embedding: Vec<f32> = bincode::deserialize(&embedding_blob).unwrap();
            Ok((path, embedding))
        })
        .unwrap();

    let mut results = Vec::new();
    for result in image_results {
        if let Ok((path, image_embedding)) = result {
            let similarity = cosine_similarity(&query_embedding, &image_embedding);
            results.push((path, similarity));
        }
    }

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); // Sort by similarity
    for (path, similarity) in results.iter().take(5) {
        println!("Image: {}, Similarity: {:.4}", path, similarity);
    }
}

#[cfg(feature = "ml")]
/// Main function to embed all images in `screen.db`
pub fn process_and_store_images(conn: &Connection, model: &CModule, image_dir: &str) {
    for entry in fs::read_dir(image_dir).expect("Failed to read directory") {
        if let Ok(entry) = entry {
            let path = entry.path();

            match path.extension().and_then(|ext| ext.to_str()) {
                Some("png") | Some("jpg") | Some("jpeg") | Some("gif") => {}
                _ => continue,
            }
            if path.is_file() {
                let image = read_image(path.to_str().unwrap());
                let embedding = embed_image(image, model);
                store_embedding(conn, 0.0, path.to_str().unwrap(), &embedding);
                println!("Embedded and stored: {:?}", path);
            }
        }
    }
}

// Non-ML stubs for platforms without ML support
#[cfg(not(feature = "ml"))]
pub fn process_and_store_images(_conn: &Connection, _image_dir: &str) {
    println!("ML features not enabled on this platform");
}

#[cfg(not(feature = "ml"))]
pub fn search_images(_conn: &Connection, _query: &str) {
    println!("ML features not enabled on this platform");
}
