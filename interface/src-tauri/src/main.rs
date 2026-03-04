// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use base64;
use base64::{engine::general_purpose, Engine as _};
use image::ImageOutputFormat;
use lifelog_interface_lib::{google, lifelog};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::process::Command;
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tonic::transport::Channel;

#[derive(Clone)]
pub struct AuthInterceptor {
    token: String,
}

impl tonic::service::Interceptor for AuthInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        if !self.token.is_empty() {
            let token_val = format!("Bearer {}", self.token);
            if let Ok(m) = tonic::metadata::MetadataValue::try_from(token_val) {
                request.metadata_mut().insert("authorization", m);
            }
        }
        Ok(request)
    }
}

pub type InterceptedClient = lifelog::LifelogServerServiceClient<
    tonic::service::interceptor::InterceptedService<Channel, AuthInterceptor>,
>;

pub struct GrpcClientState {
    client: Arc<tokio::sync::Mutex<Option<InterceptedClient>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InterfaceRuntimeConfig {
    grpc_server_address: String,
    auth_token: Option<String>,
}

#[tauri::command]
async fn initialize_app(
    _window: tauri::Window,
    _app_handle: tauri::AppHandle,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
async fn is_camera_supported() -> bool {
    #[cfg(target_os = "linux")]
    return true;
    #[cfg(not(target_os = "linux"))]
    return false;
}

const DEFAULT_GRPC_SERVER_ADDRESS: &str = "http://localhost:7182";

fn normalize_server_address(raw: &str) -> String {
    let raw = raw.trim();
    if raw.starts_with("http://") || raw.starts_with("https://") {
        raw.to_string()
    } else {
        format!("http://{}", raw)
    }
}

fn interface_config_path() -> Result<PathBuf, String> {
    let config_base = dirs::config_dir()
        .ok_or_else(|| "Could not resolve config directory for this platform".to_string())?;
    Ok(config_base.join("lifelog").join("interface-config.toml"))
}

fn load_interface_runtime_config() -> InterfaceRuntimeConfig {
    if let Ok(path) = interface_config_path() {
        if let Ok(contents) = fs::read_to_string(path) {
            if let Ok(parsed) = toml::from_str::<InterfaceRuntimeConfig>(&contents) {
                return InterfaceRuntimeConfig {
                    grpc_server_address: normalize_server_address(&parsed.grpc_server_address),
                    auth_token: parsed.auth_token,
                };
            }
        }
    }

    let env_addr = std::env::var("LIFELOG_INTERFACE_GRPC_ADDR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|s| normalize_server_address(&s));

    let env_token = std::env::var("LIFELOG_AUTH_TOKEN").ok();

    InterfaceRuntimeConfig {
        grpc_server_address: env_addr.unwrap_or_else(|| DEFAULT_GRPC_SERVER_ADDRESS.to_string()),
        auth_token: env_token,
    }
}

fn save_interface_runtime_config(cfg: &InterfaceRuntimeConfig) -> Result<(), String> {
    let path = interface_config_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| "Invalid interface config path".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create interface config directory: {}", e))?;

    let normalized = InterfaceRuntimeConfig {
        grpc_server_address: normalize_server_address(&cfg.grpc_server_address),
        auth_token: cfg.auth_token.clone(),
    };
    let contents = toml::to_string_pretty(&normalized)
        .map_err(|e| format!("Failed to serialize interface config: {}", e))?;
    fs::write(&path, contents).map_err(|e| format!("Failed to write interface config: {}", e))
}

fn grpc_server_address() -> String {
    load_interface_runtime_config().grpc_server_address
}

fn get_auth_token() -> String {
    load_interface_runtime_config()
        .auth_token
        .unwrap_or_default()
}

async fn create_grpc_channel(addr: &str) -> Result<Channel, tonic::transport::Error> {
    let addr_static: &'static str = Box::leak(addr.to_string().into_boxed_str());
    let mut endpoint = Channel::from_static(addr_static);
    if addr.starts_with("https://") {
        let ca_path = "/home/matth/.config/lifelog/tls/server-ca.pem";
        if let Ok(pem) = std::fs::read_to_string(ca_path) {
            println!("[GRPC] Using CA cert from {}", ca_path);
            let ca = tonic::transport::Certificate::from_pem(pem);
            let tls = tonic::transport::ClientTlsConfig::new()
                .ca_certificate(ca)
                .domain_name("server.matthandzel.com");
            endpoint = endpoint.tls_config(tls)?;
        } else {
            println!("[GRPC] WARNING: CA cert not found, falling back to native roots");
            let tls = tonic::transport::ClientTlsConfig::new().with_native_roots();
            endpoint = endpoint.tls_config(tls)?;
        }
    }
    endpoint.connect().await
}

fn create_client(channel: Channel) -> InterceptedClient {
    let token = get_auth_token();
    lifelog::LifelogServerServiceClient::with_interceptor(channel, AuthInterceptor { token })
}

async fn reconnect_grpc_client(state: &GrpcClientState) -> Result<(), String> {
    let server_addr = grpc_server_address();
    let channel = create_grpc_channel(&server_addr)
        .await
        .map_err(|e| format!("Failed to connect to gRPC server: {}", e))?;
    let new_client = create_client(channel);
    let mut client_guard = state.client.lock().await;
    *client_guard = Some(new_client);
    Ok(())
}

#[tauri::command]
async fn get_interface_settings() -> Result<Value, String> {
    let cfg = load_interface_runtime_config();
    let path = interface_config_path()?;
    Ok(serde_json::json!({
        "grpcServerAddress": cfg.grpc_server_address,
        "configPath": path.to_string_lossy().to_string(),
    }))
}

#[tauri::command]
async fn set_interface_settings(
    grpc_server_address: String,
    state: tauri::State<'_, GrpcClientState>,
) -> Result<(), String> {
    let cfg = InterfaceRuntimeConfig {
        grpc_server_address: normalize_server_address(&grpc_server_address),
        auth_token: Some(get_auth_token()),
    };
    save_interface_runtime_config(&cfg)?;
    reconnect_grpc_client(&state).await
}

#[tauri::command]
async fn test_interface_server_connection(grpc_server_address: String) -> Result<(), String> {
    let addr = normalize_server_address(&grpc_server_address);
    match create_grpc_channel(&addr).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Connection failed: {}", e)),
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct LifelogDataKeyWrapper {
    uuid: String,
    origin: String,
}

impl From<lifelog::LifelogDataKey> for LifelogDataKeyWrapper {
    fn from(k: lifelog::LifelogDataKey) -> Self {
        Self {
            uuid: k.uuid,
            origin: k.origin,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct TimelineEntry {
    uuid: String,
    origin: String,
    modality: String,
    timestamp: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReplayStepWrapper {
    start: Option<i64>,
    end: Option<i64>,
    screen_key: Option<LifelogDataKeyWrapper>,
    context_keys: Vec<LifelogDataKeyWrapper>,
}

#[tauri::command]
async fn get_component_config(
    collector_id: String,
    component_type: String,
) -> Result<Value, String> {
    let server_addr = grpc_server_address();
    let channel = create_grpc_channel(&server_addr)
        .await
        .map_err(|e| format!("Failed to connect to gRPC server: {}", e))?;
    let mut client = create_client(channel);

    let req = lifelog::GetSystemConfigRequest {};
    match client.get_config(req).await {
        Ok(resp) => {
            let config = resp.into_inner().config.ok_or("Missing config")?;
            let collector_cfg_opt = config.collectors.get(&collector_id);

            let component_val = match component_type.to_lowercase().as_str() {
                "browser" => {
                    serde_json::to_value(&collector_cfg_opt.and_then(|c| c.browser.as_ref()))
                }
                "screen" => {
                    serde_json::to_value(&collector_cfg_opt.and_then(|c| c.screen.as_ref()))
                }
                "camera" => {
                    serde_json::to_value(&collector_cfg_opt.and_then(|c| c.camera.as_ref()))
                }
                "microphone" => {
                    serde_json::to_value(&collector_cfg_opt.and_then(|c| c.microphone.as_ref()))
                }
                "processes" => {
                    serde_json::to_value(&collector_cfg_opt.and_then(|c| c.processes.as_ref()))
                }
                "hyprland" => {
                    serde_json::to_value(&collector_cfg_opt.and_then(|c| c.hyprland.as_ref()))
                }
                "weather" => {
                    serde_json::to_value(&collector_cfg_opt.and_then(|c| c.weather.as_ref()))
                }
                "wifi" => serde_json::to_value(&collector_cfg_opt.and_then(|c| c.wifi.as_ref())),
                "clipboard" => {
                    serde_json::to_value(&collector_cfg_opt.and_then(|c| c.clipboard.as_ref()))
                }
                "shell_history" => {
                    serde_json::to_value(&collector_cfg_opt.and_then(|c| c.shell_history.as_ref()))
                }
                "mouse" => serde_json::to_value(&collector_cfg_opt.and_then(|c| c.mouse.as_ref())),
                "window_activity" => serde_json::to_value(
                    &collector_cfg_opt.and_then(|c| c.window_activity.as_ref()),
                ),
                "keyboard" => {
                    serde_json::to_value(&collector_cfg_opt.and_then(|c| c.keyboard.as_ref()))
                }
                _ => return Err(format!("Unknown component type: {}", component_type)),
            };

            component_val.map_err(|e| format!("Failed to serialize component config: {}", e))
        }
        Err(e) => Err(format!("gRPC error: {}", e)),
    }
}

#[tauri::command]
async fn set_component_config(
    _collector_id: String,
    _component_type: String,
    _config_json: String,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
async fn query_screenshot_keys(
    state: tauri::State<'_, GrpcClientState>,
    collector_id: String,
) -> Result<Vec<LifelogDataKeyWrapper>, String> {
    let mut client_guard = state.client.lock().await;
    let client = if let Some(c) = client_guard.as_mut() {
        c
    } else {
        let channel = create_grpc_channel(&grpc_server_address())
            .await
            .map_err(|e| e.to_string())?;
        *client_guard = Some(create_client(channel));
        client_guard.as_mut().unwrap()
    };

    let query = lifelog::Query {
        search_origins: vec![format!("{}:screen", collector_id)],
        return_origins: vec![format!("{}:screen", collector_id)],
        ..Default::default()
    };
    let req = lifelog::QueryRequest { query: Some(query) };
    println!("[GRPC] query_screenshot_keys request: {:?}", req);
    match client.query(req).await {
        Ok(resp) => {
            let keys = resp.into_inner().keys;
            println!(
                "[GRPC] query_screenshot_keys response count: {}",
                keys.len()
            );
            Ok(keys.into_iter().map(Into::into).collect())
        }
        Err(e) => {
            println!("[GRPC] query_screenshot_keys error: {}", e);
            Err(format!("gRPC error: {}", e))
        }
    }
}

#[tauri::command]
async fn get_screenshots_data(
    state: tauri::State<'_, GrpcClientState>,
    keys: Vec<LifelogDataKeyWrapper>,
) -> Result<Vec<Value>, String> {
    println!(
        "[GRPC] get_screenshots_data requested for {} keys",
        keys.len()
    );
    let grpc_keys = keys
        .into_iter()
        .map(|k| lifelog::LifelogDataKey {
            uuid: k.uuid,
            origin: k.origin,
        })
        .collect();

    let mut client_guard = state.client.lock().await;
    let client = if let Some(c) = client_guard.as_mut() {
        c
    } else {
        let channel = create_grpc_channel(&grpc_server_address())
            .await
            .map_err(|e| e.to_string())?;
        *client_guard = Some(create_client(channel));
        client_guard.as_mut().unwrap()
    };

    let req = lifelog::GetDataRequest { keys: grpc_keys };
    match client.get_data(req).await {
        Ok(resp) => {
            let data = resp.into_inner().data;
            println!("[GRPC] get_screenshots_data response count: {}", data.len());
            let mut results = Vec::new();
            for d in data {
                if let Some(lifelog::lifelog_data::Payload::Screenframe(f)) = d.payload {
                    let base64_image = general_purpose::STANDARD.encode(&f.image_bytes);
                    let data_url = format!("data:image/jpeg;base64,{}", base64_image);
                    results.push(serde_json::json!({
                        "uuid": f.uuid,
                        "width": f.width,
                        "height": f.height,
                        "mime_type": f.mime_type,
                        "dataUrl": data_url,
                        "timestamp": f.timestamp.map(|ts| ts.seconds),
                    }));
                }
            }
            Ok(results)
        }
        Err(e) => {
            println!("[GRPC] get_screenshots_data error: {}", e);
            Err(format!("gRPC error: {}", e))
        }
    }
}

#[tauri::command]
async fn get_collector_ids(
    state: tauri::State<'_, GrpcClientState>,
) -> Result<Vec<String>, String> {
    let mut client_guard = state.client.lock().await;
    let client = if let Some(c) = client_guard.as_mut() {
        c
    } else {
        let channel = create_grpc_channel(&grpc_server_address())
            .await
            .map_err(|e| e.to_string())?;
        *client_guard = Some(create_client(channel));
        client_guard.as_mut().unwrap()
    };

    match client
        .list_modalities(lifelog::ListModalitiesRequest {})
        .await
    {
        Ok(resp) => {
            let mut ids = std::collections::HashSet::new();
            let modalities = resp.into_inner().modalities;
            println!(
                "[GRPC] get_collector_ids: received {} modalities",
                modalities.len()
            );
            for m in modalities {
                let id = m
                    .stream_id
                    .split(':')
                    .next()
                    .unwrap_or("unknown")
                    .to_string();
                println!("[GRPC] get_collector_ids: found collector_id {}", id);
                ids.insert(id);
            }
            Ok(ids.into_iter().collect())
        }
        Err(e) => {
            println!("[GRPC] get_collector_ids error: {}", e);
            Err(format!("Failed to list collectors: {}", e))
        }
    }
}

#[tauri::command]
async fn query_timeline(
    state: tauri::State<'_, GrpcClientState>,
    llql_query: String,
) -> Result<Vec<TimelineEntry>, String> {
    let mut client_guard = state.client.lock().await;
    let client = if let Some(c) = client_guard.as_mut() {
        c
    } else {
        let channel = create_grpc_channel(&grpc_server_address())
            .await
            .map_err(|e| e.to_string())?;
        *client_guard = Some(create_client(channel));
        client_guard.as_mut().unwrap()
    };

    let query = lifelog::Query {
        text: vec![llql_query],
        ..Default::default()
    };
    let req = lifelog::QueryRequest { query: Some(query) };

    match timeout(Duration::from_secs(15), client.query(req)).await {
        Ok(Ok(resp)) => {
            let entries = resp
                .into_inner()
                .keys
                .into_iter()
                .map(|k| {
                    let modality = k.origin.split(':').last().unwrap_or("unknown").to_string();
                    TimelineEntry {
                        uuid: k.uuid,
                        origin: k.origin,
                        modality,
                        timestamp: None,
                    }
                })
                .collect();
            Ok(entries)
        }
        Ok(Err(e)) => Err(format!("Query failed: {}", e)),
        Err(_) => Err("Query timed out".to_string()),
    }
}

async fn replay_async(
    grpc_client: &mut InterceptedClient,
    screen_origin: Option<String>,
    context_origins: Option<Vec<String>>,
    start_time: i64,
    end_time: i64,
    max_steps: Option<u32>,
    max_context_per_step: Option<u32>,
    context_pad_ms: Option<u32>,
) -> Result<Vec<ReplayStepWrapper>, String> {
    let window = lifelog::Timerange {
        start: Some(google::protobuf::Timestamp {
            seconds: start_time,
            nanos: 0,
        }),
        end: Some(google::protobuf::Timestamp {
            seconds: end_time,
            nanos: 0,
        }),
    };

    let req = lifelog::ReplayRequest {
        screen_origin: screen_origin.unwrap_or_default(),
        window: Some(window),
        context_origins: context_origins.unwrap_or_default(),
        max_steps: max_steps.unwrap_or(0),
        max_context_per_step: max_context_per_step.unwrap_or(0),
        context_pad_ms: context_pad_ms.unwrap_or(0) as u64,
    };

    match timeout(Duration::from_secs(20), grpc_client.replay(req)).await {
        Ok(Ok(resp)) => {
            let wrapped = resp
                .into_inner()
                .steps
                .into_iter()
                .map(|s| ReplayStepWrapper {
                    start: s.start.map(|ts| ts.seconds),
                    end: s.end.map(|ts| ts.seconds),
                    screen_key: s.screen_key.map(|k| LifelogDataKeyWrapper {
                        uuid: k.uuid,
                        origin: k.origin,
                    }),
                    context_keys: s
                        .context_keys
                        .into_iter()
                        .map(|k| LifelogDataKeyWrapper {
                            uuid: k.uuid,
                            origin: k.origin,
                        })
                        .collect(),
                })
                .collect();
            Ok(wrapped)
        }
        Ok(Err(e)) => Err(format!("Replay failed: {}", e)),
        Err(_) => Err("Replay timed out".to_string()),
    }
}

#[tauri::command]
async fn replay(
    state: tauri::State<'_, GrpcClientState>,
    screen_origin: Option<String>,
    context_origins: Option<Vec<String>>,
    start_time: i64,
    end_time: i64,
    max_steps: Option<u32>,
    max_context_per_step: Option<u32>,
    context_pad_ms: Option<u32>,
) -> Result<Vec<ReplayStepWrapper>, String> {
    let mut client_guard = state.client.lock().await;
    let client = if let Some(c) = client_guard.as_mut() {
        c
    } else {
        let server_addr = grpc_server_address();
        let channel = create_grpc_channel(&server_addr)
            .await
            .map_err(|e| format!("Failed to connect: {}", e))?;
        let new_client = create_client(channel);
        *client_guard = Some(new_client);
        client_guard.as_mut().unwrap()
    };

    replay_async(
        client,
        screen_origin,
        context_origins,
        start_time,
        end_time,
        max_steps,
        max_context_per_step,
        context_pad_ms,
    )
    .await
}

#[tauri::command]
async fn select_file_dialog() -> Result<Option<String>, String> {
    Ok(None)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProcessInfoWrapper {
    pid: i32,
    ppid: i32,
    name: String,
    exe: String,
    cmdline: String,
    status: String,
    cpu_usage: f64,
    memory_usage: i64,
    threads: i32,
    user: String,
    start_time: f64,
}

impl From<lifelog::ProcessInfo> for ProcessInfoWrapper {
    fn from(p: lifelog::ProcessInfo) -> Self {
        Self {
            pid: p.pid,
            ppid: p.ppid,
            name: p.name,
            exe: p.exe,
            cmdline: p.cmdline,
            status: p.status,
            cpu_usage: p.cpu_usage,
            memory_usage: p.memory_usage,
            threads: p.threads,
            user: p.user,
            start_time: p.start_time,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct FrameDataWrapper {
    uuid: String,
    modality: String,
    timestamp: Option<i64>,
    text: Option<String>,
    url: Option<String>,
    title: Option<String>,
    visit_count: Option<u32>,
    command: Option<String>,
    working_dir: Option<String>,
    exit_code: Option<i32>,
    application: Option<String>,
    window_title: Option<String>,
    duration_secs: Option<f32>,
    audio_data_url: Option<String>,
    codec: Option<String>,
    sample_rate: Option<u32>,
    channels: Option<u32>,
    audio_duration_secs: Option<f32>,
    dataUrl: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    mime_type: Option<String>,
    camera_device: Option<String>,
    processes: Option<Vec<ProcessInfoWrapper>>,
}

fn encode_dataUrl(
    image_bytes: &[u8],
    _mime_type: &str,
    thumbnail_mode: bool,
) -> Result<String, String> {
    let dynamic_image =
        image::load_from_memory(image_bytes).map_err(|e| format!("Decode error: {}", e))?;
    let img = if thumbnail_mode {
        dynamic_image.thumbnail(512, 288)
    } else {
        dynamic_image
    };
    let mut encoded = Cursor::new(Vec::new());
    img.write_to(&mut encoded, ImageOutputFormat::Jpeg(75))
        .map_err(|e| format!("Encode error: {}", e))?;
    Ok(format!(
        "data:image/jpeg;base64,{}",
        general_purpose::STANDARD.encode(encoded.into_inner())
    ))
}

async fn get_frame_data_async(
    grpc_client: &mut InterceptedClient,
    keys: Vec<LifelogDataKeyWrapper>,
    thumbnail_mode: bool,
) -> Result<Vec<FrameDataWrapper>, String> {
    let grpc_keys = keys
        .into_iter()
        .map(|k| lifelog::LifelogDataKey {
            uuid: k.uuid,
            origin: k.origin,
        })
        .collect();
    let req = lifelog::GetDataRequest { keys: grpc_keys };
    match timeout(Duration::from_secs(15), grpc_client.get_data(req)).await {
        Ok(Ok(resp)) => {
            let mut frames = Vec::new();
            for d in resp.into_inner().data {
                if let Some(payload) = d.payload {
                    let frame = match payload {
                        lifelog::lifelog_data::Payload::Screenframe(f) => FrameDataWrapper {
                            uuid: f.uuid,
                            modality: "Screen".into(),
                            timestamp: f.timestamp.map(|ts| ts.seconds),
                            dataUrl: Some(encode_dataUrl(
                                &f.image_bytes,
                                &f.mime_type,
                                thumbnail_mode,
                            )?),
                            width: Some(f.width),
                            height: Some(f.height),
                            mime_type: Some(f.mime_type),
                            ..Default::default()
                        },
                        _ => FrameDataWrapper::default(),
                    };
                    frames.push(frame);
                }
            }
            Ok(frames)
        }
        _ => Err("GetData failed".into()),
    }
}

#[tauri::command]
async fn get_frame_data(
    state: tauri::State<'_, GrpcClientState>,
    keys: Vec<LifelogDataKeyWrapper>,
) -> Result<Vec<FrameDataWrapper>, String> {
    let mut client_guard = state.client.lock().await;
    let client = if let Some(c) = client_guard.as_mut() {
        c
    } else {
        let channel = create_grpc_channel(&grpc_server_address())
            .await
            .map_err(|e| e.to_string())?;
        *client_guard = Some(create_client(channel));
        client_guard.as_mut().unwrap()
    };
    get_frame_data_async(client, keys, false).await
}

#[tauri::command]
async fn get_frame_data_thumbnails(
    state: tauri::State<'_, GrpcClientState>,
    keys: Vec<LifelogDataKeyWrapper>,
) -> Result<Vec<FrameDataWrapper>, String> {
    let mut client_guard = state.client.lock().await;
    let client = if let Some(c) = client_guard.as_mut() {
        c
    } else {
        let channel = create_grpc_channel(&grpc_server_address())
            .await
            .map_err(|e| e.to_string())?;
        *client_guard = Some(create_client(channel));
        client_guard.as_mut().unwrap()
    };
    get_frame_data_async(client, keys, true).await
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CollectorStateWrapper {
    collector_id: String,
    name: String,
    last_seen_secs: Option<i64>,
    total_buffer_size: u32,
    upload_lag_bytes: u64,
    last_upload_time_secs: Option<i64>,
    source_states: Vec<String>,
    source_buffer_sizes: Vec<String>,
}

#[tauri::command]
async fn get_system_state(
    state: tauri::State<'_, GrpcClientState>,
) -> Result<Vec<CollectorStateWrapper>, String> {
    let mut client_guard = state.client.lock().await;
    let client = if let Some(c) = client_guard.as_mut() {
        c
    } else {
        let channel = create_grpc_channel(&grpc_server_address())
            .await
            .map_err(|e| e.to_string())?;
        *client_guard = Some(create_client(channel));
        client_guard.as_mut().unwrap()
    };
    match timeout(
        Duration::from_secs(10),
        client.get_state(lifelog::GetStateRequest {}),
    )
    .await
    {
        Ok(Ok(resp)) => {
            let state = resp.into_inner().state.unwrap_or_default();
            Ok(state
                .collector_states
                .into_iter()
                .map(|(id, cs)| CollectorStateWrapper {
                    collector_id: id,
                    name: cs.name,
                    last_seen_secs: cs.last_seen.map(|ts| ts.seconds),
                    total_buffer_size: cs.total_buffer_size,
                    upload_lag_bytes: cs.upload_lag_bytes,
                    last_upload_time_secs: cs.last_upload_time.map(|ts| ts.seconds),
                    source_states: cs.source_states,
                    source_buffer_sizes: cs.source_buffer_sizes,
                })
                .collect())
        }
        _ => Err("GetState failed".into()),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let grpc_client_state = GrpcClientState {
        client: Arc::new(tokio::sync::Mutex::new(None)),
    };
    let client_arc_clone = grpc_client_state.client.clone();
    tokio::spawn(async move {
        if let Ok(channel) = create_grpc_channel(&grpc_server_address()).await {
            let mut client_guard = client_arc_clone.lock().await;
            *client_guard = Some(create_client(channel));
        }
    });

    tauri::Builder::default()
        .manage(grpc_client_state)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            initialize_app,
            is_camera_supported,
            get_interface_settings,
            set_interface_settings,
            test_interface_server_connection,
            get_component_config,
            set_component_config,
            query_screenshot_keys,
            get_screenshots_data,
            get_collector_ids,
            query_timeline,
            replay,
            get_frame_data,
            get_frame_data_thumbnails,
            get_system_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
