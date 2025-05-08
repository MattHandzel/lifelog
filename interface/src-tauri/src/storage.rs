// Add these imports at the top
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use hound;  // For WAV file processing
use chrono::{DateTime, TimeZone, Utc, Local};

// Function to get audio files with pagination
pub fn get_audio_files(output_dir: PathBuf, page: usize, page_size: usize) -> Result<Vec<AudioFile>, Box<dyn Error>> {
    let mut audio_files = Vec::new();
    let mut id_counter = 1;
    
    // Ensure the directory exists
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)?;
        return Ok(audio_files);
    }
    
    // Get all .wav files in the directory
    let mut entries: Vec<_> = fs::read_dir(&output_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            
            if path.extension()?.to_str()? == "wav" {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    
    // Sort by modification time (newest first)
    entries.sort_by(|a, b| {
        let a_time = fs::metadata(a).unwrap().modified().unwrap();
        let b_time = fs::metadata(b).unwrap().modified().unwrap();
        b_time.cmp(&a_time)
    });
    
    // Apply pagination
    let start = (page - 1) * page_size;
    let _end = start + page_size;
    
    for path in entries.into_iter().skip(start).take(page_size) {
        let metadata = fs::metadata(&path)?;
        let modified = metadata.modified()?;
        
        // Get file size in bytes
        let file_size = metadata.len();
        
        // Convert system time to DateTime
        let datetime = DateTime::<Utc>::from(modified);
        let local_time = Local.from_utc_datetime(&datetime.naive_utc());
        
        // Format timestamp for display
        let formatted_time = local_time.format("%Y-%m-%d %H:%M:%S").to_string();
        
        // Extract filename from path
        let filename = path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        // Get audio duration
        let duration = estimate_audio_duration(&path)?;
        
        audio_files.push(AudioFile {
            id: id_counter,
            timestamp: datetime.timestamp(),
            path: path.to_string_lossy().to_string(),
            filename,
            duration,
            created_at: formatted_time,
            size: file_size,
        });
        
        id_counter += 1;
        
        if audio_files.len() >= page_size {
            break;
        }
    }
    
    Ok(audio_files)
}

// Simplified function to estimate audio duration from WAV file
fn estimate_audio_duration(path: &PathBuf) -> Result<f64, Box<dyn Error>> {
    let file = fs::File::open(path)?;
    let reader = hound::WavReader::new(file)?;
    
    let spec = reader.spec();
    let duration = reader.duration() as f64 / spec.sample_rate as f64;
    
    Ok(duration)
}

// AudioFile struct for the API
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AudioFile {
    pub id: u32,
    pub timestamp: i64,
    pub path: String,
    pub filename: String,
    pub duration: f64,
    pub created_at: String,
    pub size: u64,
} 