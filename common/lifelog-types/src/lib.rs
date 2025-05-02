use dashmap::DashMap;
use lifelog_core::*;
use lifelog_macros::lifelog_type;
use serde::{Deserialize, Serialize};

#[lifelog_type(None)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorState {
    pub name: String,
    pub timestamp: DateTime<Utc>,
}

#[lifelog_type(None)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceState {}

#[lifelog_type(None)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerState {
    pub name: String,
    pub timestamp: DateTime<Utc>,
}

type CollectorId = String;
type InterfaceId = String;
type ServerId = String;

// TODO: We need to model other applications/api's state so they can be used by the server to make
// decisions
pub struct SystemState {
    pub timestamp: DateTime<Utc>,
    pub collector_states: DashMap<CollectorId, CollectorState>,
    pub interface_states: DashMap<InterfaceId, InterfaceState>,
    pub server_states: DashMap<ServerId, ServerState>, // There is only 1 server in this model, but maybe we want
                                                       // to have more servers in the future
}
