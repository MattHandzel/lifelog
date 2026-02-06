use lifelog_core::*;

use lifelog_macros::lifelog_type;
use lifelog_proto;
use serde::{Deserialize, Serialize};

use lifelog_types::Modality;

#[lifelog_type(Data)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserFrame {
    pub uuid: ::lifelog_core::uuid::Uuid,
    pub timestamp: ::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc>,
    pub url: String,
    pub title: String,
    pub visit_count: u32,
}

impl Modality for BrowserFrame {
    fn into_payload(self) -> lifelog_proto::lifelog_data::Payload {
        lifelog_proto::lifelog_data::Payload::Browserframe(self.into()) // TODO: refactor code so this is
    }
    fn get_table_name() -> &'static str {
        "browser" // TODO: automatically generate this based on folder name
    }
    fn get_surrealdb_schema() -> &'static str {
        r#"
    DEFINE FIELD timestamp  ON `{table}` TYPE datetime;
    DEFINE FIELD url      ON `{table}` TYPE string;
    DEFINE FIELD title  ON `{table}` TYPE string;
    DEFINE FIELD visit_count ON `{table}` TYPE int;
"#
    }
}
