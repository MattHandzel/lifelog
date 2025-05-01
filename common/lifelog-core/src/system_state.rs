use serde::{Deserialize, Serialize};
use DashMap::DashMap;

// TODO: We need to model other applications/api's state so they can be used by the server to make
// decisions
pub struct SystemState {
    pub timestamp: DateTime<Utc>,
    pub collector_states: DashMap<CollectorId, CollectorState>,
    pub interface_states: DashMap<InterfaceId, InterfaceState>,
    pub server_states: Vec<ServerId, ServerState>, // There is only 1 server in this model, but maybe we want
                                                   // to have more servers in the future
}
