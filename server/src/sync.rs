use lifelog_core::*;
use lifelog_types::SystemState;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::server::RegisteredCollector;

pub(crate) async fn sync_data_with_collectors(
    _state: SystemState,
    _db: &Surreal<Client>,
    _query: String,
    _collectors: &mut [RegisteredCollector],
) -> Result<(), LifelogError> {
    // TODO: Implement data synchronization via ServerCommand ("BeginUploadSession")
    // and UploadChunks stream. The old "Dial-Back" logic below is removed.
    tracing::warn!("sync_data_with_collectors is currently a stub after ControlPlane refactor");
    Ok(())
}
