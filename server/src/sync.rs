use lifelog_core::*;
use lifelog_types::{CommandType, ServerCommand, SystemState};
use serde::Serialize;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::server::RegisteredCollector;

#[derive(Serialize)]
struct BeginUploadSessionPayload {
    query: String,
    requested_at: String,
}

fn dispatch_begin_upload_session(
    query: String,
    collectors: &mut [RegisteredCollector],
) -> Result<(), LifelogError> {
    let payload = serde_json::to_string(&BeginUploadSessionPayload {
        query,
        requested_at: chrono::Utc::now().to_rfc3339(),
    })?;

    let command = Ok(ServerCommand {
        r#type: CommandType::BeginUploadSession as i32,
        payload,
    });

    let total = collectors.len();
    let mut sent = 0usize;
    let mut closed_ids: Vec<String> = Vec::new();
    let mut full_ids: Vec<String> = Vec::new();

    for collector in collectors.iter_mut() {
        match collector.command_tx.try_send(command.clone()) {
            Ok(_) => {
                sent += 1;
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                closed_ids.push(collector.id.clone());
                tracing::warn!(
                    collector_id = %collector.id,
                    "sync dispatch skipped: control channel closed"
                );
            }
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                full_ids.push(collector.id.clone());
                tracing::warn!(
                    collector_id = %collector.id,
                    "sync dispatch deferred: control channel is full"
                );
            }
        }
    }

    tracing::info!(
        collectors_total = total,
        collectors_dispatched = sent,
        collectors_closed = closed_ids.len(),
        collectors_backpressured = full_ids.len(),
        "sync_data_with_collectors dispatch summary"
    );

    if !closed_ids.is_empty() {
        tracing::debug!(collector_ids = ?closed_ids, "collectors with closed control channels");
    }
    if !full_ids.is_empty() {
        tracing::debug!(collector_ids = ?full_ids, "collectors with full control channels");
    }

    Ok(())
}

pub(crate) async fn sync_data_with_collectors(
    _state: SystemState,
    _db: &Surreal<Client>,
    query: String,
    collectors: &mut [RegisteredCollector],
) -> Result<(), LifelogError> {
    dispatch_begin_upload_session(query, collectors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_collector(
        id: &str,
        tx: tokio::sync::mpsc::Sender<Result<ServerCommand, tonic::Status>>,
    ) -> RegisteredCollector {
        RegisteredCollector {
            id: id.to_string(),
            address: "127.0.0.1:0".to_string(),
            mac: id.to_string(),
            command_tx: tx,
            latest_config: None,
        }
    }

    #[test]
    fn dispatch_sends_begin_upload_to_all_open_collectors() {
        let (tx1, mut rx1) = tokio::sync::mpsc::channel(2);
        let (tx2, mut rx2) = tokio::sync::mpsc::channel(2);
        let mut collectors = vec![mk_collector("c1", tx1), mk_collector("c2", tx2)];

        dispatch_begin_upload_session("SELECT * FROM screen".to_string(), &mut collectors)
            .expect("dispatch should succeed");

        let cmd1 = rx1
            .try_recv()
            .expect("collector 1 should receive command")
            .expect("command should be Ok");
        let cmd2 = rx2
            .try_recv()
            .expect("collector 2 should receive command")
            .expect("command should be Ok");

        assert_eq!(cmd1.r#type, CommandType::BeginUploadSession as i32);
        assert_eq!(cmd2.r#type, CommandType::BeginUploadSession as i32);
        assert!(
            cmd1.payload.contains("\"query\":\"SELECT * FROM screen\""),
            "payload should contain query"
        );
    }

    #[test]
    fn dispatch_ignores_closed_collector_channel() {
        let (tx_ok, mut rx_ok) = tokio::sync::mpsc::channel(2);
        let (tx_closed, rx_closed) = tokio::sync::mpsc::channel(2);
        drop(rx_closed);

        let mut collectors = vec![mk_collector("ok", tx_ok), mk_collector("closed", tx_closed)];

        dispatch_begin_upload_session("q".to_string(), &mut collectors)
            .expect("dispatch should succeed even with closed channel");

        let cmd = rx_ok
            .try_recv()
            .expect("open channel should still receive command")
            .expect("command should be Ok");
        assert_eq!(cmd.r#type, CommandType::BeginUploadSession as i32);
    }

    #[test]
    fn dispatch_ignores_full_collector_channel() {
        let (tx_ok, mut rx_ok) = tokio::sync::mpsc::channel(2);
        let (tx_full, mut rx_full) = tokio::sync::mpsc::channel(1);

        tx_full
            .try_send(Ok(ServerCommand {
                r#type: CommandType::PauseCapture as i32,
                payload: String::new(),
            }))
            .expect("prefill should succeed");

        let mut collectors = vec![mk_collector("ok", tx_ok), mk_collector("full", tx_full)];

        dispatch_begin_upload_session("q".to_string(), &mut collectors)
            .expect("dispatch should succeed even with full channel");

        let ok_cmd = rx_ok
            .try_recv()
            .expect("open channel should receive command")
            .expect("command should be Ok");
        assert_eq!(ok_cmd.r#type, CommandType::BeginUploadSession as i32);

        let existing = rx_full
            .try_recv()
            .expect("full channel should still only contain prefilled command")
            .expect("command should be Ok");
        assert_eq!(existing.r#type, CommandType::PauseCapture as i32);
        assert!(
            rx_full.try_recv().is_err(),
            "no additional command should be enqueued when channel is full"
        );
    }
}
