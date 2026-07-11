# RepoAtlas Report

- Nodes: 220
- Edges: 566
- Journeys: 14
- Decisions: 139
- Drift findings: 6

## Journeys

### entry:cli:collector
- Entrypoint: `entry:cli:collector`
- Narrative: entry:cli:collector expands to 29 nodes within 4 hops
- Nodes in view: 29

### entry:cli:interface
- Entrypoint: `entry:cli:interface`
- Narrative: entry:cli:interface expands to 8 nodes within 4 hops
- Nodes in view: 8

### entry:cli:server
- Entrypoint: `entry:cli:server`
- Narrative: entry:cli:server expands to 30 nodes within 4 hops
- Nodes in view: 30

### entry:rpc:ControlStream
- Entrypoint: `entry:rpc:ControlStream`
- Narrative: entry:rpc:ControlStream expands to 39 nodes within 4 hops
- Nodes in view: 39

### entry:rpc:GetConfig
- Entrypoint: `entry:rpc:GetConfig`
- Narrative: entry:rpc:GetConfig expands to 39 nodes within 4 hops
- Nodes in view: 39

### entry:rpc:SetConfig
- Entrypoint: `entry:rpc:SetConfig`
- Narrative: entry:rpc:SetConfig expands to 39 nodes within 4 hops
- Nodes in view: 39

### entry:rpc:GetData
- Entrypoint: `entry:rpc:GetData`
- Narrative: entry:rpc:GetData expands to 39 nodes within 4 hops
- Nodes in view: 39

### entry:rpc:Query
- Entrypoint: `entry:rpc:Query`
- Narrative: entry:rpc:Query expands to 39 nodes within 4 hops
- Nodes in view: 39

### entry:rpc:Replay
- Entrypoint: `entry:rpc:Replay`
- Narrative: entry:rpc:Replay expands to 39 nodes within 4 hops
- Nodes in view: 39

### entry:rpc:GetState
- Entrypoint: `entry:rpc:GetState`
- Narrative: entry:rpc:GetState expands to 39 nodes within 4 hops
- Nodes in view: 39

### entry:rpc:UploadChunks
- Entrypoint: `entry:rpc:UploadChunks`
- Narrative: entry:rpc:UploadChunks expands to 39 nodes within 4 hops
- Nodes in view: 39

### entry:rpc:GetUploadOffset
- Entrypoint: `entry:rpc:GetUploadOffset`
- Narrative: entry:rpc:GetUploadOffset expands to 39 nodes within 4 hops
- Nodes in view: 39

### entry:rpc:ListModalities
- Entrypoint: `entry:rpc:ListModalities`
- Narrative: entry:rpc:ListModalities expands to 39 nodes within 4 hops
- Nodes in view: 39

### entry:rpc:PairCollector
- Entrypoint: `entry:rpc:PairCollector`
- Narrative: entry:rpc:PairCollector expands to 39 nodes within 4 hops
- Nodes in view: 39

## Drift

- [hard] Forbidden boundary edge interface -> collector (`module:interface::components::Login` -> `resource:table:environment`)
- [soft] Unexpected boundary edge common -> collector (`module:common::server_config` -> `resource:table:environment`)
- [soft] Unexpected boundary edge interface -> common (`module:interface::bin::lifelog-logger` -> `module:common::root`)
- [soft] Unexpected boundary edge interface -> common (`module:interface::bin::lifelog-server` -> `module:common::root`)
- [soft] Unexpected boundary edge interface -> common (`module:interface::bin::text-upload` -> `module:common::root`)
- [soft] Unexpected boundary edge interface -> common (`module:interface::root` -> `module:common::root`)

## Decisions

- [TODO] Cross-boundary dependency: entry:rpc:ControlStream -> module:server::grpc_service
- [TODO] Cross-boundary dependency: entry:rpc:GetConfig -> module:server::grpc_service
- [TODO] Cross-boundary dependency: entry:rpc:GetData -> module:server::grpc_service
- [TODO] Cross-boundary dependency: entry:rpc:GetState -> module:server::grpc_service
- [TODO] Cross-boundary dependency: entry:rpc:GetUploadOffset -> module:server::grpc_service
- [TODO] Cross-boundary dependency: entry:rpc:ListModalities -> module:server::grpc_service
- [TODO] Cross-boundary dependency: entry:rpc:PairCollector -> module:server::grpc_service
- [TODO] Cross-boundary dependency: entry:rpc:Query -> module:server::grpc_service
- [TODO] Cross-boundary dependency: entry:rpc:Replay -> module:server::grpc_service
- [TODO] Cross-boundary dependency: entry:rpc:SetConfig -> module:server::grpc_service
- [TODO] Cross-boundary dependency: entry:rpc:UploadChunks -> module:server::grpc_service
- [TODO] Cross-boundary dependency: module:collector::collector -> module:common::root
- [TODO] Cross-boundary dependency: module:collector::modules::audio -> module:common::root
- [TODO] Cross-boundary dependency: module:collector::modules::browser_history -> module:common::root
- [TODO] Cross-boundary dependency: module:collector::modules::camera -> module:common::root
- [TODO] Cross-boundary dependency: module:collector::modules::clipboard -> module:common::root
- [TODO] Cross-boundary dependency: module:collector::modules::hyprland -> module:common::root
- [TODO] Cross-boundary dependency: module:collector::modules::keystrokes -> module:common::root
- [TODO] Cross-boundary dependency: module:collector::modules::microphone -> module:common::root
- [TODO] Cross-boundary dependency: module:collector::modules::mouse -> module:common::root
- [TODO] Cross-boundary dependency: module:collector::modules::processes -> module:common::root
- [TODO] Cross-boundary dependency: module:collector::modules::screen -> module:common::root
- [TODO] Cross-boundary dependency: module:collector::modules::shell_history -> module:common::root
- [TODO] Cross-boundary dependency: module:collector::modules::weather -> module:common::root
- [TODO] Cross-boundary dependency: module:collector::modules::window_activity -> module:common::root
- [TODO] Cross-boundary dependency: module:collector::root -> module:common::root
- [TODO] Cross-boundary dependency: module:common::server_config -> resource:table:environment
- [TODO] Cross-boundary dependency: module:interface::bin::lifelog-logger -> module:common::root
- [TODO] Cross-boundary dependency: module:interface::bin::lifelog-server -> module:common::root
- [TODO] Cross-boundary dependency: module:interface::bin::text-upload -> module:common::root
- [TODO] Cross-boundary dependency: module:interface::components::Login -> resource:table:environment
- [TODO] Cross-boundary dependency: module:interface::generated::lifelog -> module:server::server
- [TODO] Cross-boundary dependency: module:interface::root -> module:common::root
- [TODO] Cross-boundary dependency: module:server::replay -> module:common::replay
- [TODO] entry:cli:collector touches 8 resources
- [TODO] entry:cli:interface touches 4 resources
- [TODO] entry:cli:server touches 11 resources
- [TODO] entry:rpc:ControlStream touches 20 resources
- [TODO] entry:rpc:GetConfig touches 20 resources
- [TODO] entry:rpc:GetData touches 20 resources
- [TODO] entry:rpc:GetState touches 20 resources
- [TODO] entry:rpc:GetUploadOffset touches 20 resources
- [TODO] entry:rpc:ListModalities touches 20 resources
- [TODO] entry:rpc:PairCollector touches 20 resources
- [TODO] entry:rpc:Query touches 20 resources
- [TODO] entry:rpc:Replay touches 20 resources
- [TODO] entry:rpc:SetConfig touches 20 resources
- [TODO] entry:rpc:UploadChunks touches 20 resources
- [TODO] module:collector::modules::browser_history accesses resource:table:urls
- [TODO] module:collector::modules::browser_history accesses resource:table:visits
- [TODO] module:collector::modules::evdev_input_logger accesses resource:table:input
- [TODO] module:collector::modules::input_logger accesses resource:table:key_events
- [TODO] module:collector::modules::input_logger accesses resource:table:mouse_buttons
- [TODO] module:collector::modules::input_logger accesses resource:table:mouse_movements
- [TODO] module:collector::modules::input_logger accesses resource:table:mouse_wheel
- [TODO] module:collector::modules::text_upload accesses resource:table:text_uploads
- [TODO] module:collector::modules::wayland_input_logger accesses resource:table:key_events
- [TODO] module:collector::modules::wayland_input_logger accesses resource:table:last
- [TODO] module:collector::modules::wayland_input_logger accesses resource:table:mouse_buttons
- [TODO] module:collector::modules::wayland_input_logger accesses resource:table:mouse_movements
- [TODO] module:collector::modules::wayland_input_logger accesses resource:table:mouse_wheel
- [TODO] module:collector::modules::weather accesses resource:table:environment
- [TODO] module:collector::setup accesses resource:table:sqlite_master
- [TODO] module:collector::setup accesses resource:table:test
- [TODO] module:common::ocr accesses resource:table:dynamic_image
- [TODO] module:common::root accesses resource:table:implementations
- [TODO] module:common::root accesses resource:table:lifelog_types
- [TODO] module:common::root accesses resource:table:unified
- [TODO] module:common::server_config accesses resource:table:environment
- [TODO] module:common::time_skew accesses resource:table:median
- [TODO] module:common::time_skew accesses resource:table:samples
- [TODO] module:interface::App accesses resource:table:some
- [TODO] module:interface::components::DevicesDashboard accesses resource:table:local
- [TODO] module:interface::components::Login accesses resource:table:environment
- [TODO] module:interface::components::MicrophoneDashboard accesses resource:table:your
- [TODO] module:interface::components::NetworkTopologyDashboard accesses resource:table:backend
- [TODO] module:interface::components::NetworkTopologyDashboard accesses resource:table:modality
- [TODO] module:interface::components::ScreenDashboard accesses resource:table:backend
- [TODO] module:interface::components::ScreenDashboard accesses resource:table:tauri
- [TODO] module:interface::components::SettingsDashboard accesses resource:table:local
- [TODO] module:interface::generated::google::protobuf::timestamp accesses resource:table:current
- [TODO] module:interface::generated::google::protobuf::timestamp accesses resource:table:java
- [TODO] module:interface::generated::google::protobuf::timestamp accesses resource:table:posix
- [TODO] module:interface::generated::google::protobuf::timestamp accesses resource:table:win32
- [TODO] module:interface::generated::lifelog accesses resource:table:this
- [TODO] module:interface::interface::vite.config accesses resource:table:obscuring
- [TODO] module:interface::storage accesses resource:table:path
- [TODO] module:interface::storage accesses resource:table:wav
- [TODO] module:server::data_retrieval accesses resource:table:screen_records
- [TODO] module:server::db accesses resource:table:catalog
- [TODO] module:server::grpc_service accesses resource:table:controlstream
- [TODO] module:server::grpc_service accesses resource:table:peer
- [TODO] module:server::grpc_service accesses resource:table:upload_chunks
- [TODO] module:server::ingest accesses resource:table:audio_records
- [TODO] module:server::ingest accesses resource:table:browser_records
- [TODO] module:server::ingest accesses resource:table:clipboard_records
- [TODO] module:server::ingest accesses resource:table:duration_secs
- [TODO] module:server::ingest accesses resource:table:keystroke_records
- [TODO] module:server::ingest accesses resource:table:ocr_records
- [TODO] module:server::ingest accesses resource:table:payload
- [TODO] module:server::ingest accesses resource:table:screen_records
- [TODO] module:server::ingest accesses resource:table:shell_history_records
- [TODO] module:server::ingest accesses resource:table:upload_chunks
- [TODO] module:server::policy accesses resource:table:screen
- [TODO] module:server::postgres accesses resource:table:schema_migrations
- [TODO] module:server::query::planner accesses resource:table:for
- [TODO] module:server::query::planner accesses resource:table:operators
- [TODO] module:server::query::planner accesses resource:table:terms
- [TODO] module:server::schema accesses resource:table:catalog
- [TODO] module:server::schema accesses resource:table:upload_chunks
- [TODO] module:server::schema accesses resource:table:watermarks
- [TODO] module:server::server accesses resource:table:command
- [TODO] module:server::server accesses resource:table:payload
- [TODO] module:server::server accesses resource:table:screen_records
- [TODO] module:server::server accesses resource:table:upload_chunks
- [TODO] module:server::transform accesses resource:table:upload_chunks
- [TODO] module:tools::tools::repoatlas::repoatlas accesses resource:table:__future__
- [TODO] module:tools::tools::repoatlas::repoatlas accesses resource:table:collections
- [TODO] module:tools::tools::repoatlas::repoatlas accesses resource:table:pathlib
- [TODO] module:tools::tools::repoatlas::repoatlas accesses resource:table:typing
- [TODO] module:tools::tools::repoatlas::repoatlas_agents accesses resource:table:__future__
- [TODO] module:tools::tools::repoatlas::repoatlas_agents accesses resource:table:collections
- [TODO] module:tools::tools::repoatlas::repoatlas_agents accesses resource:table:concurrent
- [TODO] module:tools::tools::repoatlas::repoatlas_agents accesses resource:table:pathlib
- [TODO] module:tools::tools::repoatlas::repoatlas_agents accesses resource:table:typing
- [TODO] module:tools::tools::repoatlas::text_view_audit accesses resource:table:__future__
- [TODO] module:tools::tools::repoatlas::text_view_audit accesses resource:table:entry
- [TODO] module:tools::tools::repoatlas::text_view_audit accesses resource:table:pathlib
- [TODO] module:tools::tools::repoatlas::text_view_audit accesses resource:table:typing
- [TODO] module:tools::tools::repoatlas::validate_artifacts accesses resource:table:__future__
- [TODO] module:tools::tools::repoatlas::validate_artifacts accesses resource:table:pathlib
- [TODO] module:tools::tools::repoatlas::validate_artifacts accesses resource:table:typing
- [TODO] module:tools::tools::repoatlas::viewer::serve_viewer accesses resource:table:__future__
- [TODO] module:tools::tools::repoatlas::viewer::serve_viewer accesses resource:table:pathlib
- [TODO] module:tools::tools::repoatlas::viz_compose accesses resource:table:__future__
- [TODO] module:tools::tools::repoatlas::viz_compose accesses resource:table:pathlib
- [TODO] module:tools::tools::repoatlas::viz_compose accesses resource:table:provided
- [TODO] module:tools::tools::repoatlas::viz_compose accesses resource:table:static
- [TODO] module:tools::tools::repoatlas::viz_compose accesses resource:table:typing
