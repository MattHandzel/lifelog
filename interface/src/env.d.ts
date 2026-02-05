/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_BASE_URL: string;
  readonly VITE_WEATHER_API_KEY: string;
  readonly VITE_LIFELOG_HOME_DIR: string;
  readonly VITE_DEVELOPMENT_MODE: string;
  readonly VITE_APP_TITLE: string;
  readonly VITE_LOG_LEVEL: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
} 