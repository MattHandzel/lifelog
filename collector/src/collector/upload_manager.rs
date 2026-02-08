use crate::collector::Collector;
use crate::modules::data_source::BufferedSource;
use lifelog_types::lifelog_server_service_client::LifelogServerServiceClient;
use lifelog_types::{Chunk, GetUploadOffsetRequest, StreamIdentity};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use tokio::sync::mpsc;

pub struct UploadManager {
    server_address: String,
    collector_id: String,
    session_id: u64,
    trigger_rx: mpsc::Receiver<()>,
}

impl UploadManager {
    pub fn new(server_address: String, collector_id: String) -> (Self, mpsc::Sender<()>) {
        let session_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let (tx, rx) = mpsc::channel(1);

        (
            Self {
                server_address,
                collector_id,
                session_id,
                trigger_rx: rx,
            },
            tx,
        )
    }

    pub async fn run(mut self, collector: Arc<RwLock<Collector>>) {
        tracing::info!("UploadManager: Started");
        loop {
            // Wait for trigger or timeout (periodic sync)
            let _ = tokio::time::timeout(Duration::from_secs(30), self.trigger_rx.recv()).await;

            if let Err(e) = self.perform_upload_cycle(collector.clone()).await {
                tracing::error!(error = %e, "UploadManager: Upload cycle failed");
            }
        }
    }

    async fn perform_upload_cycle(
        &self,
        collector_arc: Arc<RwLock<Collector>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sources: Vec<Arc<dyn BufferedSource>> = {
            let collector = collector_arc.read().await;
            collector.get_buffered_sources().await
        };

        if sources.is_empty() {
            tracing::info!("UploadManager: No buffered sources found");
            return Ok(());
        }

        tracing::info!(
            count = sources.len(),
            "UploadManager: Starting upload cycle for buffered sources"
        );

        let mut client = LifelogServerServiceClient::connect(self.server_address.clone()).await?;

        for source in sources {
            let stream_id = source.stream_id();

            // 1. Get current offset from server
            let stream_identity = StreamIdentity {
                collector_id: self.collector_id.clone(),
                stream_id: stream_id.clone(),
                session_id: self.session_id,
            };

            let req = GetUploadOffsetRequest {
                stream: Some(stream_identity.clone()),
            };

            let resp = client.get_upload_offset(req).await?;
            let mut server_offset = resp.into_inner().offset;

            tracing::info!(stream = %stream_id, server_offset, "UploadManager: Syncing stream");

            // 2. Upload in batches
            loop {
                let (next_wal_offset, batch) = source.peek_upload_batch(10).await?;
                if batch.is_empty() {
                    break;
                }

                let mut chunks = Vec::new();
                let mut current_chunk_offset = server_offset;

                for data in batch {
                    let hash = format!("{:x}", Sha256::digest(&data));
                    chunks.push(Chunk {
                        stream: Some(stream_identity.clone()),
                        offset: current_chunk_offset,
                        data: data.clone(),
                        hash,
                    });
                    current_chunk_offset += data.len() as u64;
                }

                // Stream chunks to server
                let stream = tokio_stream::iter(chunks);
                let upload_resp = client.upload_chunks(stream).await?;
                let ack = upload_resp.into_inner();

                if ack.acked_offset > server_offset {
                    tracing::info!(stream = %stream_id, acked = ack.acked_offset, "UploadManager: Batch uploaded and ACKed");
                    server_offset = ack.acked_offset;
                    // In our current DiskBuffer, commit_offset takes a BYTE offset in the WAL,
                    // not the logical data offset.
                    // peek_upload_batch returns the next WAL offset.
                    source.commit_upload(next_wal_offset).await?;
                } else {
                    tracing::warn!(stream = %stream_id, "UploadManager: Server did not ACK forward. Stopping stream upload.");
                    break;
                }
            }
        }

        Ok(())
    }
}
