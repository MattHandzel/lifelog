use chrono::Utc;
use config::ServerPolicyConfig;
use lifelog_types::{ServerActionType, SystemState};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ServerPolicy {
    pub config: ServerPolicyConfig,
}

pub type ServerAction = lifelog_core::ServerAction<
    lifelog_types::QueryRequest,
    lifelog_types::GetDataRequest,
    lifelog_types::Uuid,
>;

impl ServerPolicy {
    pub fn new(config: ServerPolicyConfig) -> Self {
        ServerPolicy { config }
    }

    pub fn get_action(&self, state: SystemState) -> ServerAction {
        let ss = match state.server_state.as_ref() {
            Some(s) => s,
            None => return ServerAction::Sleep(Duration::from_millis(100)),
        };

        let t_now = Utc::now();
        let t_last = ss
            .timestamp_of_last_sync
            .as_ref()
            .map(|t| {
                chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32).unwrap_or_default()
            })
            .unwrap_or(chrono::DateTime::<Utc>::MIN_UTC);

        let sync_interval = self.config.collector_sync_interval as f64;

        if (t_now - t_last).num_seconds() as f64 >= sync_interval
            && !ss
                .pending_actions
                .contains(&(ServerActionType::SyncData as i32))
        {
            ServerAction::SyncData("SELECT * FROM screen".to_string())
        } else if !ss
            .pending_actions
            .contains(&(ServerActionType::TransformData as i32))
        {
            ServerAction::TransformData(vec![])
        } else {
            ServerAction::Sleep(Duration::from_millis(100))
        }
    }
}
