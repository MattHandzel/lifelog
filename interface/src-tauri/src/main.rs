// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod storage;

use crate::storage::AudioFile;
use base64;
use base64::{engine::general_purpose, Engine as _};
use chrono::Local;
use dirs;

use config::{MicrophoneConfig, ProcessesConfig, ScreenConfig, TextUploadConfig};
use lifelog_interface_lib::{
    api_client,
    config_utils,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use tauri::State;

// Define application state
struct AppState {
    text_config: Mutex<TextUploadConfig>,
    processes_config: Mutex<ProcessesConfig>,
    screen_config: Mutex<ScreenConfig>,
    api_client: reqwest::Client,
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
    let url = format!("{}/api/loggers/text/data", api_client::get_api_base_url());

    let response = state
        .api_client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    let data = response
        .json::<api_client::ApiResponse<Vec<serde_json::Value>>>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !data.success {
        return Err(data.error.unwrap_or_else(|| "Unknown error".to_string()));
    }

    let files = data
        .data
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| {
            Some(TextFile {
                filename: item.get("filename")?.as_str()?.to_string(),
                original_path: item.get("original_path")?.as_str()?.to_string(),
                file_type: item.get("file_type")?.as_str()?.to_string(),
                file_size: item.get("file_size")?.as_u64()?,
                stored_path: item.get("stored_path")?.as_str()?.to_string(),
                content_hash: item.get("content_hash")?.as_str()?.to_string(),
            })
        })
        .collect();

    Ok(files)
}

#[tauri::command]
async fn search_text_files(
    pattern: String,
    state: State<'_, AppState>,
) -> Result<Vec<TextFile>, String> {
    let url = format!(
        "{}/api/loggers/text/search?pattern={}",
        api_client::get_api_base_url(),
        pattern
    );

    let response = state
        .api_client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    let data = response
        .json::<api_client::ApiResponse<Vec<serde_json::Value>>>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !data.success {
        return Err(data.error.unwrap_or_else(|| "Unknown error".to_string()));
    }

    let files = data
        .data
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| {
            Some(TextFile {
                filename: item.get("filename")?.as_str()?.to_string(),
                original_path: item.get("original_path")?.as_str()?.to_string(),
                file_type: item.get("file_type")?.as_str()?.to_string(),
                file_size: item.get("file_size")?.as_u64()?,
                stored_path: item.get("stored_path")?.as_str()?.to_string(),
                content_hash: item.get("content_hash")?.as_str()?.to_string(),
            })
        })
        .collect();

    Ok(files)
}

// This needs to be fixed for Send/Sync issues with MutexGuard
#[tauri::command]
async fn upload_text_file(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<TextFile, String> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    if let Err(e) = std::fs::File::open(&path) {
        return Err(format!("Cannot read file: {}", e));
    }

    let file_content = match std::fs::read(&path) {
        Ok(content) => content,
        Err(e) => return Err(format!("Failed to read file: {}", e)),
    };

    let form = reqwest::multipart::Form::new()
        .text("file_path", file_path.clone())
        .part(
            "file",
            reqwest::multipart::Part::bytes(file_content)
                .file_name(path.file_name().unwrap_or_default().to_string_lossy().into_owned()),
        );

    // Use API client to upload file
    let url = format!("{}/api/loggers/text/upload", api_client::get_api_base_url());

    let response = state
        .api_client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    let data = response
        .json::<api_client::ApiResponse<serde_json::Value>>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !data.success {
        return Err(data.error.unwrap_or_else(|| "Unknown error".to_string()));
    }

    let file_data = data.data.ok_or("No file data returned from server")?;

    Ok(TextFile {
        filename: file_data.get("filename").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        original_path: file_data.get("original_path").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        file_type: file_data.get("file_type").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        file_size: file_data.get("file_size").and_then(|v| v.as_u64()).unwrap_or_default(),
        stored_path: file_data.get("stored_path").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        content_hash: file_data.get("content_hash").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
    })
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
async fn get_current_processes(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let url = format!("{}/api/loggers/processes/current", api_client::get_api_base_url());

    let response = state
        .api_client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    let data = response
        .json::<api_client::ApiResponse<Vec<serde_json::Value>>>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !data.success {
        return Err(data.error.unwrap_or_else(|| "Unknown error".to_string()));
    }

    Ok(data.data.unwrap_or_default())
}

#[tauri::command]
async fn get_process_history(
    start_time: Option<f64>,
    end_time: Option<f64>,
    limit: Option<u32>,
    process_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let mut url = format!("{}/api/loggers/processes/data?", api_client::get_api_base_url());
    
    // Add query parameters
    if let Some(start) = start_time {
        url.push_str(&format!("start_time={}&", start));
    }
    
    if let Some(end) = end_time {
        url.push_str(&format!("end_time={}&", end));
    }
    
    if let Some(limit_val) = limit {
        url.push_str(&format!("limit={}&", limit_val));
    }
    
    if let Some(name) = process_name {
        url.push_str(&format!("process_name={}&", name));
    }
    
    // Remove trailing &
    if url.ends_with('&') {
        url.pop();
    }

    let response = state
        .api_client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    let data = response
        .json::<api_client::ApiResponse<Vec<serde_json::Value>>>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !data.success {
        return Err(data.error.unwrap_or_else(|| "Unknown error".to_string()));
    }

    Ok(data.data.unwrap_or_default())
}

// Screenshot commands
#[tauri::command]
async fn get_screenshots(
    page: u32,
    page_size: u32,
    state: State<'_, AppState>,
) -> Result<Vec<Screenshot>, String> {
    let url = format!(
        "{}/api/loggers/screen/data?page={}&pageSize={}",
        api_client::get_api_base_url(),
        page,
        page_size
    );

    let response = state
        .api_client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    let data = response
        .json::<api_client::ApiResponse<Vec<serde_json::Value>>>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !data.success {
        return Err(data.error.unwrap_or_else(|| "Unknown error".to_string()));
    }

    let screenshots = data
        .data
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| {
            let id = item.get("id")?.as_i64()? as i32;
            let timestamp = item.get("timestamp")?.as_f64()?;
            let path = item.get("path")?.as_str()?.to_string();
            
            Some(Screenshot {
                id,
                timestamp,
                path,
            })
        })
        .collect();

    Ok(screenshots)
}

#[tauri::command]
async fn get_screenshot_settings(state: State<'_, AppState>) -> Result<ScreenConfig, String> {
    let url = format!(
        "{}/api/loggers/screen/config",
        api_client::get_api_base_url()
    );

    let response = state
        .api_client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    let data = response
        .json::<api_client::ApiResponse<ScreenConfig>>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !data.success {
        return Err(data.error.unwrap_or_else(|| "Unknown error".to_string()));
    }

    // Use the data if available, otherwise create a minimal config
    // This replaces the Default trait implementation
    match data.data {
        Some(config) => Ok(config),
        None => {
            // Get config from local
            let config = config_utils::load_screen_config();
            Ok(config)
        }
    }
}

#[tauri::command]
async fn update_screenshot_settings(
    enabled: bool,
    interval: f64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let url = format!(
        "{}/api/loggers/screen/config",
        api_client::get_api_base_url()
    );

    let payload = serde_json::json!({
        "enabled": enabled,
        "interval": interval
    });

    let response = state
        .api_client
        .put(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    // Update local config
    let mut config = state.screen_config.lock().unwrap();
    config.enabled = enabled;
    config.interval = interval;

    Ok(())
}

#[tauri::command]
async fn get_screenshot_data(filename: String, state: State<'_, AppState>) -> Result<String, String> {
    let url = format!(
        "{}/api/files/screen/{}",
        api_client::get_api_base_url(),
        filename
    );

    let response = state
        .api_client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    // Get the bytes and convert to base64
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to get response bytes: {}", e))?;

    let base64_data = general_purpose::STANDARD.encode(&bytes);
    Ok(base64_data)
}

#[tauri::command]
async fn stop_screen_capture(state: State<'_, AppState>) -> Result<(), String> {
    let response = state
        .api_client
        .post(&format!("{}/api/loggers/screen/stop", api_client::get_api_base_url()))
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ))
    }
}

#[tauri::command]
async fn start_screen_capture(
    interval: Option<f64>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut payload = serde_json::json!({});
    
    if let Some(interval_val) = interval {
        payload = serde_json::json!({
            "interval": interval_val
        });
    }
    
    let response = state
        .api_client
        .post(&format!("{}/api/loggers/screen/start", api_client::get_api_base_url()))
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ))
    }
}

#[tauri::command]
async fn initialize_app(
    _window: tauri::Window,
    _app_handle: tauri::AppHandle,
    _state: State<'_, AppState>,
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
            Err(_) => false,
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
                if output_str.contains("Video Devices:")
                    && (output_str.contains("Camera")
                        || output_str.contains("FaceTime")
                        || output_str.contains("iPhone")
                        || output_str.contains("Webcam"))
                {
                    println!("Camera check: Detected cameras: {}", output_str.trim());

                    // Try to check for camera permissions
                    let temp_path = std::env::temp_dir()
                        .join(format!("lifelog_cam_test_{}.jpg", std::process::id()));
                    match Command::new("imagesnap")
                        .arg(temp_path.to_str().unwrap_or("/tmp/test.jpg"))
                        .arg("-w")
                        .arg("0.1") // Short warm-up to not block
                        .output()
                    {
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
                        }
                        Err(e) => {
                            println!("Camera check: Failed to run test capture: {}", e);
                            return false;
                        }
                    }
                } else {
                    println!("Camera check: No cameras detected");
                    return false;
                }
            }
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
async fn get_camera_settings(
    config_manager: tauri::State<'_, Mutex<config_utils::ConfigManager>>,
) -> Result<serde_json::Value, String> {
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
    state: State<'_, AppState>,
) -> Result<(), String> {
    let url = format!("{}/api/loggers/camera/config", api_client::get_api_base_url());
    
    let payload = serde_json::json!({
        "enabled": enabled,
        "interval": interval,
        "fps": fps
    });

    let response = state
        .api_client
        .put(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    // Update local settings
    {
        let mut config_manager = config_manager.lock().map_err(|e| e.to_string())?;
        let mut camera_config = config_manager.get_camera_config();

        camera_config.enabled = enabled;
        camera_config.interval = interval;
        camera_config.fps = fps;

        config_manager.set_camera_config(camera_config);
        config_manager.save().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn get_camera_frames(
    page: usize,
    page_size: usize,
    config_manager: tauri::State<'_, Mutex<config_utils::ConfigManager>>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let url = format!(
        "{}/api/loggers/camera/data?page={}&pageSize={}",
        api_client::get_api_base_url(),
        page,
        page_size
    );

    let response = state
        .api_client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    let data = response
        .json::<api_client::ApiResponse<Vec<serde_json::Value>>>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !data.success {
        return Err(data.error.unwrap_or_else(|| "Unknown error".to_string()));
    }

    Ok(data.data.unwrap_or_default())
}

#[tauri::command]
async fn get_camera_frame_data(filename: String, state: State<'_, AppState>) -> Result<String, String> {
    let url = format!(
        "{}/api/files/camera/{}",
        api_client::get_api_base_url(),
        filename
    );

    let response = state
        .api_client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    // Get the bytes and convert to base64
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to get response bytes: {}", e))?;

    let base64_data = general_purpose::STANDARD.encode(&bytes);
    
    let mime_type = "image/jpeg"; // Assuming all frames are jpg
    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}

#[tauri::command]
async fn trigger_camera_capture(
    config_manager: tauri::State<'_, Mutex<config_utils::ConfigManager>>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let url = format!("{}/api/loggers/camera/capture", api_client::get_api_base_url());
    
    let response = state
        .api_client
        .post(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    Ok(())
}

#[tauri::command]
async fn restart_camera_logger(
    config_manager: tauri::State<'_, Mutex<config_utils::ConfigManager>>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let url = format!("{}/api/loggers/camera/restart", api_client::get_api_base_url());
    
    let response = state
        .api_client
        .post(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    // Capture one frame immediately to verify it works
    println!("Capturing initial test frame");
    let test_result = trigger_camera_capture(config_manager.clone(), state.clone()).await;
    if let Err(e) = &test_result {
        println!("Warning: Initial test capture failed: {}", e);
        // Continue anyway, maybe it will work with the logger
    } else {
        println!("Initial test capture successful");
    }

    Ok(())
}

// Microphone commands
#[tauri::command]
async fn get_microphone_settings(state: State<'_, AppState>) -> Result<MicrophoneConfig, String> {
    let url = format!("{}/api/loggers/microphone/config", api_client::get_api_base_url());

    let response = state
        .api_client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    let data = response
        .json::<api_client::ApiResponse<MicrophoneConfig>>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !data.success {
        return Err(data.error.unwrap_or_else(|| "Unknown error".to_string()));
    }

    // Use the data if available, otherwise create a minimal config
    // This replaces the Default trait implementation
    match data.data {
        Some(config) => Ok(config),
        None => {
            // Get config from local
            let config = config_utils::load_microphone_config();
            Ok(config)
        }
    }
}

#[tauri::command]
async fn start_microphone_recording(state: State<'_, AppState>) -> Result<(), String> {
    let url = format!("{}/api/loggers/microphone/record/start", api_client::get_api_base_url());
    
    let response = state
        .api_client
        .post(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    Ok(())
}

#[tauri::command]
async fn stop_microphone_recording(state: State<'_, AppState>) -> Result<(), String> {
    let url = format!("{}/api/loggers/microphone/record/stop", api_client::get_api_base_url());
    
    let response = state
        .api_client
        .post(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    Ok(())
}

#[tauri::command]
async fn pause_microphone_recording(state: State<'_, AppState>) -> Result<(), String> {
    let url = format!("{}/api/loggers/microphone/record/pause", api_client::get_api_base_url());
    
    let response = state
        .api_client
        .post(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    Ok(())
}

#[tauri::command]
async fn resume_microphone_recording(state: State<'_, AppState>) -> Result<(), String> {
    let url = format!("{}/api/loggers/microphone/record/resume", api_client::get_api_base_url());
    
    let response = state
        .api_client
        .post(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    Ok(())
}

#[tauri::command]
async fn get_audio_files(
    page: usize, 
    page_size: usize,
    state: State<'_, AppState>
) -> Result<Vec<AudioFile>, String> {
    // Use API client to get audio files
    let url = format!(
        "{}/api/loggers/microphone/data?page={}&pageSize={}",
        api_client::get_api_base_url(),
        page,
        page_size
    );

    let response = state
        .api_client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    let data = response
        .json::<api_client::ApiResponse<Vec<AudioFile>>>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !data.success {
        return Err(data.error.unwrap_or_else(|| "Unknown error".to_string()));
    }

    Ok(data.data.unwrap_or_default())
}

// Load processes from database with pagination
#[tauri::command]
async fn get_all_processes(
    start_time: Option<f64>,
    end_time: Option<f64>,
    limit: Option<u32>,
    process_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let mut url = format!("{}/api/loggers/processes/history?", api_client::get_api_base_url());
    
    // Add query parameters
    if let Some(start) = start_time {
        url.push_str(&format!("start_time={}&", start));
    }
    
    if let Some(end) = end_time {
        url.push_str(&format!("end_time={}&", end));
    }
    
    if let Some(limit_val) = limit {
        url.push_str(&format!("limit={}&", limit_val));
    }
    
    if let Some(name) = process_name {
        url.push_str(&format!("process_name={}&", name));
    }
    
    // Remove trailing &
    if url.ends_with('&') {
        url.pop();
    }

    let response = state
        .api_client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    let data = response
        .json::<api_client::ApiResponse<Vec<serde_json::Value>>>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !data.success {
        return Err(data.error.unwrap_or_else(|| "Unknown error".to_string()));
    }

    Ok(data.data.unwrap_or_default())
}

#[tauri::command]
async fn update_microphone_settings(
    enabled: bool,
    chunk_duration_secs: u64,
    capture_interval_secs: Option<u64>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let url = format!("{}/api/loggers/microphone/config", api_client::get_api_base_url());
    
    let mut payload = serde_json::json!({
        "enabled": enabled,
        "chunk_duration_secs": chunk_duration_secs
    });
    
    if let Some(interval) = capture_interval_secs {
        payload["capture_interval_secs"] = serde_json::json!(interval);
    }

    let response = state
        .api_client
        .put(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Server returned error: {}",
            response.status().as_u16()
        ));
    }

    Ok(())
}

fn main() {
    let config_manager = Mutex::new(config_utils::ConfigManager::new());
    
    // Create the application state with API client
    let app_state = AppState {
        text_config: Mutex::new(lifelog_interface_lib::config_utils::load_text_upload_config()),
        processes_config: Mutex::new(lifelog_interface_lib::config_utils::load_processes_config()),
        screen_config: Mutex::new(lifelog_interface_lib::config_utils::load_screen_config()),
        api_client: api_client::create_client(),
    };

    tauri::Builder::default()
        .manage(app_state)
        .manage(config_manager)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Text upload
            get_all_text_files,
            search_text_files,
            upload_text_file,
            select_file_dialog,
            // Processes
            get_current_processes,
            get_process_history,
            // Screenshots
            get_screenshots,
            get_screenshot_settings,
            update_screenshot_settings,
            get_screenshot_data,
            stop_screen_capture,
            start_screen_capture,
            // Camera
            is_camera_supported,
            get_camera_settings,
            update_camera_settings,
            get_camera_frames,
            get_camera_frame_data,
            trigger_camera_capture,
            restart_camera_logger,
            // Microphone
            get_microphone_settings,
            update_microphone_settings,
            start_microphone_recording,
            stop_microphone_recording,
            pause_microphone_recording,
            resume_microphone_recording,
            get_audio_files,
            // General
            get_all_processes,
            initialize_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
