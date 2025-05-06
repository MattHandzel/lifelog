use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("lifelog_descriptor.bin"))
        .compile_protos(
            &[
                "../../proto/lifelog.proto",
                "../../proto/lifelog_types.proto",
            ],
            &["../../proto"],
        )?;
    Ok(())
}
