use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .build_client(false)
        .build_server(false)
        .file_descriptor_set_path(out_dir.join("lifelog_descriptor.bin")) 
        .compile_well_known_types(true) 
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .client_attribute("lifelog.LifelogServerService", "#[derive(Debug)]")
        .client_attribute("lifelog.CollectorService", "#[derive(Debug)]")
        .client_attribute("lifelog.LifelogServerServiceClient", "#[allow(dead_code)]")
        .client_attribute("lifelog.CollectorServiceClient", "#[allow(dead_code)]")
        .client_attribute(".", r#"#[allow(unused_qualifications)]"#)
        .extern_path(".google.protobuf", "::prost_types")
        .field_attribute("timestamp", "#[serde(with = \"crate::timestamp_serde\")]")
        .compile(
            &[
                "../../proto/lifelog.proto", 
                "../../proto/lifelog_types.proto",
            ],
            &["../../proto/"],
        )?;

    tauri_build::build();

    Ok(())
}
