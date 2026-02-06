use crate::setup;
use chrono;
use config::TextUploadConfig;
use rusqlite::params;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;

pub struct TextFile {
    pub filename: String,
    pub original_path: String,
    pub file_type: String,
    pub file_size: u64,
    pub stored_path: String,
    pub content_hash: String,
}

/// Uploads a text file to the lifelog system.
///
/// # Arguments
///
/// * `config` - The text upload configuration
/// * `file_path` - Path to the file to upload
///
/// # Returns
///
/// A Result containing the uploaded file's info or an error
pub async fn upload_file(
    config: &TextUploadConfig,
    file_path: &Path,
) -> Result<TextFile, Box<dyn std::error::Error + Send + Sync>> {
    // Check if the file exists
    if !file_path.exists() {
        return Err(format!("File not found: {:?}", file_path).into());
    }

    // Get file metadata
    let metadata = fs::metadata(file_path)?;

    // Check file size
    let file_size = metadata.len();
    let max_size_bytes = (config.max_file_size_mb as u64) * 1024 * 1024;
    if file_size > max_size_bytes {
        return Err(format!(
            "File size exceeds maximum allowed size of {} MB",
            config.max_file_size_mb
        )
        .into());
    }

    // Get file extension
    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Check if file format is supported
    if !config
        .supported_formats
        .iter()
        .any(|format| format.to_lowercase() == extension)
    {
        return Err(format!(
            "Unsupported file format: {}. Supported formats: {:?}",
            extension, config.supported_formats
        )
        .into());
    }

    // Calculate file hash
    let mut file = fs::File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    let hash = format!("{:x}", hasher.finalize());

    // Create a unique filename
    let original_filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown_file");

    let now = chrono::Local::now();
    let timestamp = now.timestamp() as f64 + now.timestamp_subsec_nanos() as f64 / 1_000_000_000.0;

    let new_filename = format!(
        "{}_{}.{}",
        now.format("%Y-%m-%d_%H-%M-%S"),
        &hash[0..8],
        extension
    );

    // Destination path
    let dest_path = Path::new(&config.output_dir).join(&new_filename);

    // Copy the file
    fs::copy(file_path, &dest_path)?;

    // Create a connection to the database
    let conn = setup::setup_text_upload_db(Path::new(&config.output_dir))?;

    // Record in the database
    conn.execute(
        "INSERT INTO text_uploads (
            timestamp, 
            filename, 
            original_path, 
            file_type, 
            file_size, 
            stored_path, 
            content_hash
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            timestamp,
            original_filename,
            file_path.to_string_lossy().to_string(),
            extension,
            file_size as i64,
            dest_path.to_string_lossy().to_string(),
            hash
        ],
    )?;

    tracing::info!(
        filename = original_filename,
        dest = %dest_path.display(),
        "File uploaded"
    );

    Ok(TextFile {
        filename: original_filename.to_string(),
        original_path: file_path.to_string_lossy().to_string(),
        file_type: extension,
        file_size,
        stored_path: dest_path.to_string_lossy().to_string(),
        content_hash: hash,
    })
}

/// Search for uploaded text files by filename pattern
pub fn search_by_filename(
    config: &TextUploadConfig,
    pattern: &str,
) -> Result<Vec<TextFile>, Box<dyn std::error::Error + Send + Sync>> {
    let conn = setup::setup_text_upload_db(Path::new(&config.output_dir))?;
    let mut stmt = conn.prepare(
        "SELECT 
            filename, 
            original_path, 
            file_type, 
            file_size, 
            stored_path, 
            content_hash 
        FROM text_uploads 
        WHERE filename LIKE ?",
    )?;

    let search_pattern = format!("%{}%", pattern);

    let file_iter = stmt.query_map(params![search_pattern], |row| {
        Ok(TextFile {
            filename: row.get(0)?,
            original_path: row.get(1)?,
            file_type: row.get(2)?,
            file_size: row.get(3)?,
            stored_path: row.get(4)?,
            content_hash: row.get(5)?,
        })
    })?;

    let mut results = Vec::new();
    for file in file_iter {
        results.push(file?);
    }

    Ok(results)
}

/// Get all uploaded text files
pub fn get_all_files(
    config: &TextUploadConfig,
) -> Result<Vec<TextFile>, Box<dyn std::error::Error + Send + Sync>> {
    let conn = setup::setup_text_upload_db(Path::new(&config.output_dir))?;
    let mut stmt = conn.prepare(
        "SELECT 
            filename, 
            original_path, 
            file_type, 
            file_size, 
            stored_path, 
            content_hash 
        FROM text_uploads 
        ORDER BY timestamp DESC",
    )?;

    let file_iter = stmt.query_map([], |row| {
        Ok(TextFile {
            filename: row.get(0)?,
            original_path: row.get(1)?,
            file_type: row.get(2)?,
            file_size: row.get(3)?,
            stored_path: row.get(4)?,
            content_hash: row.get(5)?,
        })
    })?;

    let mut results = Vec::new();
    for file in file_iter {
        results.push(file?);
    }

    Ok(results)
}
