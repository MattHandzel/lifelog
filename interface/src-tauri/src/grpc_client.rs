use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::timeout;
use tonic::transport::{Channel, Endpoint};
use serde_json::Value;
use crate::lifelog;
use crate::lifelog::lifelog_server_service_client::LifelogServerServiceClient;

pub const GRPC_SERVER_ADDRESS: &str = "http://127.0.0.1:7182";

const CACHE_TIMEOUT_SECS: u64 = 30;

pub struct ConfigCache {
    last_updated: Instant,
    configs: std::collections::HashMap<String, Value>,
}

pub struct GrpcClient {
    channel: Channel,
    client: LifelogServerServiceClient<Channel>,
    cache: Arc<Mutex<Option<ConfigCache>>>,
}

impl GrpcClient {
    pub async fn new() -> Result<Self, String> {
        let endpoint = Endpoint::from_shared(GRPC_SERVER_ADDRESS.to_string())
            .map_err(|e| format!("Invalid endpoint URL: {}", e))?;
        
        let connect_result = timeout(
            Duration::from_secs(2),
            endpoint.connect()
        ).await;
        
        let channel = match connect_result {
            Ok(result) => result.map_err(|e| format!("Failed to connect: {}", e))?,
            Err(_) => return Err("Connection timed out".to_string()),
        };
        
        let client = LifelogServerServiceClient::new(channel.clone());
        let cache = Arc::new(Mutex::new(None));
        
        Ok(Self {
            channel,
            client,
            cache,
        })
    }
    
    pub async fn get_config(&self, component_name: &str) -> Result<Value, String> {
        // Check cache first
        {
            let cache_guard = self.cache.lock().await;
            if let Some(ref cache_data) = *cache_guard {
                // Cache valid for 30 seconds
                if cache_data.last_updated.elapsed() < Duration::from_secs(CACHE_TIMEOUT_SECS) {
                    if let Some(config) = cache_data.configs.get(component_name) {
                        println!("Using cached config for {}", component_name);
                        return Ok(config.clone());
                    }
                }
            }
        }
        
        // No valid cache, get from server
        let request = tonic::Request::new(lifelog::GetSystemConfigRequest {});
        
        let response_result = timeout(
            Duration::from_secs(5),
            self.client.clone().get_config(request)
        ).await;
        
        let response = match response_result {
            Ok(result) => result.map_err(|e| format!("gRPC GetConfig error: {}", e))?.into_inner(),
            Err(_) => return Err("gRPC call timed out".to_string()),
        };
        
        let collector_config = response.config.ok_or_else(|| "No config returned from server".to_string())?
            .collector.ok_or_else(|| "No collector config returned from server".to_string())?;
            
        let mut new_cache = std::collections::HashMap::new();
        
        if let Some(screen_config) = &collector_config.screen {
            if let Ok(value) = serde_json::to_value(screen_config) {
                new_cache.insert("screen".to_string(), value.clone());
                if component_name == "screen" {
                    self.update_cache(new_cache).await;
                    return Ok(value);
                }
            }
        }
        
        if let Some(camera_config) = &collector_config.camera {
            if let Ok(value) = serde_json::to_value(camera_config) {
                new_cache.insert("camera".to_string(), value.clone());
                if component_name == "camera" {
                    self.update_cache(new_cache).await;
                    return Ok(value);
                }
            }
        }
        
        if let Some(microphone_config) = &collector_config.microphone {
            if let Ok(value) = serde_json::to_value(microphone_config) {
                new_cache.insert("microphone".to_string(), value.clone());
                if component_name == "microphone" {
                    self.update_cache(new_cache).await;
                    return Ok(value);
                }
            }
        }
        
        if let Some(processes_config) = &collector_config.processes {
            if let Ok(value) = serde_json::to_value(processes_config) {
                new_cache.insert("processes".to_string(), value.clone());
                if component_name == "processes" {
                    self.update_cache(new_cache).await;
                    return Ok(value);
                }
            }
        }
        
        // Update cache even if requested component wasn't found
        self.update_cache(new_cache).await;
        
        Err(format!("Component {} not found in server response", component_name))
    }
    
    pub async fn set_config(&self, component_name: &str, config_value: &Value) -> Result<(), String> {
        // First get current config
        let request = tonic::Request::new(lifelog::GetSystemConfigRequest {});
        
        let response_result = timeout(
            Duration::from_secs(5),
            self.client.clone().get_config(request)
        ).await;
        
        let get_response = match response_result {
            Ok(result) => result.map_err(|e| format!("gRPC GetConfig error: {}", e))?.into_inner(),
            Err(_) => return Err("gRPC call timed out".to_string()),
        };
        
        let mut system_config = get_response.config
            .ok_or_else(|| "No config returned from server".to_string())?;
            
        if system_config.collector.is_none() {
            return Err("No collector config in system config".to_string());
        }
        
        let mut collector_config = system_config.collector.unwrap();
        
        match component_name.to_lowercase().as_str() {
            "screen" => {
                let screen_config: lifelog::ScreenConfig = serde_json::from_value(config_value.clone())
                    .map_err(|e| format!("Failed to deserialize screen config: {}", e))?;
                collector_config.screen = Some(screen_config);
            },
            "camera" => {
                let camera_config: lifelog::CameraConfig = serde_json::from_value(config_value.clone())
                    .map_err(|e| format!("Failed to deserialize camera config: {}", e))?;
                collector_config.camera = Some(camera_config);
            },
            "microphone" => {
                let microphone_config: lifelog::MicrophoneConfig = serde_json::from_value(config_value.clone())
                    .map_err(|e| format!("Failed to deserialize microphone config: {}", e))?;
                collector_config.microphone = Some(microphone_config);
            },
            "processes" => {
                let processes_config: lifelog::ProcessesConfig = serde_json::from_value(config_value.clone())
                    .map_err(|e| format!("Failed to deserialize processes config: {}", e))?;
                collector_config.processes = Some(processes_config);
            },
            _ => return Err(format!("Unsupported component: {}", component_name)),
        };
        
        system_config.collector = Some(collector_config);
        
        let set_request = tonic::Request::new(lifelog::SetSystemConfigRequest {
            config: Some(system_config),
        });
        
        let set_response_result = timeout(
            Duration::from_secs(5),
            self.client.clone().set_config(set_request)
        ).await;
        
        let set_response = match set_response_result {
            Ok(result) => result.map_err(|e| format!("gRPC SetConfig error: {}", e))?.into_inner(),
            Err(_) => return Err("gRPC set_config call timed out".to_string()),
        };
        
        if !set_response.success {
            return Err("Server reported failure in setting config".to_string());
        }
        
        self.invalidate_cache().await;
        
        Ok(())
    }
    
    async fn update_cache(&self, new_cache: std::collections::HashMap<String, Value>) {
        let mut cache_guard = self.cache.lock().await;
        *cache_guard = Some(ConfigCache {
            last_updated: Instant::now(),
            configs: new_cache,
        });
    }
    
    pub async fn invalidate_cache(&self) {
        let mut cache_guard = self.cache.lock().await;
        *cache_guard = None;
    }
}

lazy_static::lazy_static! {
    pub static ref GRPC_CLIENT: tokio::sync::Mutex<Option<Arc<GrpcClient>>> = tokio::sync::Mutex::new(None);
}

pub async fn init_grpc_client() -> Result<(), String> {
    let mut client_guard = GRPC_CLIENT.lock().await;
    if client_guard.is_none() {
        match GrpcClient::new().await {
            Ok(client) => {
                *client_guard = Some(Arc::new(client));
                Ok(())
            },
            Err(e) => Err(format!("Failed to initialize gRPC client: {}", e)),
        }
    } else {
        Ok(())
    }
}

pub async fn get_grpc_client() -> Result<Arc<GrpcClient>, String> {
    let client_guard = GRPC_CLIENT.lock().await;
    if let Some(client) = &*client_guard {
        Ok(client.clone())
    } else {
        Err("gRPC client not initialized".to_string())
    }
} 