#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use config::{ScreenConfig, MicrophoneConfig, ProcessesConfig, TextUploadConfig};
use lifelog_interface_lib::config_utils;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use tauri::State;
use serde_json::Value;

// AppState to store configurations
pub struct AppState {
    text_config: Mutex<TextUploadConfig>,
    processes_config: Mutex<ProcessesConfig>,
    screen_config: Mutex<ScreenConfig>,
}

// Helper function to get component config via gRPC
async fn get_grpc_component_config(component_name: &str) -> Result<Value, String> {
    println!("Attempting to get config for {} via gRPC", component_name);
    
    let output = Command::new("grpcurl")
        .args([
            "-plaintext",
            "-d", "{}",
            "127.0.0.1:7182",
            "lifelog.LifelogServerService/GetConfig"
        ])
        .output()
        .map_err(|e| format!("Failed to execute gRPC client: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("gRPC client error: {}", stderr);
        return Err(format!("gRPC client error: {}", stderr));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("gRPC raw response: {}", stdout);
    
    let response: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse gRPC response: {}", e))?;
    
    println!("gRPC response parsed: {:?}", response);
    
    // Extract the specific component config from the full response
    let result = match component_name.to_lowercase().as_str() {
        "screen" => {
            response.get("config")
                .and_then(|c| c.get("collector"))
                .and_then(|c| c.get("screen"))
                .cloned()
                .ok_or_else(|| "No screen config found in response".to_string())
        },
        "camera" => {
            response.get("config")
                .and_then(|c| c.get("collector"))
                .and_then(|c| c.get("camera"))
                .cloned()
                .ok_or_else(|| "No camera config found in response".to_string())
        },
        "microphone" => {
            response.get("config")
                .and_then(|c| c.get("collector"))
                .and_then(|c| c.get("microphone"))
                .cloned()
                .ok_or_else(|| "No microphone config found in response".to_string())
        },
        "processes" => {
            response.get("config")
                .and_then(|c| c.get("collector"))
                .and_then(|c| c.get("processes"))
                .cloned()
                .ok_or_else(|| "No processes config found in response".to_string())
        },
        _ => Err(format!("Unsupported component: {}", component_name)),
    };
    
    // Log the result
    match &result {
        Ok(value) => println!("Successfully extracted config for {}: {:?}", component_name, value),
        Err(e) => println!("Failed to extract config for {}: {}", component_name, e),
    }
    
    result
}

// Helper function to get local component config
fn get_local_component_config(component_name: &str) -> Result<Value, String> {
    println!("Using local config for component: {}", component_name);
    
    match component_name.to_lowercase().as_str() {
        "screen" => {
            let screen_config = lifelog_interface_lib::config_utils::load_screen_config();
            serde_json::to_value(screen_config)
                .map_err(|e| format!("Failed to serialize local screen config: {}", e))
        },
        "camera" => {
            let camera_config = config::CameraConfig {
                enabled: false,
                interval: 60.0,
                output_dir: std::path::PathBuf::from("~/lifelog/camera"),
                device: String::new(),
                fps: 30,
                resolution_x: 1280,
                resolution_y: 720,
                timestamp_format: "%Y-%m-%d_%H-%M-%S".to_string(),
            };
            serde_json::to_value(camera_config)
                .map_err(|e| format!("Failed to serialize local camera config: {}", e))
        },
        "microphone" => {
            let microphone_config = lifelog_interface_lib::config_utils::load_microphone_config();
            serde_json::to_value(microphone_config)
                .map_err(|e| format!("Failed to serialize local microphone config: {}", e))
        },
        "processes" => {
            let processes_config = lifelog_interface_lib::config_utils::load_processes_config();
            serde_json::to_value(processes_config)
                .map_err(|e| format!("Failed to serialize local processes config: {}", e))
        },
        _ => Err(format!("Unsupported component: {}", component_name)),
    }
}

// Command to get component configuration
#[tauri::command]
async fn get_component_config(component_name: String) -> Result<Value, String> {
    println!("Requesting config for component: {}", component_name);
    
    match get_grpc_component_config(&component_name).await {
        Ok(config) => {
            println!("Retrieved config for {} from gRPC server", component_name);
            Ok(config)
        },
        Err(e) => {
            println!("Failed to get config from gRPC, using local fallback: {}", e);
            get_local_component_config(&component_name)
        }
    }
}

// Initialize app command - minimal implementation
#[tauri::command]
async fn initialize_app() -> Result<(), String> {
    println!("App initialized");
    Ok(())
}

fn main() {
    let config_manager = Mutex::new(config_utils::ConfigManager::new());
    
    // Create the application state 
    let app_state = AppState {
        text_config: Mutex::new(lifelog_interface_lib::config_utils::load_text_upload_config()),
        processes_config: Mutex::new(lifelog_interface_lib::config_utils::load_processes_config()),
        screen_config: Mutex::new(lifelog_interface_lib::config_utils::load_screen_config()),
    };

    tauri::Builder::default()
        .manage(app_state)
        .manage(config_manager)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Include only the minimal necessary functions
            initialize_app,
            get_component_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}