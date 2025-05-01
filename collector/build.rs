
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .compile(
            &["src/proto/controller.proto"],
            &["proto"],
        )?;
    Ok(())
}