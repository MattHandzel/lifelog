// Auto-generated types

export interface CameraConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
  device: string;
  resolution: Resolution;
  fps: number;
  timestamp_format: string;
}

export interface TextUploadConfig {
  enabled: boolean;
  output_dir: any;
  max_file_size_mb: number;
  supported_formats: string[];
}

export interface NetworkConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
}

export interface Config {
  timestamp_format: string;
  screen: ScreenConfig;
  camera: CameraConfig;
  microphone: MicrophoneConfig;
  processes: ProcessesConfig;
  hyprland: HyprlandConfig;
}

export interface ScreenConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
  program: string;
  timestamp_format: string;
}

export interface GeoConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
  use_ip_fallback: boolean;
}

export interface InputLoggerConfig {
  output_dir: any;
  enabled: boolean;
  log_keyboard: boolean;
  log_mouse_buttons: boolean;
  log_mouse_movement: boolean;
  log_mouse_wheel: boolean;
  log_devices: boolean;
  mouse_interval: number;
}

export interface AudioConfig {
  enabled: boolean;
  output_dir: any;
  sample_rate: number;
  chunk_duration_secs: number;
}

export interface KeyboardConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
}

export interface WeatherConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
  api_key: string;
  latitude: number;
  longitude: number;
}

export interface Resolution {
  width: number;
  height: number;
}

export interface WifiConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
  scan_command: string;
}

export interface ScreenFrame {
  uuid: any;
  timestamp: any;
  image_path: string;
  resolution: any;
}

export interface ServerState {
  name: string;
  timestamp: Date;
}

export interface AmbientConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
  temperature_sensor_path: string | null;
  humidity_sensor_path: string | null;
}

export interface ProcessesConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
}

export interface SystemPerformanceConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
  log_cpu: boolean;
  log_memory: boolean;
  log_disk: boolean;
}

export interface FrameMetadata {
  uuid: any;
  timestamp: any;
  dpi: number;
  color_depth: number;
  contains_sensitive: boolean | null;
}

export interface CollectorState {
  name: string;
  timestamp: Date;
}

export interface InterfaceState {
}

export interface HyprlandConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
  log_clients: boolean;
  log_activewindow: boolean;
  log_workspace: boolean;
  log_active_monitor: boolean;
  log_devices: boolean;
}

export interface MicrophoneConfig {
  enabled: boolean;
  output_dir: any;
  sample_rate: number;
  chunk_duration_secs: number;
  timestamp_format: string;
  bits_per_sample: number;
  channels: number;
  capture_interval_secs: number;
}

export interface ServerConfig {
  host: string;
  port: number;
  database_path: string;
  database_name: string;
}

export interface MouseConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
}

