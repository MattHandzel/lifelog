# Lessons / Gotchas

- The workspace includes a Tauri UI crate (`interface/src-tauri`) that pulls native GUI deps; it should not be built by default for backend/collector CI.
- Proto compilation (`common/lifelog-proto`) requires a working `protoc` and valid `.proto` files; a vendored `protoc` fallback makes `cargo check/test` more robust.
- On Linux outside `nix develop`, `cargo check` can fail due to missing native libraries (e.g. ALSA via `cpal`). For now, treat `nix develop` as the reference environment.

