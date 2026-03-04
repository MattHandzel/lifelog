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
        .type_attribute(
            "lifelog.Query",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.LifelogDataKey",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.ModalityDescriptor",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.FieldDescriptor",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.SystemConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.CollectorConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.ServerConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.BrowserHistoryConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.ScreenConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.CameraConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.MicrophoneConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.ProcessesConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.HyprlandConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.WeatherConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.WifiConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.ClipboardConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.ShellHistoryConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.MouseConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.WindowActivityConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "lifelog.KeyboardConfig",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
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
