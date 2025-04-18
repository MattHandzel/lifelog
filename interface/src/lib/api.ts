import axios, { AxiosError, AxiosRequestConfig, AxiosResponse } from 'axios';

// API base URL from environment variables
// Use a fallback if the environment variable is not available
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080';
console.log('Using API base URL:', API_BASE_URL);

// Types for authentication
export interface User {
  id: string;
  username: string;
  role: string;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  token: string;
  user: User;
}

// Create axios instance with base configuration
const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Authentication helper
const AUTH_TOKEN_KEY = 'lifelog_auth_token';
const AUTH_USER_KEY = 'lifelog_auth_user';

// Add authentication token to outgoing requests
apiClient.interceptors.request.use((config) => {
  const token = localStorage.getItem(AUTH_TOKEN_KEY);
  if (token) {
    config.headers = config.headers || {};
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Handle authentication errors
apiClient.interceptors.response.use(
  (response) => response,
  (error: AxiosError) => {
    // If 401 Unauthorized, clear auth state and redirect to login
    if (error.response?.status === 401) {
      localStorage.removeItem(AUTH_TOKEN_KEY);
      localStorage.removeItem(AUTH_USER_KEY);
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

// Authentication functions
export const auth = {
  login: async (credentials: LoginRequest): Promise<LoginResponse> => {
    const response = await apiClient.post<LoginResponse>('/api/auth/login', credentials);
    localStorage.setItem(AUTH_TOKEN_KEY, response.data.token);
    localStorage.setItem(AUTH_USER_KEY, JSON.stringify(response.data.user));
    return response.data;
  },
  
  logout: () => {
    localStorage.removeItem(AUTH_TOKEN_KEY);
    localStorage.removeItem(AUTH_USER_KEY);
    window.location.href = '/login';
  },
  
  getUser: (): User | null => {
    const userJson = localStorage.getItem(AUTH_USER_KEY);
    if (!userJson) return null;
    try {
      return JSON.parse(userJson) as User;
    } catch (e) {
      localStorage.removeItem(AUTH_USER_KEY);
      return null;
    }
  },
  
  isAuthenticated: (): boolean => {
    return !!localStorage.getItem(AUTH_TOKEN_KEY);
  },
  
  getProfile: async (): Promise<User> => {
    const response = await apiClient.get<{ user: User }>('/api/auth/profile');
    localStorage.setItem(AUTH_USER_KEY, JSON.stringify(response.data.user));
    return response.data.user;
  },
};

// Logger API functions
export const loggerApi = {
  // Get all available loggers
  getLoggers: async (): Promise<string[]> => {
    const response = await apiClient.get<string[]>('/api/loggers');
    return response.data;
  },
  
  // Start a logger
  startLogger: async (name: string): Promise<void> => {
    await apiClient.post(`/api/logger/${name}/start`);
  },
  
  // Stop a logger
  stopLogger: async (name: string): Promise<void> => {
    await apiClient.post(`/api/logger/${name}/stop`);
  },
  
  // Get logger status
  getLoggerStatus: async (name: string): Promise<{ enabled: boolean }> => {
    const response = await apiClient.get<{ enabled: boolean }>(`/api/logger/${name}/status`);
    return response.data;
  },
  
  // Get logger config
  getLoggerConfig: async (name: string): Promise<any> => {
    const response = await apiClient.get(`/api/logger/${name}/config`);
    return response.data;
  },
  
  // Update logger config
  updateLoggerConfig: async (name: string, config: any): Promise<any> => {
    const response = await apiClient.put(`/api/logger/${name}/config`, config);
    return response.data;
  },
  
  // Get logger data
  getLoggerData: async (name: string, params?: any): Promise<any[]> => {
    const response = await apiClient.get(`/api/logger/${name}/data`, { params });
    return response.data;
  },
  
  // Get file URL
  getFileUrl: (name: string, path: string): string => {
    return `${API_BASE_URL}/api/logger/${name}/files/${path}`;
  },
  
  // Camera specific functions
  captureCamera: async (): Promise<void> => {
    await apiClient.post('/api/logger/camera/capture');
  },
  
  // Screen specific functions
  captureScreen: async (): Promise<void> => {
    await apiClient.post('/api/logger/screen/capture');
  },
  
  // Microphone specific functions
  startRecording: async (): Promise<void> => {
    await apiClient.post('/api/logger/microphone/record/start');
  },
  
  stopRecording: async (): Promise<void> => {
    await apiClient.post('/api/logger/microphone/record/stop');
  },
  
  pauseRecording: async (): Promise<void> => {
    await apiClient.post('/api/logger/microphone/record/pause');
  },
  
  resumeRecording: async (): Promise<void> => {
    await apiClient.post('/api/logger/microphone/record/resume');
  },
};

// Export the raw API client for direct use
export { apiClient }; 