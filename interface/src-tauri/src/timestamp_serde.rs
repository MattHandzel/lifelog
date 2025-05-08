// A simple module to help serialize/deserialize prost's Timestamp type
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use prost_types::Timestamp;

pub fn serialize<S>(timestamp: &Option<Timestamp>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match timestamp {
        Some(ts) => {
            let ts_str = format!("{}.{:09}", ts.seconds, ts.nanos);
            serializer.serialize_str(&ts_str)
        }
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Timestamp>, D::Error>
where
    D: Deserializer<'de>,
{
    let ts_str = Option::<String>::deserialize(deserializer)?;
    
    match ts_str {
        Some(s) => {
            let parts: Vec<&str> = s.split('.').collect();
            if parts.len() != 2 {
                return Err(serde::de::Error::custom("Invalid timestamp format"));
            }
            
            let seconds = parts[0].parse::<i64>().map_err(serde::de::Error::custom)?;
            let nanos = parts[1].parse::<i32>().map_err(serde::de::Error::custom)?;
            
            Ok(Some(Timestamp { seconds, nanos }))
        }
        None => Ok(None),
    }
} 