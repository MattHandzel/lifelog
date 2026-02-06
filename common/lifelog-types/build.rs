fn main() {
    // DataModality is now defined in lifelog-proto and re-exported.
    // This build script is no longer needed for enum generation.
    println!("cargo:rerun-if-changed=../../data-modalities");
}
