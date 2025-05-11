// Auto‐generated types

export interface ActorConfig {
}

export interface AmbientConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
  temperature_sensor_path: string | null;
  humidity_sensor_path: string | null;
}

export interface AudioConfig {
  enabled: boolean;
  output_dir: string;
  sample_rate: number;
  chunk_duration_secs: number;
}

export interface BrowserFrame {
  uuid: string;
  timestamp: Date;
  url: string;
  title: string;
  visit_count: number;
}

export interface BrowserHistoryConfig {
  enabled: boolean;
  input_file: string;
  output_file: string;
  browser_type: string;
}

export type BrowserHistoryType = "Chrome" | "Firefox";

export interface CameraConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
  device: string;
  resolution_x: number;
  resolution_y: number;
  fps: number;
  timestamp_format: string;
}

export interface CollectorConfig {
  id: string;
  host: string;
  port: number;
  timestamp_format: string;
  browser: BrowserHistoryConfig;
  screen: ScreenConfig;
  camera: CameraConfig;
  microphone: MicrophoneConfig;
  processes: ProcessesConfig;
  hyprland: HyprlandConfig;
}

export interface CollectorState {
  name: string;
  timestamp: Date;
  source_states: string[];
  source_buffer_sizes: string[];
  total_buffer_size: number;
}

export type DataModality = "Browser" | "Ocr" | "Screen";

export interface GeoConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
  use_ip_fallback: boolean;
}

export interface HyprlandConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
  log_clients: boolean;
  log_activewindow: boolean;
  log_workspace: boolean;
  log_active_monitor: boolean;
  log_devices: boolean;
}

export interface InputLoggerConfig {
  output_dir: string;
  enabled: boolean;
  log_keyboard: boolean;
  log_mouse_buttons: boolean;
  log_mouse_movement: boolean;
  log_mouse_wheel: boolean;
  log_devices: boolean;
  mouse_interval: number;
}

export interface InterfaceState {
}

export interface KeyboardConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
}

export interface MicrophoneConfig {
  enabled: boolean;
  output_dir: string;
  sample_rate: number;
  chunk_duration_secs: number;
  timestamp_format: string;
  bits_per_sample: number;
  channels: number;
  capture_interval_secs: number;
}

export interface MouseConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
}

export interface NetworkConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
}

export interface OcrFrame {
  uuid: string;
  timestamp: Date;
  text: string;
}

export interface ProcessesConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
}

export interface ScreenConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
  program: string;
  timestamp_format: string;
}

export interface ScreenFrame {
  uuid: string;
  timestamp: Date;
  width: number;
  height: number;
  image_bytes: Uint8Array;
  mime_type: string;
}

export type ServerActionType = "Querying" | "GetData" | "SyncData" | "TransformData" | "CompressData" | "RegisterActor";

export interface ServerConfig {
  host: string;
  port: number;
  database_endpoint: string;
  database_name: string;
  server_name: string;
}

export interface ServerState {
  name: string;
  timestamp: Date;
  cpu_usage: number;
  memory_usage: number;
  threads: number;
  timestamp_of_last_sync: Date;
  pending_actions: any[];
}

export interface SystemConfig {
  server: ServerConfig;
  collector: CollectorConfig;
}

export interface SystemPerformanceConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
  log_cpu: boolean;
  log_memory: boolean;
  log_disk: boolean;
}

export interface SystemState {
  collector_states: Record<string, any>;
  interface_states: Record<string, any>;
  server_state: ServerState;
}

export interface TextUploadConfig {
  enabled: boolean;
  output_dir: string;
  max_file_size_mb: number;
  supported_formats: string[];
}

export interface WeatherConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
  api_key: string;
  latitude: number;
  longitude: number;
}

export interface WifiConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
  scan_command: string;
}

