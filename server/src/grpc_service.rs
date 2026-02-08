use lifelog_core::*;
use lifelog_types::lifelog_server_service_server::LifelogServerService;
use lifelog_types::{
    Ack, Chunk, ControlMessage, GetDataRequest, GetDataResponse, GetStateRequest,
    GetSystemConfigRequest, GetSystemConfigResponse, GetSystemStateResponse,
    GetUploadOffsetRequest, GetUploadOffsetResponse, QueryRequest, QueryResponse, ServerCommand,
    SetSystemConfigRequest, SetSystemConfigResponse,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};
use tonic::{Response as TonicResponse, Status as TonicStatus};
use utils::ingest::ChunkIngester;

use crate::ingest::{ChunkRecord, SurrealIngestBackend};
use crate::server::{RegisteredCollector, ServerHandle};

pub struct GRPCServerLifelogServerService {
    pub server: ServerHandle,
}

#[tonic::async_trait]
impl LifelogServerService for GRPCServerLifelogServerService {
    type ControlStreamStream = ReceiverStream<Result<ServerCommand, Status>>;

    async fn control_stream(
        &self,
        request: Request<Streaming<ControlMessage>>,
    ) -> Result<Response<Self::ControlStreamStream>, Status> {
        let mut in_stream = request.into_inner();
        let (tx, rx) = mpsc::channel::<Result<ServerCommand, Status>>(128);
        let server_handle = self.server.clone();

        tokio::spawn(async move {
            let mut collector_id: Option<String> = None;

            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(msg) => {
                        collector_id = Some(msg.collector_id.clone());
                        if let Some(payload) = msg.msg {
                            match payload {
                                lifelog_types::control_message::Msg::Register(reg) => {
                                    if let Some(config) = reg.config {
                                        let registered = RegisteredCollector {
                                            id: config.id.clone(),
                                            mac: config.id.clone(),
                                            address: String::new(), // Address no longer needed for dial-back
                                            command_tx: tx.clone(),
                                            latest_config: Some(config),
                                        };
                                        server_handle.register_collector(registered).await;
                                        tracing::info!(id = %msg.collector_id, "Collector registered via ControlStream");
                                    }
                                }
                                lifelog_types::control_message::Msg::State(state_req) => {
                                    if let Some(state) = state_req.state {
                                        let _ = server_handle.report_collector_state(state).await;
                                    }
                                }
                                lifelog_types::control_message::Msg::Heartbeat(_) => {
                                    tracing::debug!(id = ?collector_id, "Heartbeat received");
                                }
                                lifelog_types::control_message::Msg::SuggestUpload(suggest) => {
                                    tracing::info!(id = %msg.collector_id, ?suggest, "SuggestUpload received");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("ControlStream error: {}", e);
                        break;
                    }
                }
            }

            if let Some(id) = collector_id {
                server_handle.remove_collector(&id).await;
                tracing::info!(id = %id, "Collector ControlStream closed");
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn get_config(
        &self,
        _request: tonic::Request<GetSystemConfigRequest>,
    ) -> Result<TonicResponse<GetSystemConfigResponse>, TonicStatus> {
        let system_config =
            self.server.get_system_config().await.map_err(|e| {
                TonicStatus::internal(format!("Failed to get system config: {}", e))
            })?;
        Ok(TonicResponse::new(GetSystemConfigResponse {
            config: Some(system_config),
        }))
    }

    async fn set_config(
        &self,
        _request: tonic::Request<SetSystemConfigRequest>,
    ) -> Result<TonicResponse<SetSystemConfigResponse>, TonicStatus> {
        tracing::info!("Received set config request");
        Ok(TonicResponse::new(SetSystemConfigResponse::default()))
    }

    async fn query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        let query_message = request.into_inner().query.unwrap_or_default();
        tracing::info!(query = ?query_message, "Received query request");

        let keys = self
            .server
            .process_query(query_message)
            .await
            .map_err(|e| tonic::Status::internal(format!("Failed to process query: {}", e)))?;
        let proto_keys: Vec<lifelog_types::LifelogDataKey> = keys
            .iter()
            .map(|key| lifelog_types::LifelogDataKey {
                uuid: key.uuid.to_string(),
                origin: key.origin.get_table_name(),
            })
            .collect();
        let response = QueryResponse { keys: proto_keys };
        tracing::info!(count = response.keys.len(), "Query response");
        Ok(Response::new(response))
    }

    async fn get_data(
        &self,
        request: Request<GetDataRequest>,
    ) -> Result<Response<GetDataResponse>, Status> {
        let inner_request = request.into_inner();
        tracing::info!(count = inner_request.keys.len(), "Received GetDataRequest");

        let _server_arc = self.server.clone(); // Clone Arc for use in spawn_blocking
        let keys = inner_request.keys;

        let data: Vec<lifelog_types::LifelogData> = self
            .server
            .get_data(
                keys.into_iter()
                    .map(|k| k.try_into())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| Status::invalid_argument(format!("Invalid key: {e}")))?,
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to get data: {}", e)))?
            .to_vec();

        let response = GetDataResponse { data };
        tracing::info!(count = response.data.len(), "GetData response");
        Ok(Response::new(response))
    }

    async fn get_state(
        &self,
        _request: tonic::Request<GetStateRequest>,
    ) -> Result<TonicResponse<GetSystemStateResponse>, TonicStatus> {
        tracing::debug!("Received get state request");
        let state = self.server.get_state().await;
        //let proto_state = s
        Ok(TonicResponse::new(GetSystemStateResponse {
            state: Some(state), // TODO: Replace this with the system state
                                // instead of the server state (i need some work with the proto files)
        }))
    }

    async fn upload_chunks(
        &self,
        request: Request<Streaming<Chunk>>,
    ) -> Result<Response<Ack>, Status> {
        let mut stream = request.into_inner();
        let mut ingester: Option<ChunkIngester<SurrealIngestBackend>> = None;
        let mut last_stream: Option<lifelog_types::StreamIdentity> = None;
        let mut last_acked_offset = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk =
                chunk_result.map_err(|e| Status::internal(format!("Stream error: {}", e)))?;

            if ingester.is_none() {
                let stream_id = chunk
                    .stream
                    .as_ref()
                    .ok_or_else(|| Status::invalid_argument("missing stream identity"))?;
                last_stream = Some(stream_id.clone());

                let server = self.server.server.read().await;
                let backend = SurrealIngestBackend {
                    db: server.db.clone(),
                };
                let cas = server.cas.clone();
                ingester = Some(ChunkIngester::new(
                    backend,
                    cas,
                    stream_id.collector_id.clone(),
                    stream_id.stream_id.clone(),
                    stream_id.session_id,
                    chunk.offset,
                ));
            }

            if let Some(ref mut ing) = ingester {
                let next_offset = ing
                    .apply_chunk(chunk.offset, &chunk.data, &chunk.hash)
                    .await
                    .map_err(|e| Status::invalid_argument(format!("Ingest error: {}", e)))?;

                // REQ-014: ACK only if fully indexed (durable ACK gate)
                if ing.is_chunk_indexed(chunk.offset).await {
                    last_acked_offset = next_offset;
                }
            }
        }

        Ok(Response::new(Ack {
            stream: last_stream,
            acked_offset: last_acked_offset,
        }))
    }

    async fn get_upload_offset(
        &self,
        request: Request<GetUploadOffsetRequest>,
    ) -> Result<Response<GetUploadOffsetResponse>, Status> {
        let _inner = request.into_inner();
        let stream_id = _inner
            .stream
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("missing stream identity"))?;
        let db = self.server.server.read().await.db.clone();

        // Find the highest offset for this session in the 'upload_chunks' table
        let mut response = db.query("SELECT * FROM upload_chunks WHERE collector_id = $c AND stream_id = $s AND session_id = $sess ORDER BY offset DESC LIMIT 1")
            .bind(("c", stream_id.collector_id.clone()))
            .bind(("s", stream_id.stream_id.clone()))
            .bind(("sess", stream_id.session_id))
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        #[derive(serde::Deserialize)]
        struct RawOffset {
            offset: u64,
            length: u64,
        }

        let records: Vec<RawOffset> = response
            .take(0)
            .map_err(|e| Status::internal(format!("Parse error: {}", e)))?;

        let offset = records.first().map(|r| r.offset + r.length).unwrap_or(0);

        Ok(Response::new(GetUploadOffsetResponse { offset }))
    }
}
