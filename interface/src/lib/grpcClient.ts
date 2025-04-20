import { load } from 'protobufjs';
import axios from 'axios';

// Proto types (these will be replaced by generated types in the future)
export interface TimeRangeRequest {
  startTime: string;
  endTime: string;
  limit?: number;
  offset?: number;
}

export interface SearchRequest {
  query: string;
  dataSources?: string[];
  timeRange?: TimeRangeRequest;
  useLlm?: boolean;
}

export interface SearchResult {
  type: string;
  timestamp: string;
  sourceId: string;
  metadata: Record<string, string>;
  data: { binaryData?: Uint8Array, textData?: string };
  relevanceScore: number;
}

export interface SearchResponse {
  results: SearchResult[];
  totalResults: number;
  searchId: string;
}

export interface LoggerStatusRequest {
  loggerNames?: string[];
}

export interface LoggerStatus {
  name: string;
  enabled: boolean;
  running: boolean;
  lastActive: string;
  dataPoints: number;
  error: string;
}

export interface LoggerStatusResponse {
  loggers: LoggerStatus[];
  systemStats: Record<string, string>;
}

export interface ToggleLoggerRequest {
  loggerName: string;
  enable: boolean;
}

export interface ToggleLoggerResponse {
  success: boolean;
  errorMessage: string;
  status?: LoggerStatus;
}

export interface SnapshotRequest {
  loggers?: string[];
  options?: Record<string, string>;
}

export interface SnapshotResponse {
  snapshotId: string;
  success: boolean;
  errorMessage: string;
  triggeredLoggers: string[];
}

// For streaming methods (these would ideally be real streams, but we're using REST for the interface)
export interface StreamingRequest {
  startTime: string;
  endTime: string;
  limit?: number;
  offset?: number;
}

export interface ScreenshotData {
  id: string;
  timestamp: string;
  imageData: string; // Base64 encoded
  mimeType: string;
  metadata: Record<string, string>;
}

export interface ProcessData {
  id: string;
  timestamp: string;
  processName: string;
  windowTitle: string;
  pid: number;
  cpuUsage: number;
  memoryUsage: number;
  isFocused: boolean;
}

/**
 * A client for the Lifelog gRPC service that uses the Fetch API and REST endpoints.
 * This is a temporary solution until we have a proper gRPC-web setup.
 * The requests still follow the gRPC service definitions but are sent as REST requests.
 */
export class LifelogClient {
  private baseUrl: string;

  constructor() {
    // Get the base URL from environment variables or fallback to localhost
    this.baseUrl = import.meta.env.VITE_GRPC_API_URL || 'http://localhost:50051';
  }

  /**
   * Search for entries in the lifelog
   */
  async search(request: SearchRequest): Promise<SearchResponse> {
    try {
      const response = await axios.post(`${this.baseUrl}/lifelog.LifelogService/Search`, request);
      return response.data as SearchResponse;
    } catch (error) {
      console.error('Error in search request:', error);
      throw error;
    }
  }

  /**
   * Get screenshots within a time range
   */
  async getScreenshots(request: TimeRangeRequest): Promise<ScreenshotData[]> {
    try {
      const response = await axios.post(`${this.baseUrl}/lifelog.LifelogService/GetScreenshots`, request);
      return response.data as ScreenshotData[];
    } catch (error) {
      console.error('Error in getScreenshots request:', error);
      throw error;
    }
  }

  /**
   * Get process data within a time range
   */
  async getProcesses(request: TimeRangeRequest): Promise<ProcessData[]> {
    try {
      const response = await axios.post(`${this.baseUrl}/lifelog.LifelogService/GetProcesses`, request);
      return response.data as ProcessData[];
    } catch (error) {
      console.error('Error in getProcesses request:', error);
      throw error;
    }
  }

  /**
   * Get status of all loggers
   */
  async getLoggerStatus(request: LoggerStatusRequest = {}): Promise<LoggerStatusResponse> {
    try {
      const response = await axios.post(`${this.baseUrl}/lifelog.LifelogService/GetLoggerStatus`, request);
      return response.data as LoggerStatusResponse;
    } catch (error) {
      console.error('Error in getLoggerStatus request:', error);
      throw error;
    }
  }

  /**
   * Toggle a logger on or off
   */
  async toggleLogger(request: ToggleLoggerRequest): Promise<ToggleLoggerResponse> {
    try {
      const response = await axios.post(`${this.baseUrl}/lifelog.LifelogService/ToggleLogger`, request);
      return response.data as ToggleLoggerResponse;
    } catch (error) {
      console.error('Error in toggleLogger request:', error);
      throw error;
    }
  }

  /**
   * Take a snapshot with specified loggers
   */
  async takeSnapshot(request: SnapshotRequest = {}): Promise<SnapshotResponse> {
    try {
      const response = await axios.post(`${this.baseUrl}/lifelog.LifelogService/TakeSnapshot`, request);
      return response.data as SnapshotResponse;
    } catch (error) {
      console.error('Error in takeSnapshot request:', error);
      throw error;
    }
  }
}

// Create and export a singleton instance
export const lifelogClient = new LifelogClient(); 