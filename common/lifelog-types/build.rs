use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    // Prefer an explicitly-provided `PROTOC`, but fall back to a vendored binary so
    // `cargo check/test` work on systems without protobuf tooling preinstalled.
    if env::var_os("PROTOC").is_none() {
        let protoc = protoc_bin_vendored::protoc_bin_path()?;
        env::set_var("PROTOC", protoc);
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let descriptor_set = out_dir.join("lifelog_descriptor.bin");
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(&descriptor_set)
        .extern_path(".google.protobuf.Timestamp", "::pbjson_types::Timestamp")
        .compile_protos(
            &[
                "../../proto/lifelog.proto",
                "../../proto/lifelog_types.proto",
            ],
            &["../../proto"],
        )?;

    let descriptor_bytes = std::fs::read(&descriptor_set)?;
    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_bytes)?
        .build(&[".lifelog"])?;

    Ok(())
}
