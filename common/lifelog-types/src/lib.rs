use dashmap::DashMap;
use lifelog_core::*;
use lifelog_macros::lifelog_type;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Unit {
    GB,
    Count,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageType {
    Percentage(f32),
    RealValue(u64, Unit),
}

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
    pub cpu_usage: UsageType,
    pub memory_usage: UsageType,
    pub threads: UsageType,
}

impl Default for ServerState {
    fn default() -> Self {
        ServerState {
            name: "LifelogServer".to_string(),
            timestamp: Utc::now(),
            cpu_usage: UsageType::Percentage(0.),
            memory_usage: UsageType::Percentage(0.),
            threads: UsageType::RealValue(1, Unit::Count),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisteredCollector {
    id: CollectorId,
    address: String,
}

#[derive(Clone, Debug)]
pub struct RegisteredInterface {
    id: InterfaceId,
    address: String,
}

pub type CollectorId = String;
pub type InterfaceId = String;
pub type ServerId = String;

// TODO: We need to model other applications/api's state so they can be used by the server to make
// decisions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemState {
    pub timestamp: DateTime<Utc>,
    pub collector_states: DashMap<CollectorId, CollectorState>,
    pub interface_states: DashMap<InterfaceId, InterfaceState>,
    pub server_state: ServerState, // There is only 1 server in this model, but maybe we want
                                   // to have more servers in the future
}

impl Default for SystemState {
    fn default() -> Self {
        SystemState {
            timestamp: Utc::now(),
            collector_states: DashMap::new(),
            interface_states: DashMap::new(),
            server_state: ServerState::default(),
        }
    }
}
