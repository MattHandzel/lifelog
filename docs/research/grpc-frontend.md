gRPC Frontend & Tauri Backend Integration

## 1. Build Script Adjustments (`build.rs`)

1. Enabled generation of Google "well-known" protobuf types with Serde derives:
   ```rust
   tonic_build::configure()
       .build_client(true)
       .build_server(false)
       .file_descriptor_set_path(out_dir.join("lifelog_descriptor.bin"))
       .compile_well_known_types(true) // <-- added
       .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
       .compile(
           &["../../proto/lifelog.proto", "../../proto/lifelog_types.proto"],
           &["../../proto/"],
       )?;
   ```
2. Called `compile_well_known_types(true)` so that `google.protobuf.Timestamp`, `StringValue`, etc., are re-generated with `#[derive(Serialize, Deserialize)]`, avoiding missing-Serde errors on `prost_types`.

## 2. Protobuf Module Includes in Tauri Backend (`src/main.rs`)

- Before including the `lifelog` module, we defined the `google::protobuf` module so that references in generated code resolve correctly:
  ```rust
  pub mod google {
      pub mod protobuf {
          tonic::include_proto!("google.protobuf");
      }
  }

  pub mod lifelog {
      tonic::include_proto!("lifelog");
  }
  ```

## 3. `GRPC_SERVER_ADDRESS` Constant

Keeps the server endpoint in one place:
```rust
const GRPC_SERVER_ADDRESS: &str = "http://127.0.0.1:7182";
```

## 4. Tauri Command: `get_component_config`

1. Connect to the gRPC server:
   ```rust
   println!("gRPC: get_component_config - connecting to {}", GRPC_SERVER_ADDRESS);
   let channel = Channel::from_static(GRPC_SERVER_ADDRESS)
       .connect().await?;
   println!("gRPC: get_component_config - connection established");
   ```
2. Perform the RPC:
   ```rust
   let response = client.get_config(tonic::Request::new(
       lifelog::GetSystemConfigRequest {}
   )).await?;
   println!("gRPC: get_component_config - RPC get_config succeeded: {:?}", response);
   ```
3. Extract the `collectors` JSON string from `SystemConfig` and parse it into the generated `CollectorConfig` type:
   ```rust
   let collectors_json = response.into_inner().config.unwrap().collectors;
   println!("gRPC: get_component_config - received collectors JSON: {}", collectors_json);
   let collector_config: lifelog::CollectorConfig =
       serde_json::from_str(&collectors_json)?;
   println!("gRPC: get_component_config - parsed CollectorConfig: {:?}", collector_config);
   ```
4. Extract only the requested sub-config (e.g. `screen`, `camera`, etc.):
   ```rust
   let component_value = match component_name.to_lowercase().as_str() {
       "screen" => serde_json::to_value(&collector_config.screen)?,
       // ... other branches ...
       _ => return Err(format!("Unknown component: {}", component_name)),
   };
   println!("gRPC: get_component_config - returning component '{}' value: {:?}", component_name, component_value);
   ```

## 5. Tauri Command: `set_component_config`

1. Connect and fetch existing `SystemConfig`:
   ```rust
   println!("gRPC: set_component_config - connecting to {}", GRPC_SERVER_ADDRESS);
   let channel = Channel::from_static(GRPC_SERVER_ADDRESS)
       .connect().await?;
   println!("gRPC: set_component_config - connection established");
   let get_response = client.get_config(tonic::Request::new(
       lifelog::GetSystemConfigRequest{}
   )).await?;
   println!("gRPC: set_component_config - RPC get_config succeeded: {:?}", get_response);
   ```
2. Parse the `collectors` JSON into `CollectorConfig`:
   ```rust
   let collectors_json = get_response.into_inner().config.unwrap().collectors;
   let mut collector_config: lifelog::CollectorConfig = serde_json::from_str(&collectors_json)?;
   println!("gRPC: set_component_config - parsed CollectorConfig: {:?}", collector_config);
   ```
3. Merge the new partial config value into the appropriate field:
   ```rust
   match component_name.to_lowercase().as_str() {
       "screen" => collector_config.screen = Some(serde_json::from_value(config_value)?),
       // ... other branches ...
   }
   println!("gRPC: set_component_config - sending updated CollectorConfig: {:?}", collector_config);
   ```
4. Send the `SetSystemConfigRequest` and log the success flag:
   ```rust
   let set_response = client.set_config(
       tonic::Request::new(lifelog::SetSystemConfigRequest { config: Some(collector_config.clone()) })
   ).await?;
   println!("gRPC: set_component_config - RPC set_config succeeded: {:?}", set_response);
   ```
