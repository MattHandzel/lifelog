# Proto-First Type System

This project adopts a **Protocol Buffers First** approach to type definitions. 

## Core Principles

1.  **Single Source of Truth**: All shared data structures (Configs, Events, Frames, States) are defined in `.proto` files located in the `/proto` directory.
2.  **Generated Code as Main Types**: The application code uses the generated Rust structs directly. We avoid "Domain Structs" that mirror Proto messages to eliminate manual type casting and conversion boilerplate.
3.  **Automatic Serde Support**: Generated structs are automatically decorated with `#[derive(serde::Serialize, serde::Deserialize)]` via `pbjson`. This allows using Proto types for TOML/JSON configuration files and database records.

## Workflow: Adding a New Type

If you need to add a new data modality (e.g., `HeartRateFrame`):

1.  **Modify Proto**: Add the message definition to `proto/lifelog_types.proto`.
    ```protobuf
    message HeartRateFrame {
      string uuid = 1;
      google.protobuf.Timestamp timestamp = 2;
      uint32 bpm = 3;
    }
    ```
2.  **Update Container**: Add the new type to the `oneof payload` in the `LifelogData` message.
3.  **Build**: Run `cargo build`. The `lifelog-proto` crate will regenerate the Rust structs.
4.  **Use**: Import the type from `lifelog_proto` and use it.
    ```rust
    use lifelog_proto::HeartRateFrame;
    ```

## Timestamps

We use `google.protobuf.Timestamp` in `.proto` files. In Rust, these are mapped to `pbjson_types::Timestamp` (via `extern_path` in `build.rs`). 
- **Serialization**: Serializes to ISO 8601 string in JSON (compatible with standard web APIs).
- **Conversion**: To use with `chrono`, use:
  ```rust
  let dt = chrono::DateTime::<Utc>::from_timestamp(proto_ts.seconds, proto_ts.nanos as u32);
  ```

## Configuration Defaults

Since generated Proto structs use `Default::default()` (which is all zeros), custom defaults for configuration must be handled at the loading layer (e.g., in `common/config`).
Typically, we provide a `default_struct_name()` constructor that returns a pre-populated Proto struct.

## Why not use the `lifelog_type` macro?

Previously, a custom Rust macro generated `.proto` files from Rust code. This was disabled to:
- Make the schema explicit and language-agnostic.
- Speed up compilation by removing complex proc-macro expansion.
- Enable easier schema evolution and validation using standard Protobuf tools.
