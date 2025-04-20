declare module '@improbable-eng/grpc-web' {
  export namespace grpc {
    export enum Code {
      OK = 0,
      Canceled = 1,
      Unknown = 2,
      InvalidArgument = 3,
      DeadlineExceeded = 4,
      NotFound = 5,
      AlreadyExists = 6,
      PermissionDenied = 7,
      ResourceExhausted = 8,
      FailedPrecondition = 9,
      Aborted = 10,
      OutOfRange = 11,
      Unimplemented = 12,
      Internal = 13,
      Unavailable = 14,
      DataLoss = 15,
      Unauthenticated = 16,
    }

    export interface UnaryOutput {
      status: Code;
      statusMessage: string;
      headers: Metadata;
      message: Uint8Array | null;
      trailers: Metadata;
    }

    export class Metadata {
      constructor();
      set(key: string, value: string | Uint8Array): void;
      get(key: string): string[] | Uint8Array[] | null;
      has(key: string): boolean;
      forEach(callback: (key: string, values: (string | Uint8Array)[]) => void): void;
    }

    export interface Request {
      serializeBinary(): Uint8Array;
    }

    export interface UnaryMethodDefinition<TRequest, TResponse> {
      methodName: string;
      service: {
        serviceName: string;
      };
      requestStream: boolean;
      responseStream: boolean;
      requestType: {
        new(): TRequest;
        serializeBinary(request: TRequest): Uint8Array;
      };
      responseType: {
        new(): TResponse;
        deserializeBinary(bytes: Uint8Array): TResponse;
      };
    }

    export interface UnaryMethodImpl<TRequest extends Request, TResponse> {
      (request: TRequest): Promise<TResponse>;
    }

    export interface UnaryTransportOptions {
      request: Request;
      host: string;
      metadata?: Metadata;
      onEnd: (output: UnaryOutput) => void;
      methodDescriptor: any;
      debug?: boolean;
      format?: string;
    }

    export function unary(options: UnaryTransportOptions): void;
  }
} 