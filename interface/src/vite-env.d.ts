/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_URL: string;
  // more env variables...
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

// Add declaration for process.env for compatibility
declare namespace NodeJS {
  interface ProcessEnv {
    NODE_ENV: 'development' | 'production' | 'test';
    REACT_APP_API_URL?: string;
  }
}

declare var process: {
  env: {
    NODE_ENV: 'development' | 'production' | 'test';
    REACT_APP_API_URL?: string;
  }
};
