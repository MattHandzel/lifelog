use config::CollectorConfig;
use lifelog_proto::collector_service_client::CollectorServiceClient;
use lifelog_proto::lifelog_server_service_server::LifelogServerService;
use lifelog_proto::{
    Ack, Chunk, GetDataRequest, GetDataResponse, GetStateRequest, GetSystemConfigRequest,
    GetSystemConfigResponse, GetSystemStateResponse, GetUploadOffsetRequest,
    GetUploadOffsetResponse, QueryRequest, QueryResponse, RegisterCollectorRequest,
    RegisterCollectorResponse, ReportStateRequest, ReportStateResponse, SetSystemConfigRequest,
    SetSystemConfigResponse,
};
use lifelog_types::*;
use std::time;
use tokio_stream::StreamExt;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status as TonicStatus};
use tonic::{Request, Response, Status, Streaming};
use utils::ingest::ChunkIngester;

use crate::ingest::{ChunkRecord, SurrealIngestBackend};
use crate::server::ServerHandle;

pub struct GRPCServerLifelogServerService {
    pub server: ServerHandle,
}

#[tonic::async_trait]
impl LifelogServerService for GRPCServerLifelogServerService {
    async fn register_collector(
        &self,
        request: TonicRequest<RegisterCollectorRequest>,
    ) -> Result<TonicResponse<RegisterCollectorResponse>, TonicStatus> {
        let inner = request.into_inner();
        let collector_config: CollectorConfig = inner.config.unwrap().into();
        let collector_ip = format!(
            "http://{}:{}",
            collector_config.host.clone(),
            collector_config.port.clone()
        );
        println!(
            "Received a register collector request from: {:?} for collector ID: {}",
            collector_ip, collector_config.id
        );

        let endpoint = tonic::transport::Endpoint::from_shared(collector_ip.clone());
        match endpoint {
            Err(ref e) => {
                println!("Endpoint: {:?}", endpoint);
                Err(TonicStatus::internal(format!(
                    "Failed to create endpoint: {}",
                    e
                )))
            }
            Ok(endpoint) => {
                let endpoint = endpoint.connect_timeout(time::Duration::from_secs(10));

                let channel = endpoint.connect().await.map_err(|e| {
                    TonicStatus::internal(format!("Failed to connect to endpoint: {}", e))
                })?;
                let client = CollectorServiceClient::new(channel);

                let actual_mac_id = collector_config.id.clone();

                let collector = RegisteredCollector {
                    id: actual_mac_id.clone(),
                    mac: actual_mac_id.clone(),
                    address: collector_ip.to_string(),
                    grpc_client: client.clone(),
                };

                self.server.register_collector(collector.clone()).await;

                println!("Registering collector: {:?}", collector);

                Ok(TonicResponse::new(RegisterCollectorResponse {
                    success: true,
                    session_id: chrono::Utc::now().timestamp_subsec_nanos() as u64
                        + chrono::Utc::now().timestamp() as u64,
                }))
            }
        }
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
        println!("Received a set config request!");
        Ok(TonicResponse::new(SetSystemConfigResponse::default()))
    }

    async fn query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        let query_message = request.into_inner().query;
        println!("[QUERY] Received a query request: {:?}", query_message);
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
        println!(
            "[SERVER QUERY] Responding to QueryRequest with {} keys",
            response.keys.len()
        );
        Ok(Response::new(response))
    }

    async fn get_data(
        &self,
        request: Request<GetDataRequest>,
    ) -> Result<Response<GetDataResponse>, Status> {
        let inner_request = request.into_inner();
        println!(
            "[SERVER GET_DATA] Received GetDataRequest with {} keys",
            inner_request.keys.len()
        );

        let _server_arc = self.server.clone(); // Clone Arc for use in spawn_blocking
        let keys = inner_request.keys;

        let data: Vec<lifelog_proto::LifelogData> = self
            .server
            .get_data(
                keys.iter()
                    .map(|k| LifelogFrameKey::from(k.clone()))
                    .collect(),
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to get data: {}", e)))?
            .iter()
            .map(|v| lifelog_proto::LifelogData::from(v.clone()))
            .collect();

        let response = GetDataResponse { data: data };
        println!(
            "[SERVER GET_DATA] Responding to GetDataRequest with {} data items",
            response.data.len()
        );
        Ok(Response::new(response))
    }

    // TODO: Refactor ALL functions to include this check to see if the thing doing the requesting
    // has been registered or not. I want to refactor my server to only think about clients.
    async fn report_state(
        &self,
        _request: tonic::Request<ReportStateRequest>,
    ) -> Result<TonicResponse<ReportStateResponse>, TonicStatus> {
        let state = _request.into_inner().state.unwrap();
        println!(
            "Received a report state request! {} {:?}",
            state.name,
            state.timestamp.unwrap()
        );
        // Ensure we got a state reported by a registered collector, if not then we ignore it
        match self.server.contains_collector(state.name.clone()).await {
            true => {
                let _ = self
                    .server
                    .report_collector_state(state.clone().into())
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
        println!("Received a get state request!");
        let state = self.server.get_state().await;
        //let proto_state = s
        Ok(TonicResponse::new(GetSystemStateResponse {
            state: Some(state.into()), // TODO: Replace this with the system state
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
