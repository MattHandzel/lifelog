use crate::ingest::UnifiedIngestBackend;
use crate::server::ServerHandle;
use chrono::Utc;
use futures_core::Stream;
use lifelog_types::lifelog_server_service_server::LifelogServerService;
use lifelog_types::*;
use std::pin::Pin;
use std::time::Duration;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
use tonic::{Request, Response, Status, Streaming};
use utils::ingest::ChunkIngester;

pub struct GRPCServerLifelogServerService {
    pub server: ServerHandle,
}

impl GRPCServerLifelogServerService {
    fn check_auth(&self, metadata: &tonic::metadata::MetadataMap) -> Result<(), Status> {
        let auth_token = std::env::var("LIFELOG_AUTH_TOKEN")
            .map_err(|_| Status::internal("LIFELOG_AUTH_TOKEN must be configured"))?;

        let expected = format!("Bearer {}", auth_token);

        match metadata.get("authorization") {
            Some(t) => {
                let token_str = t.to_str().unwrap_or_default();
                if constant_time_eq(token_str.as_bytes(), expected.as_bytes()) {
                    return Ok(());
                }
                tracing::warn!("Invalid authentication token provided");
                Err(Status::unauthenticated("Invalid token"))
            }
            None => {
                tracing::warn!("Unauthenticated connection attempt (no token provided)");
                Err(Status::unauthenticated("No token provided"))
            }
        }
    }
}

#[tonic::async_trait]
impl LifelogServerService for GRPCServerLifelogServerService {
    type ControlStreamStream = Pin<Box<dyn Stream<Item = Result<ServerCommand, Status>> + Send>>;

    async fn control_stream(
        &self,
        request: Request<Streaming<ControlMessage>>,
    ) -> Result<Response<Self::ControlStreamStream>, Status> {
        self.check_auth(request.metadata())?;
        let mut stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(128);
        let server_handle = self.server.clone();

        tokio::spawn(async move {
            while let Some(msg_result) = stream.next().await {
                match msg_result {
                    Ok(msg) => {
                        let collector_id = msg.collector_id.clone();
                        match msg.msg {
                            Some(lifelog_types::control_message::Msg::Register(reg)) => {
                                tracing::info!(id = %collector_id, "Registering collector");
                                let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel(32);
                                let cmd_tx_clock = cmd_tx.clone();
                                let registered = crate::server::RegisteredCollector {
                                    id: collector_id.clone(),
                                    address: "unknown".to_string(), // TODO: extract from peer
                                    mac: collector_id.clone(),
                                    command_tx: cmd_tx,
                                    latest_config: reg.config,
                                };
                                server_handle.register_collector(registered).await;

                                // Forward commands from server to this collector stream
                                let tx_clone = tx.clone();
                                tokio::spawn(async move {
                                    while let Some(cmd) = cmd_rx.recv().await {
                                        let _ = tx_clone.send(cmd).await;
                                    }
                                });

                                // Periodic clock-sync pings to improve skew estimation.
                                // This exercises the command channel and allows collectors to report
                                // (device_now, backend_now) pairs.
                                tokio::spawn(async move {
                                    let mut interval =
                                        tokio::time::interval(Duration::from_secs(30));
                                    loop {
                                        interval.tick().await;
                                        let backend_now = Utc::now().to_rfc3339();
                                        let cmd = Ok(ServerCommand {
                                            r#type: CommandType::ClockSync as i32,
                                            payload: backend_now,
                                        });
                                        if cmd_tx_clock.send(cmd).await.is_err() {
                                            break;
                                        }
                                    }
                                });
                            }
                            Some(lifelog_types::control_message::Msg::State(report)) => {
                                if let Some(state) = report.state {
                                    server_handle.report_collector_state(state).await;
                                }
                            }
                            Some(lifelog_types::control_message::Msg::ClockSample(sample)) => {
                                let backend_now = sample.backend_now.and_then(|ts| {
                                    chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                                });

                                if let Some(ts) = sample.device_now {
                                    if let Some(device_now) = chrono::DateTime::from_timestamp(
                                        ts.seconds,
                                        ts.nanos as u32,
                                    ) {
                                        server_handle
                                            .handle_clock_sample(
                                                &collector_id,
                                                device_now,
                                                backend_now,
                                            )
                                            .await;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        tracing::error!("Control stream error: {}", e);
                        break;
                    }
                }
            }
            tracing::warn!("Collector disconnected from ControlStream");
        });

        Ok(Response::new(
            Box::pin(ReceiverStream::new(rx)) as Self::ControlStreamStream
        ))
    }

    async fn get_config(
        &self,
        request: Request<GetSystemConfigRequest>,
    ) -> Result<Response<GetSystemConfigResponse>, Status> {
        self.check_auth(request.metadata())?;
        let config = self.server.get_config().await;
        Ok(Response::new(GetSystemConfigResponse {
            config: Some(config),
        }))
    }

    async fn set_config(
        &self,
        request: Request<SetSystemConfigRequest>,
    ) -> Result<Response<SetSystemConfigResponse>, Status> {
        self.check_auth(request.metadata())?;
        tracing::info!("Received set config request");
        let system_config = request
            .into_inner()
            .config
            .ok_or_else(|| Status::invalid_argument("missing system config payload"))?;

        self.server
            .apply_system_config(system_config)
            .await
            .map_err(|e| Status::internal(format!("failed to apply config: {e}")))?;

        Ok(Response::new(SetSystemConfigResponse { success: true }))
    }

    async fn query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        self.check_auth(request.metadata())?;
        let query_message = request.into_inner().query.unwrap_or_default();
        tracing::info!(query = ?query_message, "Received query request");

        let keys = self
            .server
            .process_query(query_message)
            .await
            .map_err(|e| Status::internal(format!("Failed to process query: {}", e)))?;
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

    async fn replay(
        &self,
        request: Request<ReplayRequest>,
    ) -> Result<Response<ReplayResponse>, Status> {
        self.check_auth(request.metadata())?;
        let req = request.into_inner();
        tracing::info!(req = ?req, "Received replay request");

        let resp = self
            .server
            .process_replay(req)
            .await
            .map_err(|e| Status::internal(format!("Failed to process replay: {}", e)))?;

        Ok(Response::new(resp))
    }

    async fn get_data(
        &self,
        request: Request<GetDataRequest>,
    ) -> Result<Response<GetDataResponse>, Status> {
        self.check_auth(request.metadata())?;
        let inner_request = request.into_inner();
        tracing::info!(count = inner_request.keys.len(), "Received GetDataRequest");

        let keys = inner_request.keys;
        let data = self
            .server
            .get_data(keys)
            .await
            .map_err(|e| Status::internal(format!("Failed to get data: {}", e)))?;

        let response = GetDataResponse { data };
        tracing::info!(count = response.data.len(), "GetData response");
        Ok(Response::new(response))
    }

    async fn get_state(
        &self,
        request: Request<GetStateRequest>,
    ) -> Result<Response<GetSystemStateResponse>, Status> {
        self.check_auth(request.metadata())?;
        tracing::debug!("Received get state request");
        let state = self.server.get_state().await;
        Ok(Response::new(GetSystemStateResponse { state: Some(state) }))
    }

    async fn upload_chunks(
        &self,
        request: Request<Streaming<Chunk>>,
    ) -> Result<Response<Ack>, Status> {
        self.check_auth(request.metadata())?;
        let mut stream = request.into_inner();
        let mut ingester: Option<ChunkIngester<UnifiedIngestBackend>> = None;
        let mut last_stream: Option<lifelog_types::StreamIdentity> = None;
        let mut stream_id_str: Option<String> = None;
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
                stream_id_str = Some(stream_id.stream_id.clone());

                let origin = &stream_id.collector_id;
                if origin.len() > 200 {
                    return Err(Status::invalid_argument(format!(
                        "origin too long: {} chars (max 200)",
                        origin.len()
                    )));
                }
                if origin.len() > 100 {
                    tracing::warn!(
                        origin_len = origin.len(),
                        origin = %origin,
                        "origin string exceeds 100 chars"
                    );
                }

                let server = self.server.server.read().await;
                let transforms = server.config.read().await.transforms.clone();
                let pool = server.postgres_pool.clone();
                let backend = UnifiedIngestBackend {
                    pool,
                    cas: server.cas.clone(),
                    skew_estimates: server.skew_estimates.clone(),
                    transforms,
                };
                ingester = Some(ChunkIngester::new(
                    backend,
                    server.cas.clone(),
                    stream_id.collector_id.clone(),
                    stream_id.stream_id.clone(),
                    stream_id.session_id,
                    0, // start_offset
                ));
            }

            if let Some(ref mut ing) = ingester {
                match ing
                    .apply_chunk(chunk.offset, &chunk.data, &chunk.hash)
                    .await
                {
                    Ok(_) => {
                        // Spec §6.2.1: ACK implies "fully queryable".
                        // If async work (e.g. derived OCR) is not complete, keep ACK pinned.
                        if ing.is_chunk_indexed(chunk.offset).await {
                            last_acked_offset = chunk.offset + chunk.data.len() as u64;
                        } else {
                            tracing::warn!(
                                stream_id = %stream_id_str.as_deref().unwrap_or_default(),
                                offset = chunk.offset,
                                "chunk persisted but not yet queryable; withholding ACK advance"
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("Ingest error: {}", e);
                        return Err(Status::internal(format!("Ingest error: {}", e)));
                    }
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
        self.check_auth(request.metadata())?;
        let _inner = request.into_inner();
        let stream_id = _inner
            .stream
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("missing stream identity"))?;
        let pool = {
            let server = self.server.server.read().await;
            server.postgres_pool.clone()
        };

        let client = pool
            .get()
            .await
            .map_err(|e| Status::internal(format!("Postgres pool error: {e}")))?;
        let row = client
            .query_opt(
                "SELECT \"offset\", length
                 FROM upload_chunks
                 WHERE collector_id = $1 AND stream_id = $2 AND session_id = $3
                 ORDER BY \"offset\" DESC
                 LIMIT 1",
                &[
                    &stream_id.collector_id,
                    &stream_id.stream_id,
                    &(stream_id.session_id as i64),
                ],
            )
            .await
            .map_err(|e| Status::internal(format!("Postgres query error: {e}")))?;

        let offset = row
            .map(|r| {
                let offset: i64 = r.get(0);
                let length: i32 = r.get(1);
                offset as u64 + length as u64
            })
            .unwrap_or(0);

        Ok(Response::new(GetUploadOffsetResponse { offset }))
    }

    async fn list_modalities(
        &self,
        request: Request<ListModalitiesRequest>,
    ) -> Result<Response<ListModalitiesResponse>, Status> {
        self.check_auth(request.metadata())?;
        use lifelog_types::{FieldDescriptor, ModalityDescriptor};

        fn field(
            name: &str,
            ty: &str,
            display: &str,
            primary: bool,
            searchable: bool,
        ) -> FieldDescriptor {
            FieldDescriptor {
                name: name.to_string(),
                ty: ty.to_string(),
                display: display.to_string(),
                primary,
                searchable,
            }
        }

        fn modality(
            name: &str,
            stream_id: &str,
            category: &str,
            fields: Vec<FieldDescriptor>,
        ) -> ModalityDescriptor {
            ModalityDescriptor {
                name: name.to_string(),
                stream_id: stream_id.to_string(),
                category: category.to_string(),
                fields,
            }
        }

        let mut modalities = Vec::new();

        // Add dynamically discovered Postgres origins
        let pg_origins = self.server.list_postgres_origins().await;
        for origin in pg_origins {
            let device_id = match &origin.origin {
                lifelog_core::DataOriginType::DeviceId(id) => id.clone(),
                _ => "unknown".to_string(),
            };
            let stream_id = format!("{}:{}", device_id, origin.modality_name.to_lowercase());
            let name = format!("{} ({})", origin.modality_name, device_id);
            modalities.push(modality(
                &name,
                &stream_id,
                "remote",
                vec![field("timestamp", "timestamp", "Captured At", true, false)],
            ));
        }

        Ok(Response::new(ListModalitiesResponse { modalities }))
    }

    async fn pair_collector(
        &self,
        request: Request<PairCollectorRequest>,
    ) -> Result<Response<PairCollectorResponse>, Status> {
        let client_hint = request
            .metadata()
            .get("x-lifelog-client-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim().replace(':', ""))
            .filter(|s| !s.is_empty());
        let req = request.into_inner();
        let expected_token = std::env::var("LIFELOG_ENROLLMENT_TOKEN").unwrap_or_default();
        if expected_token.is_empty() {
            tracing::warn!("Server has no enrollment token configured");
            return Err(Status::internal("Enrollment not configured on server"));
        }

        if req.enrollment_token != expected_token {
            tracing::warn!("Invalid enrollment token attempt");
            return Err(Status::permission_denied("Invalid enrollment token"));
        }

        let collector_id =
            client_hint.unwrap_or_else(|| lifelog_core::uuid::Uuid::new_v4().to_string());
        tracing::info!(%collector_id, "Successfully paired collector");

        Ok(Response::new(PairCollectorResponse { collector_id }))
    }
}
