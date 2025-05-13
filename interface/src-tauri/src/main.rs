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
use serde_json::Value;
use tonic::transport::Channel;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

pub mod google {
    pub mod protobuf {
        tonic::include_proto!("google.protobuf");
    }
}

pub mod lifelog {
    tonic::include_proto!("lifelog");
    pub use lifelog_server_service_client::LifelogServerServiceClient;
}

struct AppState {
    text_config: Mutex<TextUploadConfig>,
    processes_config: Mutex<ProcessesConfig>,
    screen_config: Mutex<ScreenConfig>,
    api_client: reqwest::Client,
}

pub struct GrpcClientState {
    client: Arc<tokio::sync::Mutex<Option<lifelog::LifelogServerServiceClient<Channel>>>>,
}

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

    match data.data {
        Some(config) => Ok(config),
        None => {
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

#[tauri::command]
async fn is_camera_supported() -> bool {
    #[cfg(target_os = "linux")]
    return true;

    #[cfg(target_os = "macos")]
    {
        let imagesnap_installed = match Command::new("which").arg("imagesnap").output() {
            Ok(output) => output.status.success(),
            Err(_) => false,
        };

        if !imagesnap_installed {
            println!("Camera check: imagesnap utility not found, camera will not work");
            return false;
        }

        match Command::new("imagesnap").arg("-l").output() {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);

                if output_str.contains("Video Devices:")
                    && (output_str.contains("Camera")
                        || output_str.contains("FaceTime")
                        || output_str.contains("iPhone")
                        || output_str.contains("Webcam"))
                {
                    println!("Camera check: Detected cameras: {}", output_str.trim());

                    let temp_path = std::env::temp_dir()
                        .join(format!("lifelog_cam_test_{}.jpg", std::process::id()));
                    match Command::new("imagesnap")
                        .arg(temp_path.to_str().unwrap_or("/tmp/test.jpg"))
                        .arg("-w")
                        .arg("0.1")
                        .output()
                    {
                        Ok(capture) => {
                            let success = capture.status.success();
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
    let camera_config = {
        let config_manager = config_manager.lock().map_err(|e| e.to_string())?;
        config_manager.get_camera_config()
    };

    Ok(serde_json::json!({
        "enabled": camera_config.enabled,
        "interval": camera_config.interval,
        "output_dir": camera_config.output_dir.to_str().unwrap_or_default(),
        "device": camera_config.device,
        "resolution_x": camera_config.resolution_x,
        "resolution_y": camera_config.resolution_y,
        "fps": camera_config.fps,
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

    // Get the byes and convert to base64
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to get response bytes: {}", e))?;

    let base64_data = general_purpose::STANDARD.encode(&bytes);
    
    let mime_type = "image/jpeg";
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

    println!("Capturing initial test frame");
    let test_result = trigger_camera_capture(config_manager.clone(), state.clone()).await;
    if let Err(e) = &test_result {
        println!("Warning: Initial test capture failed: {}", e);
    } else {
        println!("Initial test capture successful");
    }

    Ok(())
}

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

    match data.data {
        Some(config) => Ok(config),
        None => {
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

#[tauri::command]
async fn get_all_processes(
    start_time: Option<f64>,
    end_time: Option<f64>,
    limit: Option<u32>,
    process_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let mut url = format!("{}/api/loggers/processes/history?", api_client::get_api_base_url());
    
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

const GRPC_SERVER_ADDRESS: &str = "http://localhost:50051";

#[tauri::command]
async fn get_component_config(collector_id: String, component_type: String) -> Result<Value, String> {
    println!("Attempting to get config for collector '{}', component type '{}' via ServerService", collector_id, component_type);
    println!("gRPC: get_component_config - connecting to {}", GRPC_SERVER_ADDRESS);
    let channel = Channel::from_static(GRPC_SERVER_ADDRESS)
        .connect()
        .await
        .map_err(|e| format!("Failed to connect to gRPC server: {}", e))?;
    println!("gRPC: get_component_config - connection established");
    let mut client = lifelog::LifelogServerServiceClient::new(channel);
    
    let request = tonic::Request::new(lifelog::GetSystemConfigRequest {});
    println!("gRPC: get_component_config - sending GetSystemConfigRequest: {:?}", request);

    match client.get_config(request).await { // Changed to get_config
        Ok(response) => {
            println!("gRPC: get_component_config - received GetSystemConfigResponse");
            let system_config = response.into_inner().config.ok_or_else(|| {
                "Server response did not contain SystemConfig data".to_string()
            })?;

            let target_collector_config = system_config.collectors.get(&collector_id).ok_or_else(|| {
                format!("No collector found with ID '{}' in SystemConfig", collector_id)
            })?;

            println!("gRPC: get_component_config - using collector config for '{}': {:?}", collector_id, target_collector_config);

            let component_value = match component_type.to_lowercase().as_str() {
                "screen" => serde_json::to_value(target_collector_config.screen.as_ref())
                    .map_err(|e| format!("Failed to serialize screen config: {}", e))?,
                "camera" => serde_json::to_value(target_collector_config.camera.as_ref())
                    .map_err(|e| format!("Failed to serialize camera config: {}", e))?,
                "microphone" => serde_json::to_value(target_collector_config.microphone.as_ref())
                    .map_err(|e| format!("Failed to serialize microphone config: {}", e))?,
                "processes" => serde_json::to_value(target_collector_config.processes.as_ref())
                    .map_err(|e| format!("Failed to serialize processes config: {}", e))?,
                "hyprland" => serde_json::to_value(target_collector_config.hyprland.as_ref())
                    .map_err(|e| format!("Failed to serialize hyprland config: {}", e))?,
                _ => return Err(format!("Unknown component type '{}' for config lookup", component_type)),
            };
            println!("gRPC: get_component_config - returning component '{}' from collector '{}' value: {:?}", component_type, collector_id, component_value);
            Ok(component_value)
        }
        Err(e) => {
            println!("gRPC: get_component_config: error from server (get_config call): {:?}", e);
            Err(format!("ServerService GetSystemConfig gRPC request failed: {}", e))
        }
    }
}

#[tauri::command]
async fn set_component_config(collector_id: String, component_type: String, config_value: Value) -> Result<(), String> {
    println!("gRPC: set_component_config - connecting to {}", GRPC_SERVER_ADDRESS);
    println!("Attempting to set config for collector '{}', component type '{}' with data: {:?}", collector_id, component_type, config_value);
    let channel = Channel::from_static(GRPC_SERVER_ADDRESS)
        .connect()
        .await
        .map_err(|e| format!("Failed to connect to gRPC server: {}", e))?;
    println!("gRPC: set_component_config - connection established");
    let mut client = lifelog::LifelogServerServiceClient::new(channel);

    let get_request = tonic::Request::new(lifelog::GetSystemConfigRequest {});
    let get_response = client
        .get_config(get_request)
        .await
        .map_err(|e| format!("Failed to get current SystemConfig: {}", e))?;
    println!("gRPC: set_component_config - RPC get_config succeeded: {:?}", get_response);
    let mut system_config = get_response.into_inner().config.ok_or_else(|| {
        "Server response did not contain SystemConfig data".to_string()
    })?;

    let mut target_collector_config = system_config.collectors.get(&collector_id).cloned()
        .unwrap_or_else(|| {
            println!("gRPC: set_component_config - No existing collector config found for ID '{}'. Creating new default.", collector_id);
            lifelog::CollectorConfig {
                id: collector_id.clone(),
                ..Default::default()
            }
        });

    println!("gRPC: set_component_config - starting with CollectorConfig for ID '{}': {:?}", collector_id, target_collector_config);

    match component_type.to_lowercase().as_str() {
        "screen" => {
            let local_conf: config::ScreenConfig = serde_json::from_value(config_value)
                .map_err(|e| format!("Invalid screen config format: {}", e))?;
            target_collector_config.screen = Some(lifelog::ScreenConfig {
                enabled: local_conf.enabled,
                interval: local_conf.interval,
                output_dir: local_conf.output_dir.to_string_lossy().into_owned(), 
                program: local_conf.program,
                timestamp_format: local_conf.timestamp_format,
            });
        }
        "camera" => {
            let local_conf: config::CameraConfig = serde_json::from_value(config_value)
                .map_err(|e| format!("Invalid camera config format: {}", e))?;
            target_collector_config.camera = Some(lifelog::CameraConfig {
                enabled: local_conf.enabled,
                interval: local_conf.interval,
                output_dir: local_conf.output_dir.to_string_lossy().into_owned(),
                device: local_conf.device,
                resolution_x: local_conf.resolution_x,
                resolution_y: local_conf.resolution_y,
                fps: local_conf.fps,
                timestamp_format: local_conf.timestamp_format,
            });
        }
        "microphone" => {
            let local_conf: config::MicrophoneConfig = serde_json::from_value(config_value)
                .map_err(|e| format!("Invalid microphone config format: {}", e))?;
            target_collector_config.microphone = Some(lifelog::MicrophoneConfig {
                enabled: local_conf.enabled,
                output_dir: local_conf.output_dir.to_string_lossy().into_owned(),
                chunk_duration_secs: local_conf.chunk_duration_secs,
                capture_interval_secs: local_conf.capture_interval_secs,
                timestamp_format: local_conf.timestamp_format,
                sample_rate: local_conf.sample_rate,
                bits_per_sample: local_conf.bits_per_sample,
                channels: local_conf.channels,
            });
        }
        "processes" => {
            let local_conf: config::ProcessesConfig = serde_json::from_value(config_value)
                .map_err(|e| format!("Invalid processes config format: {}", e))?;
            target_collector_config.processes = Some(lifelog::ProcessesConfig {
                enabled: local_conf.enabled,
                interval: local_conf.interval,
                output_dir: local_conf.output_dir.to_string_lossy().into_owned(),
            });
        }
        "hyprland" => {
            let local_conf: config::HyprlandConfig = serde_json::from_value(config_value)
                .map_err(|e| format!("Invalid hyprland config format: {}", e))?;
            target_collector_config.hyprland = Some(lifelog::HyprlandConfig {
                enabled: local_conf.enabled,
                interval: local_conf.interval,
                output_dir: local_conf.output_dir.to_string_lossy().into_owned(),
                log_active_monitor: local_conf.log_active_monitor,
                log_activewindow: local_conf.log_activewindow,
                log_workspace: local_conf.log_workspace,
                log_clients: local_conf.log_clients,
                log_devices: local_conf.log_devices,
            });
        }
        _ => return Err(format!("Unknown component type '{}' for setting config", component_type)),
    }
    
    println!("gRPC: set_component_config - sending modified CollectorConfig (via SetSystemConfigRequest) for ID '{}': {:?}", collector_id, target_collector_config);
    
    let set_request = tonic::Request::new(lifelog::SetSystemConfigRequest {
        config: Some(target_collector_config.clone()), 
    });
    let set_response = client
        .set_config(set_request)
        .await
        .map_err(|e| format!("ServerService SetConfig gRPC request failed: {}", e))?;
    println!("gRPC: set_component_config - RPC set_config succeeded: {:?}", set_response);
    let success_flag = set_response.into_inner().success;
    if !success_flag {
        return Err("Server failed to apply the new configuration.".to_string());
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LifelogDataKeyWrapper {
    uuid: String,
    origin: String,
}

impl From<lifelog::LifelogDataKey> for LifelogDataKeyWrapper {
    fn from(key: lifelog::LifelogDataKey) -> Self {
        Self {
            uuid: key.uuid,
            origin: key.origin,
        }
    }
}
impl From<LifelogDataKeyWrapper> for lifelog::LifelogDataKey {
    fn from(wrapper: LifelogDataKeyWrapper) -> Self {
        Self {
            uuid: wrapper.uuid,
            origin: wrapper.origin,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct ScreenFrameWrapper {
    uuid: String,
    timestamp: Option<i64>, 
    width: u32,
    height: u32,
    dataUrl: String,
    mime_type: String,
}

async fn query_screenshot_keys_async(
    grpc_client: &mut lifelog::LifelogServerServiceClient<Channel>,
    collector_id: Option<String>,
) -> Result<Vec<LifelogDataKeyWrapper>, String> {
    println!("[TAURI] query_screenshot_keys_async: received collector_id: {:?}", collector_id);

    let mut query_message = lifelog::Query {
        search_origins: Vec::new(),
        return_origins: Vec::new(),
        time_ranges: Vec::new(),
        image_embedding: None,
        text_embedding: None,
        text: Vec::new(),
    };

    if let Some(id_str) = collector_id.as_deref() {
        if !id_str.is_empty() && id_str != "undefined" && id_str != "null" {
            let origin_string = format!("{}:{}", id_str, lifelog::DataModality::Screen.as_str_name());
            query_message.search_origins = vec![origin_string.clone()];
            println!("[TAURI] query_screenshot_keys_async: constructed search_origins: {:?}, return_origins: [] (simplified)", query_message.search_origins);
        } else {
            println!("[TAURI] query_screenshot_keys_async: collector_id was present but empty or invalid ('{}'), not setting specific origins.", id_str);
        }
    } else {
        println!("[TAURI] query_screenshot_keys_async: no collector_id provided, searching all screen origins.");
    }

    let request = tonic::Request::new(lifelog::QueryRequest {
        query: Some(query_message),
    });
    println!("[TAURI] query_screenshot_keys_async: sending Simplified QueryRequest: {:?}", request);

    const QUERY_TIMEOUT_SECONDS: u64 = 15;

    println!("[TAURI] query_screenshot_keys_async: Entering timeout block ({}s)...", QUERY_TIMEOUT_SECONDS);
    match timeout(Duration::from_secs(QUERY_TIMEOUT_SECONDS), grpc_client.query(request)).await {
        Ok(inner_result) => {
            println!("[TAURI] query_screenshot_keys_async: Exited gRPC call future (before inner match).");
            match inner_result {
                Ok(response) => {
                    let keys = response.into_inner().keys;
                    println!("[TAURI] query_screenshot_keys_async: received {} keys from server.", keys.len());
                    if !keys.is_empty() {
                        println!("[TAURI] query_screenshot_keys_async: first key: {:?}", keys.first());
                    }
                    let wrapped_keys = keys
                        .into_iter()
                        .map(|k| LifelogDataKeyWrapper {
                            uuid: k.uuid,
                            origin: k.origin,
                        })
                        .collect();
                    Ok(wrapped_keys)
                }
                Err(e) => {
                    println!("[TAURI] query_screenshot_keys_async: error from server (gRPC status): {:?}", e);
                    Err(format!("Failed to query keys from server: {}", e))
                }
            }
        }
        Err(_elapsed_error) => {
            println!("[TAURI] query_screenshot_keys_async: query timed out after {} seconds.", QUERY_TIMEOUT_SECONDS);
            Err(format!("Query to server timed out after {} seconds", QUERY_TIMEOUT_SECONDS))
        }
    }
}

#[tauri::command]
async fn query_screenshot_keys(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, GrpcClientState>,
    collector_id: Option<String>,
) -> Result<Vec<LifelogDataKeyWrapper>, String> {
    println!("[TAURI] query_screenshot_keys: invoked with collector_id: {:?}", collector_id);
    let mut client_guard = state.client.lock().await;
    if let Some(client_instance) = client_guard.as_mut() {
        query_screenshot_keys_async(client_instance, collector_id).await
    } else {
        println!("[TAURI] query_screenshot_keys: gRPC client not initialized trying to reconnect");
        match Channel::from_static(GRPC_SERVER_ADDRESS)
            .connect().await {
            Ok(channel) => {
                let new_client = lifelog::LifelogServerServiceClient::new(channel);
                *client_guard = Some(new_client);
                println!("[TAURI] query_screenshot_keys: Reconnected successfully.");
                query_screenshot_keys_async(client_guard.as_mut().unwrap(), collector_id).await
            }
            Err(e) => {
                println!("[TAURI] query_screenshot_keys: Failed to reconnect: {}", e);
                Err("gRPC client not initialized and failed to reconnect.".to_string())
            }
        }
    }
}

async fn get_screenshots_data_async(
    grpc_client: &mut lifelog::LifelogServerServiceClient<Channel>,
    keys: Vec<LifelogDataKeyWrapper>,
) -> Result<Vec<ScreenFrameWrapper>, String> {
    println!("[TAURI] ENTERED get_screenshots_data_async with {} keys", keys.len());
    if keys.is_empty() {
        println!("[TAURI] get_screenshots_data_async: received empty keys, returning empty vec.");
        return Ok(Vec::new());
    }
    println!("[TAURI] get_screenshots_data_async: received {} total keys to process in batches.", keys.len());

    let batch_size = 1;
    let mut all_screen_frames: Vec<ScreenFrameWrapper> = Vec::new();

    for key_batch_chunk in keys.chunks(batch_size) {
        let current_batch_keys: Vec<lifelog::LifelogDataKey> = key_batch_chunk
            .to_vec()
            .into_iter()
            .map(|k_wrapper| lifelog::LifelogDataKey {
                uuid: k_wrapper.uuid,
                origin: k_wrapper.origin,
            })
            .collect();

        if current_batch_keys.is_empty() {
            continue;   
        }

        println!("[TAURI] get_screenshots_data_async: sending GetDataRequest for a batch of {} keys.", current_batch_keys.len());
        let request = tonic::Request::new(lifelog::GetDataRequest { keys: current_batch_keys });

        const GET_DATA_TIMEOUT_SECONDS: u64 = 15;

        match timeout(Duration::from_secs(GET_DATA_TIMEOUT_SECONDS), grpc_client.get_data(request)).await {
            Ok(inner_result) => { 
                match inner_result {
                    Ok(response) => { 
                        println!("[TAURI] get_screenshots_data_async: successfully received response for batch.");
                        let data_response = response.into_inner();
                        println!("[TAURI] get_screenshots_data_async: received {} data items from server for current batch.", data_response.data.len());

                        let screen_frames_batch: Vec<ScreenFrameWrapper> = data_response
                            .data
                            .into_iter()
                            .filter_map(|lifelog_data| {
                                if let Some(payload) = lifelog_data.payload {
                                    match payload {
                                        lifelog::lifelog_data::Payload::Screenframe(proto_frame) => {
                                            println!("[TAURI] get_screenshots_data_async: processing Screenframe with uuid: {}", 
                                                if proto_frame.uuid.is_empty() { "no_uuid_provided".to_string() } else { proto_frame.uuid.clone() });
                                            
                                            let _timestamp_nanos = proto_frame.timestamp.as_ref().map_or(0, |ts| ts.nanos);
                                            let timestamp_secs = proto_frame.timestamp.as_ref().map_or(0, |ts| ts.seconds);

                                            let temp_uuid_str = proto_frame.uuid.clone();
                                            let uuid = if temp_uuid_str.is_empty() {
                                                eprintln!("[TAURI] Warning: Screenframe had empty UUID string, generating a fallback one.");
                                                format!("empty_uuid_fallback_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default())
                                            } else {
                                                temp_uuid_str
                                            };

                                            Some(ScreenFrameWrapper {
                                                uuid: uuid,
                                                timestamp: Some(timestamp_secs),
                                                width: proto_frame.width as u32,
                                                height: proto_frame.height as u32,
                                                dataUrl: format!(
                                                    "data:{};base64,{}",
                                                    proto_frame.mime_type,
                                                    general_purpose::STANDARD.encode(&proto_frame.image_bytes)
                                                ),
                                                mime_type: proto_frame.mime_type,
                                            })
                                        }
                                        _ => {
                                            println!("[TAURI] get_screenshots_data_async: received non-Screenframe payload in batch, skipping.");
                                            None
                                        },
                                    }
                                } else {
                                    println!("[TAURI] get_screenshots_data_async: received LifelogData with no payload in batch, skipping.");
                                    None
                                }
                            })
                            .collect();
                        all_screen_frames.extend(screen_frames_batch);
                    }
                    Err(e) => { 
                        println!("[TAURI] get_screenshots_data_async: received ERROR for batch (gRPC status): {:?}", e);
                        return Err(format!("Failed to get data for a batch: {}", e)); 
                    }
                }
            }
            Err(_elapsed_error) => { 
                println!("[TAURI] get_screenshots_data_async: get_data for batch timed out after {} seconds.", GET_DATA_TIMEOUT_SECONDS);
                return Err(format!("Fetching data timed out for a batch after {} seconds", GET_DATA_TIMEOUT_SECONDS));
            }
        }
    }
    println!("[TAURI] get_screenshots_data_async: successfully processed all batches, returning {} total screen_frames.", all_screen_frames.len());
    Ok(all_screen_frames)
}

#[tauri::command]
async fn get_screenshots_data(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, GrpcClientState>,
    keys: Vec<LifelogDataKeyWrapper>,
) -> Result<Vec<ScreenFrameWrapper>, String> {
    println!("-----> [TAURI] ENTERED get_screenshots_data command function with {} keys.", keys.len());
    println!("[TAURI] get_screenshots_data: invoked with {} keys.", keys.len());
    let mut client_guard = state.client.lock().await;
    if let Some(client_instance) = client_guard.as_mut() {
        get_screenshots_data_async(client_instance, keys).await
    } else {
        println!("[TAURI] get_screenshots_data: gRPC client not initialized trying to reconnect");
        match Channel::from_static(GRPC_SERVER_ADDRESS)
            .connect().await {
            Ok(channel) => {
                let new_client = lifelog::LifelogServerServiceClient::new(channel);
                *client_guard = Some(new_client);
                println!("[TAURI] get_screenshots_data: Reconnected successfully.");
                get_screenshots_data_async(client_guard.as_mut().unwrap(), keys).await
            }
            Err(e) => {
                println!("[TAURI] get_screenshots_data: Failed to reconnect: {}", e);
                Err("gRPC client not initialized and failed to reconnect.".to_string())
            }
        }
    }
}

#[tauri::command]
async fn get_collector_ids(state: tauri::State<'_, GrpcClientState>) -> Result<Vec<String>, String> {
    println!("[TAURI] get_collector_ids: invoked");
    let mut client_guard = state.client.lock().await;
    println!("[TAURI] get_collector_ids: client lock acquired");

    if let Some(client_instance) = client_guard.as_mut() {
        println!("[TAURI] get_collector_ids: client instance obtained, preparing GetSystemConfigRequest");
        let request = tonic::Request::new(lifelog::GetSystemConfigRequest {});
        println!("[TAURI] get_collector_ids: sending GetSystemConfigRequest to server...");
        match client_instance.get_config(request).await {
            Ok(response) => {
                println!("[TAURI] get_collector_ids: GetConfig response received from server: {:?}", response);
                let inner_response = response.into_inner();
                println!("[TAURI] get_collector_ids: Inner response: {:?}", inner_response);
                let system_config = inner_response.config.ok_or_else(|| {
                    let err_msg = "[TAURI] get_collector_ids: Server response did not contain SystemConfig data".to_string();
                    println!("{}", err_msg);
                    err_msg
                })?;
                println!("[TAURI] get_collector_ids: SystemConfig extracted: {:?}", system_config);
                let collector_ids: Vec<String> = system_config.collectors.keys().cloned().collect();
                println!("[TAURI] get_collector_ids: returning collector IDs: {:?}", collector_ids);
                Ok(collector_ids)
            }
            Err(e) => {
                println!("[TAURI] get_collector_ids: gRPC GetConfig failed: {:?}", e);
                Err(format!("Failed to get collector IDs: {}", e))
            }
        }
    } else {
        println!("[TAURI] get_collector_ids: gRPC client was None after lock.");
        Err("gRPC client not initialized after lock".to_string())
    }
}

#[tokio::main]
async fn main() {
    let config_manager = Mutex::new(config_utils::ConfigManager::new());
    
    let app_state = AppState {
        text_config: Mutex::new(lifelog_interface_lib::config_utils::load_text_upload_config()),
        processes_config: Mutex::new(lifelog_interface_lib::config_utils::load_processes_config()),
        screen_config: Mutex::new(lifelog_interface_lib::config_utils::load_screen_config()),
        api_client: api_client::create_client(),
    };

    let grpc_client_state = GrpcClientState {
        client: Arc::new(tokio::sync::Mutex::new(None)),
    };

    let client_arc_clone = grpc_client_state.client.clone();
    tokio::spawn(async move {
        match Channel::from_static(GRPC_SERVER_ADDRESS)
            .connect().await {
            Ok(channel) => {
                let client = lifelog::LifelogServerServiceClient::new(channel);
                let mut client_guard = client_arc_clone.lock().await;
                *client_guard = Some(client);
                println!("[TAURI_MAIN] Initial gRPC client connection successful.");
            }
            Err(e) => {
                eprintln!("[TAURI_MAIN] Initial gRPC client connection failed: {}", e);
            }
        }
    });

    tauri::Builder::default()
        .manage(app_state)
        .manage(config_manager)
        .manage(grpc_client_state)
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
            get_component_config,
            set_component_config,
            query_screenshot_keys,
            get_screenshots_data,
            get_collector_ids
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
