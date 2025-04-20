import { grpc } from '@improbable-eng/grpc-web';
import { Message } from 'google-protobuf';
import * as $protobuf from 'protobufjs/minimal';

// Define type interfaces
interface UnaryMethodDefinition {
  methodName: string;
  service: {
    serviceName: string;
  };
  requestStream: boolean;
  responseStream: boolean;
  requestType: {
    serializeBinary: (data: any) => Uint8Array;
  };
  responseType: {
    deserializeBinary: (bytes: Uint8Array) => Uint8Array;
  };
}

/**
 * A client for the Lifelog gRPC service that uses gRPC-web.
 */
export class LifelogGrpcWebClient {
  private host: string;
  private lifelog: any;
  private protoLoaded: boolean = false;
  private loadingPromise: Promise<void> | null = null;

  constructor() {
    this.host = import.meta.env.VITE_GRPC_API_URL || 'http://localhost:50051';
    console.log(`Initializing gRPC-Web client with host: ${this.host}`);
    // Initialize protobuf immediately
    this.loadingPromise = this.initProtobuf();
  }
  
  private async initProtobuf() {
    try {
      console.log('Loading protobuf definitions...');
      const proto = await import('../generated/proto');
      this.lifelog = proto.lifelog;
      this.protoLoaded = true;
      console.log('Protobuf definitions loaded successfully:', this.lifelog ? 'yes' : 'no');
    } catch (error) {
      console.error('Failed to load protobuf definitions:', error);
      throw new Error(`Failed to load protobuf definitions: ${error}`);
    }
  }

  /**
   * Get status of all loggers
   */
  async getLoggerStatus(request: { loggerNames?: string[] } = {}): Promise<any> {
    await this.ensureProtoLoaded();
    console.log('Creating LoggerStatusRequest with:', request);
    
    const req = this.lifelog.LoggerStatusRequest.create(request);
    const reqBytes = this.lifelog.LoggerStatusRequest.encode(req).finish();
    
    console.log('Request bytes created, length:', reqBytes.length);
    console.log('Using host:', this.host);
    
    const methodDescriptor = {
      methodName: 'GetLoggerStatus',
      service: { serviceName: 'lifelog.LifelogService' },
      requestStream: false,
      responseStream: false,
      requestType: {
        serializeBinary: () => reqBytes,
      },
      responseType: {
        deserializeBinary: (bytes: Uint8Array) => bytes,
      }
    };
    
    console.log('Method descriptor created:', methodDescriptor);
    
    return new Promise((resolve, reject) => {
      try {
        grpc.unary({
          request: {
            serializeBinary: () => reqBytes,
          },
          host: this.host,
          onEnd: (response: grpc.UnaryOutput) => {
            const { status, statusMessage, message } = response;
            console.log('gRPC response received:', { status, statusMessage, hasMessage: !!message });
            
            if (status === grpc.Code.OK && message) {
              try {
                const loggerStatusResponse = this.lifelog.LoggerStatusResponse.decode(message);
                console.log('Decoded response:', loggerStatusResponse);
                resolve(loggerStatusResponse);
              } catch (error) {
                console.error('Failed to decode response:', error);
                reject(new Error(`Failed to decode response: ${error}`));
              }
            } else {
              console.error('gRPC error:', statusMessage || status);
              reject(new Error(`gRPC error: ${statusMessage || status}`));
            }
          },
          metadata: new grpc.Metadata(),
          methodDescriptor
        });
      } catch (error) {
        console.error('Error in grpc.unary call:', error);
        reject(error);
      }
    });
  }
  
  private createMethodDescriptor(methodName: string): UnaryMethodDefinition {
    console.log(`Creating method descriptor for: ${methodName}`);
    return {
      methodName: methodName,
      service: {
        serviceName: 'lifelog.LifelogService'
      },
      requestStream: false,
      responseStream: false,
      requestType: {
        serializeBinary: (req: any) => req,
      },
      responseType: {
        deserializeBinary: (bytes: Uint8Array) => bytes,
      }
    };
  }
  
  private async ensureProtoLoaded() {
    if (!this.protoLoaded) {
      console.log('Proto not yet loaded, waiting for initialization...');
      if (this.loadingPromise) {
        await this.loadingPromise;
      } else {
        console.log('No loading promise found, initializing now...');
        this.loadingPromise = this.initProtobuf();
        await this.loadingPromise;
      }
      
      if (!this.lifelog) {
        throw new Error('Failed to load protobuf definitions after waiting');
      }
      console.log('Proto loaded successfully after waiting');
    }
  }

  /**
   * Toggle a logger on or off
   */
  async toggleLogger(request: { loggerName: string, enable: boolean }): Promise<any> {
    await this.ensureProtoLoaded();
    
    const req = this.lifelog.ToggleLoggerRequest.create(request);
    const reqBytes = this.lifelog.ToggleLoggerRequest.encode(req).finish();
    
    return new Promise((resolve, reject) => {
      grpc.unary({
        request: {
          serializeBinary: () => reqBytes,
        },
        host: this.host,
        onEnd: (response: grpc.UnaryOutput) => {
          const { status, statusMessage, message } = response;
          
          if (status === grpc.Code.OK && message) {
            try {
              const toggleLoggerResponse = this.lifelog.ToggleLoggerResponse.decode(message);
              resolve(toggleLoggerResponse);
            } catch (error) {
              reject(new Error(`Failed to decode response: ${error}`));
            }
          } else {
            reject(new Error(`gRPC error: ${statusMessage || status}`));
          }
        },
        metadata: new grpc.Metadata(),
        methodDescriptor: this.createMethodDescriptor('ToggleLogger')
      });
    });
  }

  /**
   * Take a snapshot with specified loggers
   */
  async takeSnapshot(request: { loggers?: string[], options?: Record<string, string> } = {}): Promise<any> {
    await this.ensureProtoLoaded();
    
    const req = this.lifelog.SnapshotRequest.create(request);
    const reqBytes = this.lifelog.SnapshotRequest.encode(req).finish();
    
    return new Promise((resolve, reject) => {
      grpc.unary({
        request: {
          serializeBinary: () => reqBytes,
        },
        host: this.host,
        onEnd: (response: grpc.UnaryOutput) => {
          const { status, statusMessage, message } = response;
          
          if (status === grpc.Code.OK && message) {
            try {
              const snapshotResponse = this.lifelog.SnapshotResponse.decode(message);
              resolve(snapshotResponse);
            } catch (error) {
              reject(new Error(`Failed to decode response: ${error}`));
            }
          } else {
            reject(new Error(`gRPC error: ${statusMessage || status}`));
          }
        },
        metadata: new grpc.Metadata(),
        methodDescriptor: this.createMethodDescriptor('TakeSnapshot')
      });
    });
  }

  /**
   * Search for entries in the lifelog
   */
  async search(request: { 
    query: string, 
    dataSources?: string[], 
    timeRange?: { startTime: string, endTime: string },
    useLlm?: boolean 
  }): Promise<any> {
    await this.ensureProtoLoaded();
    
    const req = this.lifelog.SearchRequest.create(request);
    const reqBytes = this.lifelog.SearchRequest.encode(req).finish();
    
    return new Promise((resolve, reject) => {
      grpc.unary({
        request: {
          serializeBinary: () => reqBytes,
        },
        host: this.host,
        onEnd: (response: grpc.UnaryOutput) => {
          const { status, statusMessage, message } = response;
          
          if (status === grpc.Code.OK && message) {
            try {
              const searchResponse = this.lifelog.SearchResponse.decode(message);
              resolve(searchResponse);
            } catch (error) {
              reject(new Error(`Failed to decode response: ${error}`));
            }
          } else {
            reject(new Error(`gRPC error: ${statusMessage || status}`));
          }
        },
        metadata: new grpc.Metadata(),
        methodDescriptor: this.createMethodDescriptor('Search')
      });
    });
  }

  /**
   * Login with username and password
   */
  async login(request: { username: string, password: string }): Promise<any> {
    await this.ensureProtoLoaded();
    
    const req = this.lifelog.LoginRequest.create(request);
    const reqBytes = this.lifelog.LoginRequest.encode(req).finish();
    
    return new Promise((resolve, reject) => {
      grpc.unary({
        request: {
          serializeBinary: () => reqBytes,
        },
        host: this.host,
        onEnd: (response: grpc.UnaryOutput) => {
          const { status, statusMessage, message } = response;
          
          if (status === grpc.Code.OK && message) {
            try {
              const loginResponse = this.lifelog.LoginResponse.decode(message);
              resolve(loginResponse);
            } catch (error) {
              reject(new Error(`Failed to decode response: ${error}`));
            }
          } else {
            reject(new Error(`gRPC error: ${statusMessage || status}`));
          }
        },
        metadata: new grpc.Metadata(),
        methodDescriptor: this.createMethodDescriptor('Login')
      });
    });
  }

  /**
   * Register a new user
   */
  async register(request: { 
    username: string, 
    password: string, 
    email: string, 
    displayName: string 
  }): Promise<any> {
    await this.ensureProtoLoaded();
    
    const req = this.lifelog.RegisterRequest.create(request);
    const reqBytes = this.lifelog.RegisterRequest.encode(req).finish();
    
    return new Promise((resolve, reject) => {
      grpc.unary({
        request: {
          serializeBinary: () => reqBytes,
        },
        host: this.host,
        onEnd: (response: grpc.UnaryOutput) => {
          const { status, statusMessage, message } = response;
          
          if (status === grpc.Code.OK && message) {
            try {
              const registerResponse = this.lifelog.RegisterResponse.decode(message);
              resolve(registerResponse);
            } catch (error) {
              reject(new Error(`Failed to decode response: ${error}`));
            }
          } else {
            reject(new Error(`gRPC error: ${statusMessage || status}`));
          }
        },
        metadata: new grpc.Metadata(),
        methodDescriptor: this.createMethodDescriptor('Register')
      });
    });
  }

  /**
   * Get user profile
   */
  async getUserProfile(request: { userId?: string } = {}): Promise<any> {
    await this.ensureProtoLoaded();
    
    const req = this.lifelog.UserRequest.create(request);
    const reqBytes = this.lifelog.UserRequest.encode(req).finish();
    
    return new Promise((resolve, reject) => {
      grpc.unary({
        request: {
          serializeBinary: () => reqBytes,
        },
        host: this.host,
        onEnd: (response: grpc.UnaryOutput) => {
          const { status, statusMessage, message } = response;
          
          if (status === grpc.Code.OK && message) {
            try {
              const userProfile = this.lifelog.UserProfile.decode(message);
              resolve(userProfile);
            } catch (error) {
              reject(new Error(`Failed to decode response: ${error}`));
            }
          } else {
            reject(new Error(`gRPC error: ${statusMessage || status}`));
          }
        },
        metadata: new grpc.Metadata(),
        methodDescriptor: this.createMethodDescriptor('GetUserProfile')
      });
    });
  }
}

export const lifelogGrpcClient = new LifelogGrpcWebClient(); 