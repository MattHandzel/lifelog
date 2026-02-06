#![allow(clippy::needless_lifetimes)]

pub mod lifelog {
    include!(concat!(env!("OUT_DIR"), "/lifelog.rs"));
    include!(concat!(env!("OUT_DIR"), "/lifelog.serde.rs"));
}

pub use lifelog::*;

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("lifelog_descriptor");

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::print_stdout)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_timestamp() {
        let ts = Timerange::default();
        let json = serde_json::to_string(&ts).unwrap();
        println!("{}", json);
    }
}
