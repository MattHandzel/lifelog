
pub mod _proto {
    tonic::include_proto!("lifelog");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("lifelog_descriptor");
}

pub use _proto::*;
