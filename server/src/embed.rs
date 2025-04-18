// Image and text embedding functionality using CLIP model
#[cfg(feature = "ml")]
use bincode;
#[cfg(feature = "ml")]
use image::DynamicImage;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ml")]
use tch::{CModule, Kind, Tensor};

#[derive(Debug)]
pub enum EmbedError {
    ModelLoadError(String),
    ImageReadError(String),
    EmbeddingError(String),
    DatabaseError(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct ImageEmbedding {
    id: i32,
    timestamp: f64,
    embedding: Vec<f32>,
    resource_uri: String,
}

#[cfg(feature = "ml")]
/// Loads the CLIP model from a `.pt` file.
pub fn load_clip_model() -> Result<CModule, EmbedError> {
    CModule::load("clip_model.pt")
        .map_err(|e| EmbedError::ModelLoadError(e.to_string()))
}

#[cfg(feature = "ml")]
/// Reads an image from a given file path.
pub fn read_image(path: &str) -> Result<DynamicImage, EmbedError> {
    image::open(path)
        .map_err(|e| EmbedError::ImageReadError(e.to_string()))
}

#[cfg(feature = "ml")]
/// Embeds an image using the CLIP model.
pub fn embed_image(image: DynamicImage, model: &CModule) -> Result<Vec<f32>, EmbedError> {
    let resized = image.resize_exact(224, 224, image::imageops::FilterType::CatmullRom);
    let tensor = Tensor::of_slice(&(resized.to_rgb8().into_raw()))
        .view([1, 3, 224, 224])
        .to_kind(Kind::Float)
        / 255.0;

    model.forward_ts(&[tensor])
        .map_err(|e| EmbedError::EmbeddingError(e.to_string()))
        .map(|embedding| embedding
            .contiguous()
            .view(-1)
            .try_into::<Vec<f32>>()
            .unwrap()
        )
}

#[cfg(feature = "ml")]
/// Embeds text using the CLIP model (assuming it can handle text).
pub fn embed_text(text: &str, model: &CModule) -> Result<Vec<f32>, EmbedError> {
    let text_tensor = Tensor::of_slice(&text.as_bytes()).view([-1]);
    model.forward_ts(&[text_tensor])
        .map_err(|e| EmbedError::EmbeddingError(e.to_string()))
        .map(|embedding| embedding
            .contiguous()
            .view(-1)
            .try_into::<Vec<f32>>()
            .unwrap()
        )
}

#[cfg(feature = "ml")]
/// Stores an image embedding in SQLite.
pub fn store_embedding(
    conn: &Connection,
    timestamp: f64,
    image_path: &str,
    embedding: &[f32]
) -> Result<(), EmbedError> {
    let embedding_blob = bincode::serialize(embedding)
        .map_err(|e| EmbedError::DatabaseError(e.to_string()))?;
        
    conn.execute(
        "INSERT INTO image_embeddings (timestamp, embedding, resource_uri) VALUES (?1, ?2, ?3)",
        params![timestamp, embedding_blob, image_path],
    )
    .map_err(|e| EmbedError::DatabaseError(e.to_string()))?;
    
    Ok(())
}

#[cfg(not(feature = "ml"))]
/// Stores an image embedding in SQLite.
pub fn store_embedding(_conn: &Connection, _timestamp: f64, _image_path: &str, _embedding: &[f32]) -> Result<(), EmbedError> {
    Ok(())
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
pub fn search_images(conn: &Connection, query: &str, model: &CModule) -> Result<Vec<(String, f32)>, EmbedError> {
    let query_embedding = embed_text(query, model)?;

    let mut stmt = conn.prepare("SELECT path, embedding FROM images")
        .map_err(|e| EmbedError::DatabaseError(e.to_string()))?;
        
    let image_results = stmt
        .query_map([], |row| {
            let path: String = row.get(0)?;
            let embedding_blob: Vec<u8> = row.get(1)?;
            let embedding: Vec<f32> = bincode::deserialize(&embedding_blob)
                .map_err(|e| rusqlite::Error::InvalidQuery)?;
            Ok((path, embedding))
        })
        .map_err(|e| EmbedError::DatabaseError(e.to_string()))?;

    let mut results = Vec::new();
    for result in image_results {
        if let Ok((path, image_embedding)) = result {
            let similarity = cosine_similarity(&query_embedding, &image_embedding);
            results.push((path, similarity));
        }
    }

    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    Ok(results)
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
                let image = read_image(path.to_str().unwrap()).expect("Failed to read image");
                let embedding = embed_image(image, model).expect("Failed to embed image");
                store_embedding(conn, 0.0, path.to_str().unwrap(), &embedding).expect("Failed to store embedding");
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
pub fn search_images(_conn: &Connection, _query: &str) -> Result<Vec<(String, f32)>, EmbedError> {
    Ok(Vec::new())
}
