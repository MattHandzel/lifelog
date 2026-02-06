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
        // TODO: Parallelize this
        println!("Syncing data with collector: {:?}", collector);
        // TODO: This code can fail here (notice the unwraps, I should handle it.
        let mut stream = collector
            .grpc_client
            .get_data(GetDataRequest { keys: vec![] })
            .await
            .unwrap()
            .into_inner();
        println!("Defined the stream here...");
        let mut data = vec![];

        while let Some(chunk) = stream.next().await {
            data.push(chunk.unwrap().payload);
        }
        println!("Done receiving data");
        let mac = collector.mac.clone();

        // TODO: REFACTOR THIS FUNCTION
        for chunk in data {
            // record id = random UUID
            let chunk = chunk.unwrap();

            // TODO: this can be automated with a macro
            match chunk {
                lifelog_proto::lifelog_data::Payload::Screenframe(c) => {
                    let data_origin = DataOrigin::new(
                        DataOriginType::DeviceId(mac.clone()),
                        DataModality::Screen,
                    );
                    let _record = add_data_to_db::<ScreenFrame, ScreenFrameSurreal>(
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

                    let _record = add_data_to_db::<BrowserFrame, BrowserFrameSurreal>(
                        db,
                        c.into(),
                        &data_origin,
                    )
                    .await;
                }
                _ => unimplemented!(),
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
    // TODO: dude what are you doing? use surrealdb to do this query, don't manuall do the set
    // difference.
    // Get the record uuids from source
    let uuids_from_source = get_all_uuids_from_origin(db, &source)
        .await
        .expect(format!("Unable to get uuids from source: {}", source).as_str());

    // Get the record uuids from destination
    let uuids_from_destination = get_all_uuids_from_origin(db, &destination)
        .await
        .expect(format!("Unable to get uuids from destination: {}", destination).as_str());

    // Get the record uuids from source that are not in destination
    let uuids_in_source_not_in_destination: Vec<LifelogFrameKey> = uuids_from_source
        .iter()
        .filter(|uuid| !uuids_from_destination.contains(uuid))
        .cloned()
        .map(|uuid| {
            let key = LifelogFrameKey::new(uuid, source.clone());
            key
        })
        .collect();

    uuids_in_source_not_in_destination
}
