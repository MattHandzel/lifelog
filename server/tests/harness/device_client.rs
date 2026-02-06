//! `DeviceClient` wraps the gRPC client with a device identity and provides
//! ergonomic helpers for the register → upload → verify workflow.

#![allow(dead_code)]

use lifelog_proto::lifelog_server_service_client::LifelogServerServiceClient;
use lifelog_proto::{
    control_message, Chunk, CollectorConfig, CollectorState, ControlMessage,
    GetUploadOffsetRequest, RegisterCollectorRequest, ReportStateRequest, ServerCommand,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;
use tonic::Streaming;

/// A simulated device that talks to the lifelog server over gRPC.
pub struct DeviceClient {
    pub device_id: String,
    client: LifelogServerServiceClient<Channel>,
    /// Sender side of the ControlStream (kept alive to maintain the bidi stream).
    control_tx: Option<mpsc::Sender<ControlMessage>>,
    /// Receiver side of server commands from the ControlStream.
    #[allow(dead_code)]
    server_commands: Option<Streaming<ServerCommand>>,
}

impl DeviceClient {
    pub fn new(device_id: impl Into<String>, client: LifelogServerServiceClient<Channel>) -> Self {
        Self {
            device_id: device_id.into(),
            client,
            control_tx: None,
            server_commands: None,
        }
    }

    /// Open a ControlStream and send a Register message.
    pub async fn register(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = mpsc::channel(32);

        let config = CollectorConfig {
            id: self.device_id.clone(),
            host: "127.0.0.1".to_string(),
            port: 0,
            ..Default::default()
        };

        let reg_msg = ControlMessage {
            collector_id: self.device_id.clone(),
            msg: Some(control_message::Msg::Register(RegisterCollectorRequest {
                config: Some(config),
            })),
        };
        tx.send(reg_msg).await?;

        let response = self.client.control_stream(ReceiverStream::new(rx)).await?;

        self.control_tx = Some(tx);
        self.server_commands = Some(response.into_inner());
        Ok(())
    }

    /// Report device state via the ControlStream.
    #[allow(dead_code)]
    pub async fn report_state(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let tx = self
            .control_tx
            .as_ref()
            .ok_or("not registered — call register() first")?;

        let state = CollectorState {
            name: self.device_id.clone(),
            ..Default::default()
        };
        let msg = ControlMessage {
            collector_id: self.device_id.clone(),
            msg: Some(control_message::Msg::State(ReportStateRequest {
                state: Some(state),
            })),
        };
        tx.send(msg).await?;
        Ok(())
    }

    /// Upload a sequence of pre-built chunks via client-streaming `UploadChunks`.
    /// Returns the `Ack` with the final acked offset.
    pub async fn upload_chunks(
        &mut self,
        chunks: Vec<Chunk>,
    ) -> Result<lifelog_proto::Ack, tonic::Status> {
        let stream = tokio_stream::iter(chunks);
        let response = self.client.upload_chunks(stream).await?;
        Ok(response.into_inner())
    }

    /// Query the server for the current upload offset.
    #[allow(dead_code)]
    pub async fn get_offset(
        &mut self,
        stream_id: &str,
        session_id: u64,
    ) -> Result<u64, tonic::Status> {
        let resp = self
            .client
            .get_upload_offset(GetUploadOffsetRequest {
                stream: Some(lifelog_proto::StreamIdentity {
                    collector_id: self.device_id.clone(),
                    stream_id: stream_id.to_string(),
                    session_id,
                }),
            })
            .await?;
        Ok(resp.into_inner().offset)
    }

    /// Get a clone of the underlying gRPC client for advanced operations.
    #[allow(dead_code)]
    pub fn raw_client(&self) -> LifelogServerServiceClient<Channel> {
        self.client.clone()
    }
}
