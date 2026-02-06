use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    // Ensure this runs before the tauri_build::build()
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .build_client(true) // We need the client code
        .build_server(false) // We don't need server code in the interface
        .file_descriptor_set_path(out_dir.join("lifelog_descriptor.bin")) // Store the descriptor set
        .compile_well_known_types(true) // Generate code for well-known types so serde derive can be applied
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]") // Add serde derive attributes to generated message types
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
