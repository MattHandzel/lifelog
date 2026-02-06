use lifelog_proto::DataModality;
use lifelog_proto::SystemState;
use lifelog_core::*;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::query::get_all_uuids_from_origin;
use crate::server::RegisteredCollector;

pub(crate) async fn sync_data_with_collectors(
    _state: SystemState,
    _db: &Surreal<Client>,
    _query: String,
    _collectors: &mut Vec<RegisteredCollector>,
) -> Result<(), LifelogError> {
    // TODO: Implement data synchronization via ServerCommand ("BeginUploadSession") 
    // and UploadChunks stream. The old "Dial-Back" logic below is removed.
    tracing::warn!("sync_data_with_collectors is currently a stub after ControlPlane refactor");
    Ok(())
}

pub(crate) async fn get_keys_in_source_not_in_destination(
    db: &Surreal<Client>,
    source: DataOrigin,
    destination: DataOrigin,
) -> Vec<LifelogFrameKey> {
    let uuids_from_source = match get_all_uuids_from_origin(db, &source).await {
        Ok(uuids) => uuids,
        Err(_) => return vec![],
    };

    let uuids_from_destination = match get_all_uuids_from_origin(db, &destination).await {
        Ok(uuids) => uuids,
        Err(_) => return vec![],
    };

    uuids_from_source
        .into_iter()
        .filter(|uuid| !uuids_from_destination.contains(uuid))
        .map(|uuid| LifelogFrameKey::new(uuid, source.clone()))
        .collect()
}