# Plan: Interface Security Implementation

## Objective
Update the Tauri backend to handle gRPC authentication and trust self-signed TLS certificates for the demo.

## Phase 1: Authentication
1. Implement a gRPC `Interceptor` in `interface/src-tauri/src/main.rs` that adds the `Authorization: Bearer <token>` header.
2. Update `InterfaceRuntimeConfig` to include an `auth_token` field.
3. Ensure the token can be loaded from `~/.config/lifelog/interface-config.toml` or the `LIFELOG_AUTH_TOKEN` environment variable.

## Phase 2: TLS & Connection
1. Update `create_grpc_channel` to support `https://` properly.
2. Add a mechanism to trust the demo's self-signed CA certificate (or use `with_native_roots` and provide instructions for the user to trust the cert at the OS level).
3. Update all tauri commands (`get_component_config`, `query_timeline`, etc.) to use the intercepted client.

## Phase 3: Validation
1. Verify that the interface can successfully call `GetState` on a hardened server.
