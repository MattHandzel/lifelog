use lifelog_core::*;

use lifelog_macros::lifelog_type;
use lifelog_proto;
use serde::{Deserialize, Serialize};

#[lifelog_type(Data)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserFrame {
    pub url: String,
    pub title: String,
    pub visit_count: u32,
}