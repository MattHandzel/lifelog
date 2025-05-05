use lifelog_types::{Unit, UsageType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPolicyConfig {
    max_cpu_usage: UsageType,
    max_memory_usage: UsageType,
    max_threads: UsageType,
}

impl Default for ServerPolicyConfig {
    fn default() -> Self {
        Self {
            max_cpu_usage: UsageType::Percentage(20.0),
            max_memory_usage: UsageType::RealValue(8, Unit::GB),
            max_threads: UsageType::RealValue(10, Unit::Count),
        }
    }
}
