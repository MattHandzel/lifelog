// Auto‐generated types

export interface CollectorConfig {
  id: string;
  host: string;
  port: number;
  timestamp_format: string;
  screen: ScreenConfig;
  camera: CameraConfig;
  microphone: MicrophoneConfig;
  processes: ProcessesConfig;
  hyprland: HyprlandConfig;
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

export interface TextUploadConfig {
  enabled: boolean;
  output_dir: string;
  max_file_size_mb: number;
  supported_formats: string[];
}

export interface CollectorState {
  name: string;
  timestamp: Date;
}

export type ServerCommand = "RegisterCollector" | "GetConfig" | "SetConfig" | "GetData" | "Query" | "ReportState" | "GetState";

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

export interface SystemConfig {
  server: ServerConfig;
  collector: CollectorConfig;
}

export interface ServerState {
  name: string;
  timestamp: Date;
  cpu_usage: number;
  memory_usage: number;
  threads: number;
  pending_commands: any[];
}

export interface MouseConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
}

export interface ServerConfig {
  host: string;
  port: number;
  database_endpoint: string;
  database_name: string;
  server_name: string;
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

export interface NetworkConfig {
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

export interface ProcessesConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
}

export interface AudioConfig {
  enabled: boolean;
  output_dir: string;
  sample_rate: number;
  chunk_duration_secs: number;
}

export interface KeyboardConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
}

export interface GeoConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
  use_ip_fallback: boolean;
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

export type DataModality = "Screen";

export interface SystemPerformanceConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
  log_cpu: boolean;
  log_memory: boolean;
  log_disk: boolean;
}

export interface ScreenFrame {
  uuid: string;
  timestamp: Date;
  width: number;
  height: number;
  image_bytes: Uint8Array;
  mime_type: string;
}

export interface InterfaceState {
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

export interface AmbientConfig {
  enabled: boolean;
  interval: number;
  output_dir: string;
  temperature_sensor_path: string | null;
  humidity_sensor_path: string | null;
}

