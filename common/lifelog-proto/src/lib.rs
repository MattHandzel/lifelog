use tonic;

pub mod _proto {
    tonic::include_proto!("lifelog");
}

pub use _proto::*;
