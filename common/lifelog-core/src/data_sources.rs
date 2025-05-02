use serde::{Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/data_modalities.rs"));

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct DataSource {
    name: String, // human-definable name
    location: String,
    modality: DataModality, // the type of data modality
}
