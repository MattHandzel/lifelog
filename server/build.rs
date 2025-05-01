use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("lifelog_descriptor.bin"))
        .compile_protos(&["../proto/lifelog.proto"], &["../proto"])?;
        .compile_protos(&["../proto/lifelog_types.proto"], &["../proto"])?;
    tonic_build::compile_protos("../proto/lifelog.proto")?;
    tonic_build::compile_protos("../proto/lifelog_types.proto")?;
    Ok(())
}
