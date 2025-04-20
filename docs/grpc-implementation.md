# gRPC Implementation in Lifelog

This document describes the implementation of gRPC communication between the Lifelog interface and server components.

## Overview

We've implemented gRPC for communication between the UI (Tauri/React) and the backend server, replacing or augmenting the existing REST API. gRPC offers several benefits for our use case:

- **Performance**: Binary protocol is more efficient than JSON for large datasets
- **Strongly Typed**: Protocol Buffers provide type safety and code generation
- **Bidirectional Streaming**: Perfect for real-time data like camera/screen feeds
- **API Documentation**: Proto files act as self-documenting API contracts

## Architecture

```
┌────────────────┐          ┌─────────────────┐
│                │  gRPC    │                 │
│  Tauri/React   ├─────────►│  Rust Server    │
│   Interface    │◄─────────┤                 │
│                │          │                 │
└────────────────┘          └─────────────────┘
```

### Components

1. **Protocol Definitions**: `/proto/lifelog.proto` - Contains message and service definitions
2. **Server Implementation**: `server/src/grpc.rs` - Implements the gRPC service using Tonic
3. **Client Wrapper**: `interface/src/lib/grpcClient.ts` - TypeScript client for the UI

## Protocol Buffer Definitions

The main interface between client and server is defined in `proto/lifelog.proto`. It includes:

- Search functionality
- Data retrieval (screenshots, processes, camera frames)
- Analytics and aggregation
- Authentication
- Logger management

## Server Implementation

The server uses [Tonic](https://github.com/hyperium/tonic), a Rust implementation of gRPC. Key components:

- `server/src/grpc.rs` - Service implementation
- `server/build.rs` - Builds Protocol Buffers during compilation
- Environment variables for configuration

## Client Implementation

The client is implemented in TypeScript and uses:

- Initial implementation: REST wrapper that follows gRPC service pattern
- Future implementation: Full gRPC-Web integration

## Endpoints

| Service Method        | Description                                    | Streaming |
|-----------------------|------------------------------------------------|-----------|
| `Search`              | Search across all data modalities              | No        |
| `GetScreenshots`      | Retrieve screenshots within a time range       | Server    |
| `GetProcesses`        | Retrieve process data within a time range      | Server    |
| `GetCameraFrames`     | Retrieve camera frames within a time range     | Server    |
| `GetActivitySummary`  | Get summary of activities within a time range  | No        |
| `GetProcessStats`     | Get statistics about process usage             | No        |
| `GetLoggerStatus`     | Check status of all loggers                    | No        |
| `ToggleLogger`        | Enable/disable a specific logger               | No        |
| `TakeSnapshot`        | Trigger a snapshot from specified loggers      | No        |

## Authentication

gRPC authentication is handled via:

- JWT tokens passed in metadata
- Authentication interceptors on the server

## Deployment Considerations

When deploying:

1. Configure firewalls to allow gRPC traffic (default port: 50051)
2. For browser clients, use an HTTP/1.1 -> HTTP/2 proxy (e.g., Envoy)
3. Set appropriate connection limits and timeouts

## Future Work

- Implement true bidirectional streaming for real-time data
- Add full gRPC-Web support with proper streaming
- Optimize for large binary transfers (e.g., video streams)
- Add compression for network efficiency 