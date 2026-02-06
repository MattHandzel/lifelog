use data_modalities::*;
use lifelog_proto::GetDataRequest;
use lifelog_types::*;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tokio_stream::StreamExt;

use crate::db::add_data_to_db;
use crate::query::get_all_uuids_from_origin;
use crate::surreal_types::*;

pub(crate) async fn sync_data_with_collectors(
    _state: SystemState,
    db: &Surreal<Client>,
    _query: String,
    collectors: &mut Vec<RegisteredCollector>,
) -> Result<(), LifelogError> {
    for collector in collectors.iter_mut() {
        let mut stream = collector
            .grpc_client
            .get_data(GetDataRequest { keys: vec![] })
            .await
            .map_err(|e| LifelogError::GrpcStatus(e))?
            .into_inner();

        let mut data = vec![];
        while let Some(chunk) = stream.next().await {
            if let Ok(chunk) = chunk {
                data.push(chunk.payload);
            }
        }
        let mac = collector.mac.clone();

        for chunk in data {
            let Some(chunk) = chunk else { continue };

            match chunk {
                lifelog_proto::lifelog_data::Payload::Screenframe(c) => {
                    let data_origin = DataOrigin::new(
                        DataOriginType::DeviceId(mac.clone()),
                        DataModality::Screen,
                    );
                    let _ = add_data_to_db::<ScreenFrame, ScreenFrameSurreal>(
                        db,
                        c.into(),
                        &data_origin,
                    )
                    .await;
                }
                lifelog_proto::lifelog_data::Payload::Browserframe(c) => {
                    let data_origin = DataOrigin::new(
                        DataOriginType::DeviceId(mac.clone()),
                        DataModality::Browser,
                    );
                    let _ = add_data_to_db::<BrowserFrame, BrowserFrameSurreal>(
                        db,
                        c.into(),
                        &data_origin,
                    )
                    .await;
                }
                _ => {}
            };
        }
    }
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
