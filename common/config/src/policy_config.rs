use lifelog_types::{Unit, UsageType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPolicyConfig {
    pub max_cpu_usage: UsageType,
    pub max_memory_usage: UsageType,
    pub max_threads: UsageType,
    pub collector_sync_interval: f32, // TDOO: REFACTOR TO time duration
}

impl Default for ServerPolicyConfig {
    fn default() -> Self {
        Self {
            max_cpu_usage: UsageType::Percentage(20.0),
            max_memory_usage: UsageType::RealValue(8, Unit::GB),
            max_threads: UsageType::RealValue(10, Unit::Count),
            collector_sync_interval: 5.0,
        }
    }
}
