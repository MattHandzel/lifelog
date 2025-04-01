// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use rusqlite::params;
use lifelog::{
  config::{TextUploadConfig, ProcessesConfig, ScreenConfig},
  modules::text_upload,
  setup,
};
use tauri::http::Response;
use chrono::{DateTime, Utc, TimeZone, Local};
use dirs;
use base64;

// Define application state
struct AppState {
  text_config: Mutex<TextUploadConfig>,
  processes_config: Mutex<ProcessesConfig>,
  screen_config: Mutex<ScreenConfig>,
}

// Define types for frontend
#[derive(Serialize, Deserialize, Clone)]
struct TextFile {
  filename: String,
  original_path: String,
  file_type: String,
  file_size: u64,
  stored_path: String,
  content_hash: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Process {
  pid: i32,
  parent_pid: i32,
  name: String,
  executable: Option<String>,
  command: String,
  status: String,
  cpu_usage: f32,
  memory: i64,
  runtime: i32,
  user: Option<String>,
  start_time: f64,
}

#[derive(Serialize, Deserialize, Clone)]
struct Screenshot {
  id: i32,
  timestamp: f64,
  path: String,
}

// Text upload commands
#[tauri::command]
async fn get_all_text_files(state: State<'_, AppState>) -> Result<Vec<TextFile>, String> {
  let config = state.text_config.lock().unwrap();
  
  match text_upload::get_all_files(&config) {
    Ok(files) => {
      let result = files.into_iter().map(|f| TextFile {
        filename: f.filename,
        original_path: f.original_path,
        file_type: f.file_type,
        file_size: f.file_size,
        stored_path: f.stored_path,
        content_hash: f.content_hash,
      }).collect();
      Ok(result)
    },
    Err(e) => Err(e.to_string())
  }
}

#[tauri::command]
async fn search_text_files(pattern: String, state: State<'_, AppState>) -> Result<Vec<TextFile>, String> {
  let config = state.text_config.lock().unwrap();
  
  match text_upload::search_by_filename(&config, &pattern) {
    Ok(files) => {
      let result = files.into_iter().map(|f| TextFile {
        filename: f.filename,
        original_path: f.original_path,
        file_type: f.file_type,
        file_size: f.file_size,
        stored_path: f.stored_path,
        content_hash: f.content_hash,
      }).collect();
      Ok(result)
    },
    Err(e) => Err(e.to_string())
  }
}

// This needs to be fixed for Send/Sync issues with MutexGuard
#[tauri::command]
async fn upload_text_file(file_path: String, state: State<'_, AppState>) -> Result<TextFile, String> {
  // Create a clone of the config to avoid holding the MutexGuard across await points
  let config = {
    let guard = state.text_config.lock().unwrap();
    TextUploadConfig {
      enabled: guard.enabled,
      output_dir: guard.output_dir.clone(),
      supported_formats: guard.supported_formats.clone(),
      max_file_size_mb: guard.max_file_size_mb,
    }
  };
  
  // Ensure the output directory exists
  if let Err(e) = std::fs::create_dir_all(&config.output_dir) {
    return Err(format!("Failed to create output directory: {}", e));
  }
  
  let path = PathBuf::from(&file_path);
  
  // Check if file exists
  if !path.exists() {
    return Err(format!("File not found: {}", file_path));
  }
  
  // Check if file is readable
  if let Err(e) = std::fs::File::open(&path) {
    return Err(format!("Cannot read file: {}", e));
  }
  
  // Initialize the database
  if let Err(e) = setup::setup_text_upload_db(&config.output_dir) {
    return Err(format!("Failed to initialize database: {}", e));
  }
  
  match text_upload::upload_file(&config, &path).await {
    Ok(file) => {
      println!("File uploaded successfully: {}", file.filename);
      Ok(TextFile {
        filename: file.filename,
        original_path: file.original_path,
        file_type: file.file_type,
        file_size: file.file_size,
        stored_path: file.stored_path,
        content_hash: file.content_hash,
      })
    },
    Err(e) => {
      println!("Upload failed: {}", e);
      Err(e.to_string())
    }
  }
}

#[tauri::command]
async fn select_file_dialog() -> Result<String, String> {
  //Temporary fixed path for testing
  #[cfg(target_os = "macos")]
  let path = "/Users/vincenw/Documents/test.txt".to_string();
  
  #[cfg(target_os = "windows")]
  let path = "C:\\Users\\Documents\\test.txt".to_string();
  
  #[cfg(target_os = "linux")]
  let path = "/home/user/Documents/test.txt".to_string();
  
  Ok(path)
}

// Process commands
#[tauri::command]
async fn get_current_processes(_state: State<'_, AppState>) -> Result<Vec<Process>, String> {
  // TODO: To be edited
  Ok(Vec::new())
}

// Screenshot commands
#[tauri::command]
async fn get_screenshots(page: u32, page_size: u32, state: State<'_, AppState>) -> Result<Vec<Screenshot>, String> {
  let config = state.screen_config.lock().unwrap();
  let output_dir = config.output_dir.clone();
  
  // Make sure the output directory exists
  if let Err(e) = std::fs::create_dir_all(&output_dir) {
    return Err(format!("Failed to create output directory: {}", e));
  }
  
  // Open the database connection
  let db_path = output_dir.join("screen.db");
  let conn = match rusqlite::Connection::open(&db_path) {
    Ok(conn) => conn,
    Err(e) => return Err(format!("Failed to open database: {}", e)),
  };
  
  // First, query all entries to check which ones have missing files
  let mut check_stmt = match conn.prepare(
    "SELECT rowid, timestamp FROM screen ORDER BY timestamp DESC"
  ) {
    Ok(stmt) => stmt,
    Err(e) => return Err(format!("Failed to prepare check query: {}", e)),
  };
  
  let entries_to_delete = match check_stmt.query_map([], |row| {
    let id: i32 = row.get(0)?;
    let timestamp: f64 = row.get(1)?;
    
    // Format the path based on the timestamp
    let datetime = DateTime::<Utc>::from_timestamp(
      timestamp.trunc() as i64,
      (timestamp.fract() * 1_000_000_000.0) as u32
    ).unwrap_or_default();
    
    // Convert to local time to match how screenshots are saved
    let local_datetime = Local.from_utc_datetime(&datetime.naive_utc());
    
    // Use the same format string as defined in config
    let path_str = local_datetime.format(&config.timestamp_format).to_string() + ".png";
    
    // Get the full path to check if the file exists
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let full_path = home_dir.join("lifelog_screenshots").join(&path_str);
    
    if !full_path.exists() {
      println!("File for screenshot id {} does not exist: {:?}", id, full_path);
      Ok(Some(id))
    } else {
      Ok(None)
    }
  }) {
    Ok(iter) => iter,
    Err(e) => return Err(format!("Failed to check for missing files: {}", e)),
  };
  
  // Collect IDs of entries with missing files
  let mut ids_to_delete = Vec::new();
  for entry in entries_to_delete {
    match entry {
      Ok(Some(id)) => ids_to_delete.push(id),
      Ok(None) => (), // File exists, nothing to do
      Err(e) => println!("Error checking file existence: {}", e),
    }
  }
  
  // Delete entries with missing files
  if !ids_to_delete.is_empty() {
    println!("Deleting {} entries with missing files: {:?}", ids_to_delete.len(), ids_to_delete);
    for id in ids_to_delete {
      match conn.execute("DELETE FROM screen WHERE rowid = ?", params![id]) {
        Ok(_) => println!("Deleted entry with id {}", id),
        Err(e) => println!("Failed to delete entry with id {}: {}", id, e),
      }
    }
  }
  
  // Calculate offsets for pagination
  let offset = (page - 1) * page_size;
  
  // Now query the database again, but only for valid entries
  let mut stmt = match conn.prepare(
    "SELECT rowid, timestamp FROM screen ORDER BY timestamp DESC LIMIT ? OFFSET ?"
  ) {
    Ok(stmt) => stmt,
    Err(e) => return Err(format!("Failed to prepare query: {}", e)),
  };
  
  let screenshot_iter = match stmt.query_map(params![page_size, offset], |row| {
    let id: i32 = row.get(0)?;
    let timestamp: f64 = row.get(1)?;
    
    // Format the path based on the timestamp
    let datetime = DateTime::<Utc>::from_timestamp(
      timestamp.trunc() as i64,
      (timestamp.fract() * 1_000_000_000.0) as u32
    ).unwrap_or_default();
    
    // Convert to local time to match how screenshots are saved
    let local_datetime = Local.from_utc_datetime(&datetime.naive_utc());
    
    // Use the same format string as defined in config
    let path_str = local_datetime.format(&config.timestamp_format).to_string() + ".png";
    println!("Generated path for screenshot {}: {}", id, path_str);
    
    // Debug the full path that would be used to retrieve this file
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let full_path = home_dir.join("lifelog_screenshots").join(&path_str);
    println!("Full path would be: {:?}", full_path);
    
    // Double-check the file exists (it should at this point)
    if !full_path.exists() {
      println!("WARNING: Generated path does not exist on disk: {:?}", full_path);
    } else {
      println!("GOOD: File exists at expected path: {:?}", full_path);
    }
    
    Ok(Screenshot {
      id,
      timestamp,
      path: path_str,
    })
  }) {
    Ok(iter) => iter,
    Err(e) => return Err(format!("Failed to execute query: {}", e)),
  };
  
  let mut screenshots = Vec::new();
  for screenshot in screenshot_iter {
    match screenshot {
      Ok(s) => screenshots.push(s),
      Err(e) => return Err(format!("Error processing screenshot row: {}", e)),
    }
  }
  
  Ok(screenshots)
}

#[tauri::command]
fn get_screenshot_settings(state: State<'_, AppState>) -> Result<ScreenConfig, String> {
  let config = state.screen_config.lock().unwrap();
  Ok(ScreenConfig {
    enabled: config.enabled,
    interval: config.interval,
    output_dir: config.output_dir.clone(),
    program: config.program.clone(),
    timestamp_format: config.timestamp_format.clone(),
  })
}

#[tauri::command]
async fn update_screenshot_settings(enabled: bool, interval: f64, state: State<'_, AppState>) -> Result<(), String> {
  // Validate interval is within range
  if interval < 30.0 || interval > 600.0 {
    return Err("Interval must be between 30 seconds and 10 minutes".to_string());
  }
  
  // Get a copy of the current config (to check if state changed)
  let old_config = {
    let config = state.screen_config.lock().unwrap();
    ScreenConfig {
      enabled: config.enabled,
      interval: config.interval,
      output_dir: config.output_dir.clone(),
      program: config.program.clone(),
      timestamp_format: config.timestamp_format.clone(),
    }
  };
  
  // Update the settings
  {
    let mut config = state.screen_config.lock().unwrap();
    config.enabled = enabled;
    config.interval = interval;
  }
  
  // If enabled changed to false, stop the logger
  if !enabled && old_config.enabled {
    lifelog::modules::screen::stop_logger();
  }
  
  // If enabled status changed to true OR interval changed while enabled, 
  // we need to restart the logger
  if (enabled && !old_config.enabled) || (enabled && old_config.enabled && interval != old_config.interval) {
    // Get a copy of the current config for the new thread
    let config = {
      let config = state.screen_config.lock().unwrap();
      ScreenConfig {
        enabled: config.enabled,
        interval: config.interval,
        output_dir: config.output_dir.clone(),
        program: config.program.clone(),
        timestamp_format: config.timestamp_format.clone(),
      }
    };
    
    // Start a new logger thread with the updated settings
    std::thread::spawn(move || {
      let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
      runtime.block_on(async {
        if let Err(e) = lifelog::modules::screen::start_logger(&config).await {
          eprintln!("Screenshot logger error: {}", e);
        }
      });
    });
  }
  
  Ok(())
}

#[tauri::command]
async fn get_screenshot_data(filename: String) -> Result<String, String> {
  // Construct the full path to the file in the home directory
  let home_dir = dirs::home_dir().expect("Failed to get home directory");
  let screenshot_dir = home_dir.join("lifelog_screenshots");
  let file_path = screenshot_dir.join(&filename);
  
  println!("Loading screenshot data for: {:?}", file_path);
  
  if !file_path.exists() {
    return Err(format!("Screenshot file does not exist: {}", filename));
  }
  
  // Read the file
  match std::fs::read(&file_path) {
    Ok(data) => {
      // Convert to base64
      let base64_data = base64::encode(&data);
      let mime = if file_path.extension().map_or(false, |ext| ext.to_string_lossy().to_lowercase() == "png") {
        "image/png"
      } else {
        "application/octet-stream"
      };
      
      // Return as data URL
      Ok(format!("data:{};base64,{}", mime, base64_data))
    },
    Err(e) => Err(format!("Failed to read screenshot: {}", e))
  }
}

fn main() {
  // Set up default configurations
  let text_config = TextUploadConfig {
    enabled: true,
    output_dir: PathBuf::from("./data/text_uploads"),
    max_file_size_mb: 10,
    supported_formats: vec!["txt".to_string(), "md".to_string(), "json".to_string(), "csv".to_string()],
  };
  
  // Ensure the output directory exists
  if let Err(e) = std::fs::create_dir_all(&text_config.output_dir) {
    eprintln!("Failed to create text upload output directory: {}", e);
  }
  
  let processes_config = ProcessesConfig {
    enabled: true,
    output_dir: PathBuf::from("./data/processes"),
    interval: 5.0,
  };
  
  // Use the home directory + lifelog_screenshots for storing screenshots
  // Path outside of project directory to avoid file watching issues
  let home_dir = dirs::home_dir().expect("Failed to get home directory");
  let screenshots_dir = home_dir.join("lifelog_screenshots");
  
  let screen_config = ScreenConfig {
    enabled: true,
    output_dir: screenshots_dir.clone(),
    interval: 60.0,
    program: "screencapture".to_string(),
    timestamp_format: "%Y-%m-%d_%H-%M-%S".to_string(),
  };
  
  // Ensure screenshot directory exists
  if let Err(e) = std::fs::create_dir_all(&screen_config.output_dir) {
    eprintln!("Failed to create screenshots output directory: {}", e);
  }
  
  let app_state = AppState {
    text_config: Mutex::new(text_config),
    processes_config: Mutex::new(processes_config),
    screen_config: Mutex::new(screen_config.clone()),
  };

  // Start the screenshot logger in a background task if enabled
  if screen_config.enabled {
    let screen_config_clone = screen_config.clone();
    std::thread::spawn(move || {
      let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
      runtime.block_on(async {
        if let Err(e) = lifelog::modules::screen::start_logger(&screen_config_clone).await {
          eprintln!("Screenshot logger error: {}", e);
        }
      });
    });
  }

  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_dialog::init())
    .register_uri_scheme_protocol("asset", move |_app, request| {
      // Format: tauri://asset/screenshot/FILENAME.png
      let path = request.uri().path();
      println!("Asset request path: {}", path);
      
      // Check if it's a screenshot request
      if path.starts_with("/screenshot/") {
        // Extract just the filename (e.g., "2025-03-31_17-41-09.png")
        let filename = path.trim_start_matches("/screenshot/");
        println!("Extracted filename: '{}'", filename);
        
        // Construct the full path to the file in the home directory
        let home_dir = dirs::home_dir().expect("Failed to get home directory");
        let screenshot_dir = home_dir.join("lifelog_screenshots");
        let file_path = screenshot_dir.join(filename);
        
        println!("Looking for screenshot at: {:?}", file_path);
        
        // Check if the file exists before trying to read it
        if !file_path.exists() {
            eprintln!("ERROR: Screenshot file does not exist: {:?}", file_path);
            
            // List all files in the directory to debug
            if let Ok(entries) = std::fs::read_dir(&screenshot_dir) {
                println!("Files in {:?}:", screenshot_dir);
                
                // Try to find a case-insensitive match
                let mut potential_match: Option<PathBuf> = None;
                
                for entry in entries {
                    if let Ok(entry) = entry {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        // Print exact length and contents for debugging
                        println!("  - {:?} (len: {})", name_str, name_str.len());
                        
                        // Check if this file is similar to the one we're looking for
                        if name_str.contains(&filename[0..10]) {
                            println!("    SIMILAR MATCH! Comparison: '{}' vs '{}'", name_str, filename);
                            // Check exact character by character to find discrepancies
                            for (i, (c1, c2)) in name_str.chars().zip(filename.chars()).enumerate() {
                                if c1 != c2 {
                                    println!("    Mismatch at position {}: '{}' vs '{}'", i, c1, c2);
                                }
                            }
                            
                            // If the filename is the same when converted to lowercase, it's probably a case sensitivity issue
                            if name_str.to_lowercase() == filename.to_lowercase() {
                                println!("    FOUND CASE-INSENSITIVE MATCH: {}", name_str);
                                potential_match = Some(entry.path());
                                break;
                            }
                        }
                    }
                }
                
                // If we found a case-insensitive match, use it instead
                if let Some(match_path) = potential_match {
                    println!("Using case-insensitive match: {:?}", match_path);
                    
                    match std::fs::read(&match_path) {
                        Ok(data) => {
                            println!("Successfully read screenshot file (case-insensitive match): {} bytes", data.len());
                            let mime = if match_path.extension().map_or(false, |ext| ext.to_string_lossy().to_lowercase() == "png") {
                                "image/png"
                            } else {
                                "application/octet-stream"
                            };
                            
                            return Response::builder()
                                .header("Content-Type", mime)
                                .body(data)
                                .unwrap();
                        }
                        Err(e) => {
                            eprintln!("Failed to read case-insensitive match: {} at path: {:?}", e, match_path);
                        }
                    }
                }
            }
            
            return Response::builder()
                .status(404)
                .body(Vec::new())
                .unwrap();
        } else {
            println!("File exists! Size: {} bytes", std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0));
        }
        
        match std::fs::read(&file_path) {
          Ok(data) => {
            println!("Successfully read screenshot file: {} bytes", data.len());
            let mime = if file_path.extension().map_or(false, |ext| ext.to_string_lossy().to_lowercase() == "png") {
              "image/png"
            } else {
              "application/octet-stream"
            };
            
            Response::builder()
               .header("Content-Type", mime)
               .header("Access-Control-Allow-Origin", "*")
               .header("Cache-Control", "no-cache, no-store, must-revalidate")
               .header("Pragma", "no-cache")
               .header("Expires", "0")
               .body(data)
               .unwrap()
          }
          Err(e) => {
            eprintln!("Failed to read screenshot file: {} at path: {:?}", e, file_path);
            Response::builder()
               .status(404)
               .body(Vec::new())
               .unwrap()
          }
        }
      } else {
        Response::builder()
           .status(404)
           .body(Vec::new())
           .unwrap()
      }
    })
    .manage(app_state)
    .invoke_handler(tauri::generate_handler![
      get_all_text_files,
      search_text_files,
      upload_text_file,
      select_file_dialog,
      get_current_processes,
      get_screenshots,
      get_screenshot_settings,
      update_screenshot_settings,
      get_screenshot_data,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
