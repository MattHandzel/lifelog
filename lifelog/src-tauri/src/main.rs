// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod storage;

use serde::{Deserialize, Serialize};
use std::path::{PathBuf, Path};
use std::sync::Mutex;
use tauri::State;
use rusqlite::params;
use lifelog_interface_lib::{
  config_utils,
  config::{TextUploadConfig, ProcessesConfig, ScreenConfig, MicrophoneConfig},
  modules,
  modules::text_upload,
  setup,
};
use chrono::{DateTime, Utc, TimeZone, Local};
use dirs;
use base64;
use std::fs;
use std::time::SystemTime;
use base64::{Engine as _, engine::general_purpose};
use users;
use std::process::Command;
use rusqlite::Connection;
use crate::storage::AudioFile;

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
fn get_current_processes() -> Result<Vec<serde_json::Value>, String> {
    match std::env::consts::OS {
        "linux" => {
            let users_cache = users::UsersCache::new();
            match modules::processes::get_current_processes(&users_cache) {
                Ok(processes) => {
                    // Convert ProcessInfo to JSON
                    let process_values: Vec<serde_json::Value> = processes
                        .into_iter()
                        .map(|p| {
                            serde_json::json!({
                                "pid": p.pid,
                                "ppid": p.ppid,
                                "name": p.name,
                                "exe": p.exe,
                                "cmdline": p.cmdline,
                                "status": p.status,
                                "cpu_usage": p.cpu_usage,
                                "memory_usage": p.memory_usage,
                                "threads": p.threads,
                                "user": p.user,
                                "start_time": p.start_time
                            })
                        })
                        .collect();
                    Ok(process_values)
                }
                Err(e) => Err(format!("Failed to get processes: {}", e)),
            }
        }
        "macos" => {
            // On macOS, use the ps command to get process information
            match Command::new("ps")
                .args(["aux"])
                .output()
            {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let lines: Vec<&str> = output_str.lines().collect();
                    
                    if lines.is_empty() {
                        return Ok(vec![]);
                    }
                    
                    // Skip the header line
                    let processes: Vec<serde_json::Value> = lines.iter().skip(1).filter_map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 11 {
                            // ps aux format: USER PID %CPU %MEM VSZ RSS TTY STAT START TIME COMMAND
                            let user = parts[0];
                            let pid = parts[1].parse::<i32>().unwrap_or(0);
                            let cpu = parts[2].parse::<f64>().unwrap_or(0.0);
                            let _mem = parts[3].parse::<f64>().unwrap_or(0.0);
                            let rss = parts[5].parse::<i64>().unwrap_or(0);
                            let status = parts[7];
                            let start = parts[8];
                            
                            // Command is the rest of the line
                            let command_idx = line.find(parts[10]).unwrap_or(0);
                            let command = if command_idx > 0 { &line[command_idx..] } else { "" };
                            
                            // Get the process name from the command
                            let name = command
                                .split('/')
                                .last()
                                .unwrap_or(command)
                                .split_whitespace()
                                .next()
                                .unwrap_or(command);
                            
                            Some(serde_json::json!({
                                "pid": pid,
                                "ppid": 0, // ps aux doesn't show ppid by default
                                "name": name,
                                "exe": None::<String>,
                                "cmdline": command,
                                "status": status,
                                "cpu_usage": cpu,
                                "memory_usage": rss,
                                "threads": 0, // ps aux doesn't show thread count by default
                                "user": user,
                                "start_time": start
                            }))
                        } else {
                            None
                        }
                    }).collect();
                    
                    Ok(processes)
                }
                Err(e) => Err(format!("Failed to execute ps command: {}", e)),
            }
        }
        _ => Err("Process listing not implemented for this OS".to_string()),
    }
}

#[tauri::command]
fn get_process_history(
    start_time: Option<f64>,
    end_time: Option<f64>,
    limit: Option<u32>,
    process_name: Option<String>
) -> Result<Vec<serde_json::Value>, String> {
    // Get config for database path
    let config = config_utils::load_config();
    let db_path = Path::new(&config.processes.output_dir).join("processes.db");
    
    // Connect to the database
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    // Build query with filters
    let mut query = "SELECT * FROM processes".to_string();
    let mut params = Vec::<Box<dyn rusqlite::ToSql>>::new();
    let mut where_added = false;
    
    if let Some(start) = start_time {
        query.push_str(" WHERE timestamp >= ?");
        params.push(Box::new(start));
        where_added = true;
    }
    
    if let Some(end) = end_time {
        if where_added {
            query.push_str(" AND timestamp <= ?");
        } else {
            query.push_str(" WHERE timestamp <= ?");
            where_added = true;
        }
        params.push(Box::new(end));
    }
    
    if let Some(name) = process_name {
        if where_added {
            query.push_str(" AND name LIKE ?");
        } else {
            query.push_str(" WHERE name LIKE ?");
            where_added = true;
        }
        params.push(Box::new(format!("%{}%", name)));
    }
    
    // Add order and limit
    query.push_str(" ORDER BY timestamp DESC");
    
    if let Some(lim) = limit {
        query.push_str(" LIMIT ?");
        params.push(Box::new(lim));
    }
    
    // Prepare and execute the query
    let mut stmt = conn.prepare(&query)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;
    
    let process_iter = stmt.query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), |row| {
        Ok(serde_json::json!({
            "timestamp": row.get::<_, f64>(0)?,
            "pid": row.get::<_, i32>(1)?,
            "ppid": row.get::<_, i32>(2)?,
            "name": row.get::<_, String>(3)?,
            "exe": row.get::<_, Option<String>>(4)?,
            "cmdline": row.get::<_, Option<String>>(5)?,
            "status": row.get::<_, String>(6)?,
            "cpu_usage": row.get::<_, Option<f64>>(7)?,
            "memory_usage": row.get::<_, Option<i64>>(8)?,
            "threads": row.get::<_, i32>(9)?,
            "user": row.get::<_, Option<String>>(10)?,
            "start_time": row.get::<_, f64>(11)?
        }))
    }).map_err(|e| format!("Failed to execute query: {}", e))?;
    
    let processes: Result<Vec<_>, _> = process_iter.collect();
    processes.map_err(|e| format!("Failed to collect processes: {}", e))
}

// Screenshot commands
#[tauri::command]
async fn get_screenshots(page: u32, page_size: u32, state: State<'_, AppState>) -> Result<Vec<Screenshot>, String> {
  let config = state.screen_config.lock().unwrap();
  let output_dir = config.output_dir.clone();
  
  println!("Using screenshot output directory: {:?}", output_dir);
  
  // Make sure the output directory exists
  if let Err(e) = std::fs::create_dir_all(&output_dir) {
    return Err(format!("Failed to create output directory: {}", e));
  }
  
  // List files in the output directory to debug
  match fs::read_dir(&output_dir) {
    Ok(entries) => {
      println!("Files in screenshot directory:");
      for entry in entries {
        if let Ok(entry) = entry {
          println!("  {:?}", entry.path());
        }
      }
    },
    Err(e) => println!("Failed to read screenshot directory: {}", e),
  }
  
  // Open the database connection
  let db_path = output_dir.join("screen.db");
  println!("Using database at: {:?}", db_path);
  
  let conn = match rusqlite::Connection::open(&db_path) {
    Ok(conn) => conn,
    Err(e) => return Err(format!("Failed to open database: {}", e)),
  };
  
  // Count total screenshots for debugging
  let count: i32 = match conn.query_row("SELECT COUNT(*) FROM screen", [], |row| row.get(0)) {
    Ok(count) => {
      println!("Total screenshots in database: {}", count);
      count
    },
    Err(e) => {
      println!("Failed to count screenshots: {}", e);
      0
    }
  };
  
  // Calculate the offset
  let offset = (page - 1) * page_size;
  
  // Now query the database again with appropriate limits
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
    let full_path = output_dir.join(&path_str);
    println!("Full path would be: {:?}", full_path);
    
    // Double-check the file exists (it should at this point)
    if !full_path.exists() {
      println!("WARNING: Generated path does not exist on disk: {:?}", full_path);
      
      // Try to find the file with just the filename
      #[cfg(target_os = "macos")]
      {
        let filename_only = Path::new(&path_str).file_name().unwrap_or_default();
        let alt_path = output_dir.join(filename_only);
        println!("Trying alternative macOS path: {:?}", alt_path);
        
        if alt_path.exists() {
          println!("FOUND: File exists at alternative path: {:?}", alt_path);
        }
      }
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
    lifelog_interface_lib::modules::screen::stop_logger();
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
    tokio::spawn(async move {
      if let Err(e) = lifelog_interface_lib::modules::screen::start_logger(&config).await {
          eprintln!("Screenshot logger error: {}", e);
        }
    });
  }
  
  Ok(())
}

#[tauri::command]
async fn get_screenshot_data(filename: String) -> Result<String, String> {
  // Construct the full path to the file in the home directory
  let home_dir = dirs::home_dir().expect("Failed to get home directory");
  let screenshot_dir = home_dir.join("lifelog_screenshots");
  
  // Clean up the filename - remove any leading slashes that might be present
  let clean_filename = filename.trim_start_matches('/').to_string();
  let file_path = screenshot_dir.join(&clean_filename);
  
  println!("Loading screenshot data for: {:?}", file_path);
  
  if !file_path.exists() {
    // Check if the path exists with modification for macOS
    #[cfg(target_os = "macos")]
    {
      // Try alternative paths on macOS
      let alt_path = screenshot_dir.join(Path::new(&clean_filename).file_name().unwrap_or_default());
      println!("Trying alternative path: {:?}", alt_path);
      
      if alt_path.exists() {
        // Use the alternative path if it exists
        return match std::fs::read(&alt_path) {
          Ok(data) => {
            let base64_data = general_purpose::STANDARD.encode(&data);
            let mime = "image/png";
            Ok(format!("data:{};base64,{}", mime, base64_data))
          },
          Err(e) => Err(format!("Failed to read screenshot from alternative path: {}", e))
        };
      }
    }
    
    return Err(format!("Screenshot file does not exist: {}", clean_filename));
  }
  
  // Read the file
  match std::fs::read(&file_path) {
    Ok(data) => {
      // Convert to base64
      let base64_data = general_purpose::STANDARD.encode(&data);
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

#[tauri::command]
async fn stop_screen_capture() -> Result<(), String> {
  lifelog_interface_lib::modules::screen::stop_logger();
  Ok(())
}

#[tauri::command]
async fn start_screen_capture(
  interval: Option<f64>,
  state: State<'_, AppState>
) -> Result<(), String> {
  // Validate interval is within range
  if interval.map_or(false, |i| i < 30.0 || i > 600.0) {
    return Err("Interval must be between 30 seconds and 10 minutes".to_string());
  }
  
  // Clone the config to avoid holding MutexGuard across await
  let config = {
    let config_guard = state.screen_config.lock().unwrap();
    ScreenConfig {
      enabled: config_guard.enabled,
      interval: config_guard.interval,
      output_dir: config_guard.output_dir.clone(),
      program: config_guard.program.clone(),
      timestamp_format: config_guard.timestamp_format.clone(),
    }
  };
  
  tokio::spawn(async move {
    // Attempt to start the screen logger
    if let Err(e) = lifelog_interface_lib::modules::screen::start_logger(&config).await {
      eprintln!("Failed to start screen capture: {}", e);
    }
  });
  
  Ok(())
}

#[tauri::command]
async fn initialize_app(
  _window: tauri::Window,
  _app_handle: tauri::AppHandle,
  _state: State<'_, AppState>
) -> Result<(), String> {
  Ok(())
}

// Add Camera APIs
#[tauri::command]
async fn is_camera_supported() -> bool {
    #[cfg(target_os = "linux")]
    return true;
    
    #[cfg(target_os = "macos")]
    {
        // First check if imagesnap is installed
        let imagesnap_installed = match Command::new("which").arg("imagesnap").output() {
            Ok(output) => output.status.success(),
            Err(_) => false
        };
        
        if !imagesnap_installed {
            println!("Camera check: imagesnap utility not found, camera will not work");
            return false;
        }
        
        // Now check if any cameras are detected
        match Command::new("imagesnap").arg("-l").output() {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                
                // Check if any devices are listed
                if output_str.contains("Video Devices:") && 
                   (output_str.contains("Camera") || 
                    output_str.contains("FaceTime") || 
                    output_str.contains("iPhone") || 
                    output_str.contains("Webcam")) {
                    
                    println!("Camera check: Detected cameras: {}", output_str.trim());
                    
                    // Try to check for camera permissions
                    let temp_path = std::env::temp_dir().join(format!("lifelog_cam_test_{}.jpg", std::process::id()));
                    match Command::new("imagesnap")
                        .arg(temp_path.to_str().unwrap_or("/tmp/test.jpg"))
                        .arg("-w").arg("0.1") // Short warm-up to not block
                        .output() {
                        
                        Ok(capture) => {
                            let success = capture.status.success();
                            // Clean up test file
                            let _ = std::fs::remove_file(temp_path);
                            
                            if !success {
                                let stderr = String::from_utf8_lossy(&capture.stderr);
                                println!("Camera check: Permission issue detected: {}", stderr);
                                return false;
                            }
                            
                            println!("Camera check: Successfully captured test image");
                            return true;
                        },
                        Err(e) => {
                            println!("Camera check: Failed to run test capture: {}", e);
                            return false;
                        }
                    }
                } else {
                    println!("Camera check: No cameras detected");
                    return false;
                }
            },
            Err(e) => {
                println!("Camera check: Failed to list cameras: {}", e);
                return false;
            }
        }
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        println!("Camera check: Platform not supported");
        return false;
    }
}

#[tauri::command]
async fn get_camera_settings(config_manager: tauri::State<'_, Mutex<config_utils::ConfigManager>>) -> Result<serde_json::Value, String> {
    // Get the camera_config and release the mutex guard immediately
    let camera_config = {
        let config_manager = config_manager.lock().map_err(|e| e.to_string())?;
        config_manager.get_camera_config()
    };
    
    Ok(serde_json::json!({
        "enabled": camera_config.enabled,
        "device": camera_config.device,
        "fps": camera_config.fps,
        "interval": camera_config.interval,
        "output_dir": camera_config.output_dir,
        "resolution": camera_config.resolution,
        "timestamp_format": camera_config.timestamp_format,
    }))
}

#[tauri::command]
async fn update_camera_settings(
    enabled: bool,
    interval: f64,
    fps: u32,
    config_manager: tauri::State<'_, Mutex<config_utils::ConfigManager>>,
) -> Result<(), String> {
    // Get the current config to check for changes
    let old_enabled = {
        let config_manager = config_manager.lock().map_err(|e| e.to_string())?;
        let current_config = config_manager.get_camera_config();
        current_config.enabled
    };

    // Update settings
    {
        let mut config_manager = config_manager.lock().map_err(|e| e.to_string())?;
        let mut camera_config = config_manager.get_camera_config();
        
        camera_config.enabled = enabled;
        camera_config.interval = interval;
        camera_config.fps = fps;
        
        config_manager.set_camera_config(camera_config);
        config_manager.save().map_err(|e| e.to_string())?;
    }
    
    // Get a copy of the config for the camera logger
    let camera_config = {
        let config_manager = config_manager.lock().map_err(|e| e.to_string())?;
        config_manager.get_camera_config()
    };
    
    // If enabled state changed, start or stop the camera logger
    if enabled != old_enabled {
        if enabled {
            // Start the camera logger
            println!("Starting camera logger with interval: {} seconds", interval);
            
            // Create the output directory if it doesn't exist
            let output_dir = PathBuf::from(&camera_config.output_dir);
            if !output_dir.exists() {
                fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
            }
            
            // Start the camera logger
            tokio::spawn(async move {
                if let Err(e) = modules::camera::start_camera_logger(&camera_config).await {
                    eprintln!("Failed to start camera logger: {}", e);
                }
            });
        } else {
            // Stop the camera logger - as we don't have a stop function, 
            // the logger will stop on its own at the next iteration when it checks enabled status
            println!("Camera logger will stop at next interval check");
        }
    }
    
    Ok(())
}

#[tauri::command]
async fn get_camera_frames(
    page: usize,
    page_size: usize,
    config_manager: tauri::State<'_, Mutex<config_utils::ConfigManager>>,
) -> Result<Vec<serde_json::Value>, String> {
    // Get the camera_config and release the mutex guard immediately
    let camera_config = {
        let config_manager = config_manager.lock().map_err(|e| e.to_string())?;
        config_manager.get_camera_config()
    };
    
    let output_dir = PathBuf::from(&camera_config.output_dir);
    if !output_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut frames = Vec::new();
    let entries = fs::read_dir(&output_dir).map_err(|e| e.to_string())?;
    
    // First collect all jpg files
    let mut frame_files = Vec::new();
                for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "jpg" {
                    frame_files.push(path);
                }
            }
        }
    }
    
    // Sort by modified time (newest first)
    frame_files.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).unwrap_or_else(|_| SystemTime::now());
        let b_time = b.metadata().and_then(|m| m.modified()).unwrap_or_else(|_| SystemTime::now());
        b_time.cmp(&a_time)
    });
    
    // Paginate
    let start = page.saturating_sub(1) * page_size;
    // We don't actually need to calculate end, just skip and take
    
    // Process the files
    for path in frame_files.iter().skip(start).take(page_size) {
        // No need to extract filename if we're not using it
        let timestamp = if let Ok(meta) = path.metadata() {
            if let Ok(modified) = meta.modified() {
                modified
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64()
            } else {
                // Fallback to current time
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64()
            }
                            } else {
            // Another fallback
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64()
        };
        
        // For this example, we'll use fixed dimensions, but ideally you'd extract from image
        frames.push(serde_json::json!({
            "timestamp": timestamp,
            "path": path.to_string_lossy(),
            "width": camera_config.resolution.0,
            "height": camera_config.resolution.1,
        }));
    }
    
    Ok(frames)
}

#[tauri::command]
async fn get_camera_frame_data(filename: String) -> Result<String, String> {
    let path = PathBuf::from(filename);
    if !path.exists() {
        return Err("File not found".to_string());
    }
    
    // Read the file and encode to base64
    let data = fs::read(&path).map_err(|e| e.to_string())?;
    let base64_data = general_purpose::STANDARD.encode(&data);
    
    // Return as data URL
    let mime_type = "image/jpeg"; // Assuming all frames are jpg
    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}

#[tauri::command]
async fn trigger_camera_capture(
    config_manager: tauri::State<'_, Mutex<config_utils::ConfigManager>>,
) -> Result<(), String> {
    // Get the camera_config and release the mutex guard immediately
    let camera_config = {
        let config_manager = config_manager.lock().map_err(|e| e.to_string())?;
        config_manager.get_camera_config()
    };
    
    // Create the output directory if it doesn't exist
    let output_dir = PathBuf::from(&camera_config.output_dir);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
    }
    
    // Format timestamp
    let timestamp = Local::now().format(&camera_config.timestamp_format);
    let output_path = output_dir.join(format!("{}.jpg", timestamp));
    
    // Call capture_frame from the camera module
    #[cfg(target_os = "linux")]
    {
        let frame = modules::camera::capture_frame(&camera_config)
            .await
            .map_err(|e| e.to_string())?;
        
        fs::write(&output_path, &frame.data).map_err(|e| e.to_string())?;
        Ok(())
    }
    
    #[cfg(target_os = "macos")]
    {
        println!("Triggering camera capture using imagesnap...");
        
        // First check if imagesnap is installed
        let check = Command::new("which")
            .arg("imagesnap")
            .output()
            .map_err(|e| format!("Failed to check for imagesnap: {}", e))?;
            
        if !check.status.success() {
            return Err("imagesnap utility not found. Please install with: brew install imagesnap".to_string());
        }
        
        // Check if cameras are available
        let devices = Command::new("imagesnap")
            .arg("-l")
            .output()
            .map_err(|e| format!("Failed to list cameras: {}", e))?;
            
        let devices_str = String::from_utf8_lossy(&devices.stdout);
        println!("Available cameras: {}", devices_str.trim());
        
        if !devices_str.contains("Video Devices:") || devices_str.lines().count() <= 1 {
            return Err("No cameras were detected. Check camera connections and permissions.".to_string());
        }
        
        // Use imagesnap utility to capture the frame directly to the output path
        println!("Capturing image to: {}", output_path.display());
        
        let output = Command::new("imagesnap")
            .arg("-v") // Verbose output
            .arg("-w") // Warm-up period for better exposure
            .arg("0.5")
            .arg(output_path.to_str().ok_or("Invalid output path".to_string())?)
            .output()
            .map_err(|e| format!("Failed to run imagesnap: {}", e))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if !output.status.success() {
            // Check for permission errors
            if stderr.contains("permission") || stderr.contains("denied") || 
               stdout.contains("permission") || stdout.contains("denied") {
                return Err("Camera access permission denied. Please grant camera access in System Settings > Privacy & Security > Camera.".to_string());
            }
            
            return Err(format!("imagesnap failed: {}\nOutput: {}", stderr, stdout));
        }
        
        // Verify the image was created and has content
        match fs::metadata(&output_path) {
            Ok(meta) if meta.len() > 0 => {
                println!("Successfully captured image: {} bytes", meta.len());
            },
            Ok(_) => {
                return Err("Image was captured but file is empty. Camera may not be working properly.".to_string());
            },
                        Err(e) => {
                return Err(format!("Failed to verify captured image: {}", e));
            }
        }
        
        Ok(())
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    Err("Camera capture is not supported on this platform".to_string())
}

#[tauri::command]
async fn restart_camera_logger(
    config_manager: tauri::State<'_, Mutex<config_utils::ConfigManager>>,
) -> Result<(), String> {
    // Get the camera_config and release the mutex guard immediately
    let camera_config = {
        let config_manager = config_manager.lock().map_err(|e| e.to_string())?;
        config_manager.get_camera_config()
    };
    
    // Only proceed if enabled
    if !camera_config.enabled {
        return Err("Camera is not enabled in settings".to_string());
    }
    
    println!("Manually starting camera logger with interval: {} seconds", camera_config.interval);
    
    // Create the output directory if it doesn't exist
    let output_dir = PathBuf::from(&camera_config.output_dir);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
    }
    
    // Capture one frame immediately to verify it works
    println!("Capturing initial test frame");
    let test_result = trigger_camera_capture(config_manager.clone()).await;
    if let Err(e) = &test_result {
        println!("Warning: Initial test capture failed: {}", e);
        // Continue anyway, maybe it will work with the logger
      } else {
        println!("Initial test capture successful");
    }
    
    // Start the camera logger
    let config_clone = camera_config.clone();
    tokio::spawn(async move {
        if let Err(e) = modules::camera::start_camera_logger(&config_clone).await {
            eprintln!("Failed to start camera logger: {}", e);
        }
    });
    
    Ok(())
}

// Microphone commands
#[tauri::command]
async fn get_microphone_settings() -> Result<MicrophoneConfig, String> {
    // Use the config_utils module to get the microphone config
    let config = lifelog_interface_lib::config::load_config();
    Ok(config.microphone)
}

#[tauri::command]
async fn update_microphone_settings(
    enabled: bool,
    chunk_duration_secs: u64,
    capture_interval_secs: Option<u64>
) -> Result<(), String> {
    let mut config = lifelog_interface_lib::config::load_config();
    
    // Update settings
    config.microphone.enabled = enabled;
    config.microphone.chunk_duration_secs = chunk_duration_secs;
    
    // Update capture interval if provided
    if let Some(interval) = capture_interval_secs {
        config.microphone.capture_interval_secs = interval;
    }
    
    // Save the config with custom code
    let home_dir = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?;
    
    #[cfg(feature = "dev")]
    let config_path: PathBuf = "dev-config.toml".into();

    #[cfg(not(feature = "dev"))]
    let config_path: PathBuf = [home_dir.to_str().unwrap(), ".config/lifelog/config.toml"]
        .iter()
        .collect();
    
    let config_str = toml::to_string(&config).map_err(|e| e.to_string())?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(config_path, config_str).map_err(|e| e.to_string())?;
    
    // Update runtime settings for the microphone module
    modules::microphone::update_settings(&config.microphone);
    
    Ok(())
}

#[tauri::command]
async fn start_microphone_recording() -> Result<(), String> {
    // Update the configuration
    update_microphone_settings(true, 
                               lifelog_interface_lib::config::load_config().microphone.chunk_duration_secs,
                               None)
        .await?;
    
    // Call the start_recording function
    modules::microphone::start_recording();
    Ok(())
}

#[tauri::command]
async fn stop_microphone_recording() -> Result<(), String> {
    // Update the configuration
    update_microphone_settings(false, 
                               lifelog_interface_lib::config::load_config().microphone.chunk_duration_secs,
                               None)
        .await?;
    
    // Call the stop_recording function
    modules::microphone::stop_recording();
    Ok(())
}

#[tauri::command]
async fn pause_microphone_recording() -> Result<(), String> {
    // Call the pause_recording function
    modules::microphone::pause_recording();
    Ok(())
}

#[tauri::command]
async fn resume_microphone_recording() -> Result<(), String> {
    // Call the resume_recording function
    modules::microphone::resume_recording();
    Ok(())
}

#[tauri::command]
async fn get_audio_files(page: usize, page_size: usize) -> Result<Vec<AudioFile>, String> {
    // Get audio files with pagination
    let config = lifelog_interface_lib::config::load_config();
    let output_dir = config.microphone.output_dir;
    
    let files = storage::get_audio_files(output_dir, page, page_size)
        .map_err(|e| e.to_string())?;
    
    Ok(files)
}

// Load processes from database with pagination
#[tauri::command]
async fn get_all_processes(
    start_time: Option<f64>,
    end_time: Option<f64>,
    limit: Option<u32>,
    process_name: Option<String>
) -> Result<Vec<serde_json::Value>, String> {
    // Get config for database path
    let config = lifelog_interface_lib::config::load_config();
    let db_path = Path::new(&config.processes.output_dir).join("processes.db");
    
    // Connect to the database
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    // Build query with filters
    let mut query = "SELECT * FROM processes".to_string();
    let mut params = Vec::<Box<dyn rusqlite::ToSql>>::new();
    let mut where_added = false;
    
    if let Some(start) = start_time {
        query.push_str(" WHERE timestamp >= ?");
        params.push(Box::new(start));
        where_added = true;
    }
    
    if let Some(end) = end_time {
        if where_added {
            query.push_str(" AND timestamp <= ?");
        } else {
            query.push_str(" WHERE timestamp <= ?");
            where_added = true;
        }
        params.push(Box::new(end));
    }
    
    if let Some(name) = process_name {
        if where_added {
            query.push_str(" AND name LIKE ?");
        } else {
            query.push_str(" WHERE name LIKE ?");
            where_added = true;
        }
        params.push(Box::new(format!("%{}%", name)));
    }
    
    // Add order and limit
    query.push_str(" ORDER BY timestamp DESC");
    
    if let Some(lim) = limit {
        query.push_str(" LIMIT ?");
        params.push(Box::new(lim));
    }
    
    // Prepare and execute the query
    let mut stmt = conn.prepare(&query)
        .map_err(|e| format!("Failed to prepare query: {}", e))?;
    
    let process_iter = stmt.query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), |row| {
        Ok(serde_json::json!({
            "timestamp": row.get::<_, f64>(0)?,
            "pid": row.get::<_, i32>(1)?,
            "ppid": row.get::<_, i32>(2)?,
            "name": row.get::<_, String>(3)?,
            "exe": row.get::<_, Option<String>>(4)?,
            "cmdline": row.get::<_, Option<String>>(5)?,
            "status": row.get::<_, String>(6)?,
            "cpu_usage": row.get::<_, Option<f64>>(7)?,
            "memory_usage": row.get::<_, Option<i64>>(8)?,
            "threads": row.get::<_, i32>(9)?,
            "user": row.get::<_, Option<String>>(10)?,
            "start_time": row.get::<_, f64>(11)?
        }))
    }).map_err(|e| format!("Failed to execute query: {}", e))?;
    
    let processes: Result<Vec<_>, _> = process_iter.collect();
    processes.map_err(|e| format!("Failed to collect processes: {}", e))
}

#[tauri::command]
fn start_recording() {
    modules::microphone::start_recording();
}

#[tauri::command]
fn pause_recording() {
    modules::microphone::pause_recording();
}

#[tauri::command]
fn resume_recording() {
    modules::microphone::resume_recording();
}

#[tauri::command]
fn stop_recording() {
    modules::microphone::stop_recording();
}

#[tauri::command]
fn enable_auto_recording() {
    modules::microphone::enable_auto_recording();
}

#[tauri::command]
fn disable_auto_recording() {
    modules::microphone::disable_auto_recording();
}

#[tauri::command]
fn get_microphone_config() -> MicrophoneConfig {
    lifelog_interface_lib::config::load_config().microphone
}

#[tauri::command]
fn update_microphone_config(config: MicrophoneConfig) {
    let mut current_config = lifelog_interface_lib::config::load_config();
    current_config.microphone = config;
    
    // Save the config by writing it directly to the file
    let home_dir = dirs::home_dir().expect("Could not determine home directory");
    
    #[cfg(feature = "dev")]
    let config_path: PathBuf = "dev-config.toml".into();

    #[cfg(not(feature = "dev"))]
    let config_path: PathBuf = [home_dir.to_str().unwrap(), ".config/lifelog/config.toml"]
        .iter()
        .collect();
    
    let config_str = toml::to_string(&current_config).expect("Failed to serialize config");
    if let Some(parent) = config_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(config_path, config_str);
    
    // Update runtime settings for the microphone module
    modules::microphone::update_settings(&current_config.microphone);
}

#[tauri::command]
fn get_recording_status() -> serde_json::Value {
    serde_json::json!({
        "is_recording": modules::microphone::is_recording(),
        "is_paused": modules::microphone::is_paused(),
        "auto_recording_enabled": modules::microphone::is_auto_recording_enabled()
    })
}

fn main() {
  let app_state = AppState {
    text_config: Mutex::new(lifelog_interface_lib::config_utils::load_text_upload_config()),
    processes_config: Mutex::new(lifelog_interface_lib::config_utils::load_processes_config()),
    screen_config: Mutex::new(lifelog_interface_lib::config_utils::load_screen_config()),
  };

  let config_manager = Mutex::new(lifelog_interface_lib::config::ConfigManager::new());

  tauri::Builder::default()
    .manage(app_state)
    .manage(config_manager)
    .invoke_handler(tauri::generate_handler![
      get_all_text_files,
      search_text_files,
      upload_text_file,
      select_file_dialog,
      get_current_processes,
      get_process_history,
      get_screenshots,
      get_screenshot_settings,
      update_screenshot_settings,
      get_screenshot_data,
      stop_screen_capture,
      start_screen_capture,
      initialize_app,
      is_camera_supported,
      get_camera_settings,
      update_camera_settings,
      get_camera_frames,
      get_camera_frame_data,
      trigger_camera_capture,
      restart_camera_logger,
      get_microphone_settings,
      update_microphone_settings,
      start_microphone_recording,
      stop_microphone_recording,
      pause_microphone_recording,
      resume_microphone_recording,
      get_audio_files,
      get_all_processes,
      start_recording,
      pause_recording,
      resume_recording,
      stop_recording,
      enable_auto_recording,
      disable_auto_recording,
      get_microphone_config,
      update_microphone_config,
      get_recording_status,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
