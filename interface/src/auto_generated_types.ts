// Auto-generated types

export interface WifiConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
  scan_command: string;
}

export interface AudioConfig {
  enabled: boolean;
  output_dir: any;
  sample_rate: number;
  chunk_duration_secs: number;
}

export interface Config {
  timestamp_format: string;
  screen: ScreenConfig;
  camera: CameraConfig;
  microphone: MicrophoneConfig;
  network: NetworkConfig;
  processes: ProcessesConfig;
  system_performance: SystemPerformanceConfig;
  ambient: AmbientConfig;
  weather: WeatherConfig;
  audio: AudioConfig;
  geolocation: GeoConfig;
  wifi: WifiConfig;
  hyprland: HyprlandConfig;
  server: ServerConfig;
  input_logger: InputLoggerConfig;
  text_upload: TextUploadConfig;
}

export interface AmbientConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
  temperature_sensor_path: string | null;
  humidity_sensor_path: string | null;
}

export interface NetworkConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
}

export interface TextUploadConfig {
  enabled: boolean;
  output_dir: any;
  max_file_size_mb: number;
  supported_formats: string[];
}

export interface WeatherConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
  api_key: string;
  latitude: number;
  longitude: number;
}

export interface KeyboardConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
}

export interface ServerConfig {
  host: string;
  port: number;
  database_path: string;
  database_name: string;
}

export interface ProcessesConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
}

export interface GeoConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
  use_ip_fallback: boolean;
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

export interface ScreenConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
  program: string;
  timestamp_format: string;
}

export interface MouseConfig {
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

export interface FrameMetadata {
  uuid: string;
  timestamp: Date;
  dpi: number;
  color_depth: number;
  contains_sensitive: boolean | null;
}

export interface ScreenFrame {
  uuid: string;
  timestamp: Date;
  image_path: string;
  resolution: any;
}

export interface CameraConfig {
  enabled: boolean;
  interval: number;
  output_dir: any;
  device: string;
  resolution: any;
  fps: number;
  timestamp_format: string;
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

