use lifelog_proto::lifelog_server_service_server::LifelogServerService;
use lifelog_proto::{
    Ack, Chunk, ControlMessage, GetDataRequest, GetDataResponse, GetStateRequest,
    GetSystemConfigRequest, GetSystemConfigResponse, GetSystemStateResponse,
    GetUploadOffsetRequest, GetUploadOffsetResponse, QueryRequest, QueryResponse,
    RegisterCollectorRequest, RegisterCollectorResponse, ReportStateRequest, ReportStateResponse,
    ServerCommand, SetSystemConfigRequest, SetSystemConfigResponse,
};
use lifelog_core::*;
use std::time;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status as TonicStatus};
use tonic::{Request, Response, Status, Streaming};
use utils::ingest::ChunkIngester;

use crate::ingest::{ChunkRecord, SurrealIngestBackend};
use crate::server::{ServerHandle, RegisteredCollector};

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
                                lifelog_proto::control_message::Msg::Register(reg) => {
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
                                lifelog_proto::control_message::Msg::State(state_req) => {
                                    if let Some(state) = state_req.state {
                                        let _ = server_handle.report_collector_state(state).await;
                                    }
                                }
                                lifelog_proto::control_message::Msg::Heartbeat(_) => {
                                    tracing::debug!(id = ?collector_id, "Heartbeat received");
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

    async fn register_collector(
        &self,
        request: TonicRequest<RegisterCollectorRequest>,
    ) -> Result<TonicResponse<RegisterCollectorResponse>, TonicStatus> {
        // Legacy unary registration — kept for backward compatibility but discouraged
        let inner = request.into_inner();
        let config = inner
            .config
            .ok_or_else(|| TonicStatus::invalid_argument("missing config"))?;

        tracing::warn!(id = %config.id, "Legacy RegisterCollector called, use ControlStream instead");

        let (tx, _rx) = mpsc::channel(1); // Dummy channel for legacy
        let collector = RegisteredCollector {
            id: config.id.clone(),
            mac: config.id.clone(),
            address: String::new(),
            command_tx: tx,
            latest_config: Some(config),
        };

        self.server.register_collector(collector).await;

        Ok(TonicResponse::new(RegisterCollectorResponse {
            success: true,
            session_id: chrono::Utc::now().timestamp() as u64,
        }))
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
            config: Some(system_config.into()),
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
        let query_message = request.into_inner().query;
        tracing::info!(query = ?query_message, "Received query request");
        let _server_arc = self.server.clone(); // Clone Arc for use in spawn_blocking

        let _uuids: Vec<LifelogFrameKey> = vec![];
        // NOTE: Right now we just return all uuids for a query, in the future actually parse the
        // query message and return the uuids that match
        let query = String::from("");
        let keys = self
            .server
            .process_query(query.clone())
            .await
            .map_err(|e| {
                tonic::Status::internal(format!("Failed to process query: {}", e.to_string()))
            })?;
        let proto_keys: Vec<lifelog_proto::LifelogDataKey> = keys
            .iter()
            .map(|key| lifelog_proto::LifelogDataKey {
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

        let data: Vec<lifelog_proto::LifelogData> = self
            .server
            .get_data(
                keys.iter()
                    .map(|k| k.clone().into())
                    .collect(),
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to get data: {}", e)))?
            .iter()
            .map(|v| lifelog_proto::LifelogData::from(v.clone()))
            .collect();

        let response = GetDataResponse { data: data };
        tracing::info!(count = response.data.len(), "GetData response");
        Ok(Response::new(response))
    }

    // TODO: Refactor ALL functions to include this check to see if the thing doing the requesting
    // has been registered or not. I want to refactor my server to only think about clients.
    async fn report_state(
        &self,
        _request: tonic::Request<ReportStateRequest>,
    ) -> Result<TonicResponse<ReportStateResponse>, TonicStatus> {
        let state = _request
            .into_inner()
            .state
            .ok_or_else(|| TonicStatus::invalid_argument("missing state"))?;
        // Ensure we got a state reported by a registered collector, if not then we ignore it
        match self.server.contains_collector(state.name.clone()).await {
            true => {
                let _ = self
                    .server
                    .report_collector_state(state)
                    .await;
                Ok(TonicResponse::new(ReportStateResponse {
                    acknowledged: true,
                }))
            }
            false => Err(TonicStatus::internal(format!(
                "Collector {} is not registered",
                state.name
            ))),
        }
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
        let mut last_session_info = (String::new(), String::new(), 0u64);
        let mut last_acked_offset = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk =
                chunk_result.map_err(|e| Status::internal(format!("Stream error: {}", e)))?;

            if ingester.is_none() {
                last_session_info = (
                    chunk.collector_id.clone(),
                    chunk.stream_id.clone(),
                    chunk.session_id,
                );
                let server = self.server.server.read().await;
                let backend = SurrealIngestBackend {
                    db: server.db.clone(),
                };
                let cas = server.cas.clone();
                ingester = Some(ChunkIngester::new(
                    backend,
                    cas,
                    chunk.collector_id.clone(),
                    chunk.stream_id.clone(),
                    chunk.session_id,
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
            collector_id: last_session_info.0,
            stream_id: last_session_info.1,
            session_id: last_session_info.2,
            acked_offset: last_acked_offset,
        }))
    }

    async fn get_upload_offset(
        &self,
        request: Request<GetUploadOffsetRequest>,
    ) -> Result<Response<GetUploadOffsetResponse>, Status> {
        let _inner = request.into_inner();
        let db = self.server.server.read().await.db.clone();

        // Find the highest offset for this session in the 'upload_chunks' table
        let mut response = db.query("SELECT * FROM upload_chunks WHERE collector_id = $c AND stream_id = $s AND session_id = $sess ORDER BY offset DESC LIMIT 1")
            .bind(("c", _inner.collector_id.clone()))
            .bind(("s", _inner.stream_id.clone()))
            .bind(("sess", _inner.session_id))
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        let records: Vec<ChunkRecord> = response
            .take(0)
            .map_err(|e| Status::internal(format!("Parse error: {}", e)))?;

        let offset = records.first().map(|r| r.offset + r.length).unwrap_or(0);

        Ok(Response::new(GetUploadOffsetResponse { offset }))
    }
}
