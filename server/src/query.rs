use crate::server::LifelogData;
use anyhow;
use lifelog_core::uuid::Uuid;
use lifelog_types::*;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::surreal_types::*;

pub(crate) async fn get_all_uuids_from_origin(
    db: &Surreal<Client>,
    data_origin: &DataOrigin,
) -> Result<Vec<Uuid>, surrealdb::Error> {
    let table = data_origin.get_table_name();
    let sql = format!("SELECT VALUE record::id(id) as uuid FROM `{table}`"); //FIX: Sql injection
    let uuids: Vec<String> = db
        .query(sql)
        .await
        .expect("Couldn't do the query")
        .take(0)
        .expect("We should only ever have one query");
    let uuids = uuids
        .into_iter()
        .map(|s| {
            let uuid = s.parse::<Uuid>().expect("Unable to go from string to uuid");
            uuid
        })
        .collect::<Vec<Uuid>>();
    Ok(uuids)
}

pub(crate) async fn get_data_by_key(
    db: &Surreal<Client>,
    key: &LifelogFrameKey,
) -> Result<LifelogData, anyhow::Error> {
    match key.origin.modality {
        DataModality::Screen => {
            let row: Option<ScreenFrameSurreal> = db
                .select((key.origin.get_table_name(), key.uuid.to_string()))
                .await?;
            let mut screen_frame: data_modalities::ScreenFrame = row
                .expect(
                    format!(
                        "Unabled to find record <{}>:<{}>",
                        key.origin.get_table_name(),
                        key.uuid
                    )
                    .as_str(),
                )
                .into();
            screen_frame.uuid = key.uuid; //NOTE: This is important. This needs to be fixed with a code refactor
            Ok(screen_frame.into())
        }
        DataModality::Ocr => {
            let row: Option<OcrFrameSurreal> = db
                .select((key.origin.get_table_name(), key.uuid.to_string()))
                .await?;
            let mut ocr_frame: data_modalities::OcrFrame = row
                .expect(
                    format!(
                        "Unabled to find record <{}>:<{}>",
                        key.origin.get_table_name(),
                        key.uuid
                    )
                    .as_str(),
                )
                .into();
            ocr_frame.uuid = key.uuid; //NOTE: This is important. This needs to be fixed with a code refactor
            Ok(ocr_frame.into())
        }
        DataModality::Browser => {
            let row: Option<BrowserFrameSurreal> = db
                .select((key.origin.get_table_name(), key.uuid.to_string()))
                .await?;
            let mut browser_frame: data_modalities::BrowserFrame = row
                .expect(
                    format!(
                        "Unabled to find record <{}>:<{}>",
                        key.origin.get_table_name(),
                        key.uuid
                    )
                    .as_str(),
                )
                .into();
            browser_frame.uuid = key.uuid; //NOTE: This is important. This needs to be fixed with a code refactor
            Ok(browser_frame.into())
        }
    }
}
