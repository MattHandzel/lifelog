use lifelog_core::*;

use lifelog_macros::lifelog_type;
use lifelog_proto;
use serde::{Deserialize, Serialize};

use lifelog_types::Modality;
use rand::distr::{Alphanumeric, Distribution, StandardUniform};
use rand::{thread_rng, Rng};

#[lifelog_type(Data)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserFrame {
    pub url: String,
    pub title: String,
    pub visit_count: u32,
}

impl Modality for BrowserFrame {
    fn into_payload(self) -> lifelog_proto::lifelog_data::Payload {
        lifelog_proto::lifelog_data::Payload::Browserframe(self.into()) // TODO: refactor code so this is
    }
    fn get_table_name() -> &'static str {
        "screen" // TODO: automatically generate this based on folder name
    }
    fn get_surrealdb_schema() -> &'static str {
        r#"
    DEFINE FIELD timestamp  ON browser TYPE datetime;
    DEFINE FIELD url      ON browser TYPE string;
    DEFINE FIELD title  ON browser TYPE string;
    DEFINE FIELD visit_count ON browser TYPE int;
"#
    }
}

