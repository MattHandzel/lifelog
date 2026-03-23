use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    // Ensure this runs before the tauri_build::build()
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .file_descriptor_set_path(out_dir.join("lifelog_descriptor.bin"))
        // Add serde derives to ALL lifelog types — needed for Tauri invoke serialization
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .extern_path(".google.protobuf.Timestamp", "::pbjson_types::Timestamp")
        .compile_protos(
            &[
                "../../proto/lifelog.proto",       // Path relative to build.rs
                "../../proto/lifelog_types.proto", // Path relative to build.rs
            ],
            &["../../proto/"], // Include path for imports within protos
        )?;

    // Add rerun-if-changed directives
    println!("cargo:rerun-if-changed=../../proto/lifelog.proto");
    println!("cargo:rerun-if-changed=../../proto/lifelog_types.proto");

    // Default Tauri build steps
    tauri_build::build();

    Ok(())
}
