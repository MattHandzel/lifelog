use dashmap::DashSet;
use lifelog_core::DataType;
use lifelog_types::*;
use once_cell::sync::Lazy;
use serde::de::DeserializeOwned;
use serde::Serialize;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Error;
use surrealdb::Surreal;

pub(crate) static CREATED_TABLES: Lazy<DashSet<String>> = Lazy::new(DashSet::new);

/// Ensure a table exists for the given data origin.
/// Delegates to the centralized schema registry.
pub(crate) async fn ensure_table(
    db: &Surreal<Client>,
    data_origin: &DataOrigin,
) -> surrealdb::Result<()> {
    crate::schema::ensure_table_schema(db, data_origin).await
}

pub(crate) async fn add_data_to_db<LifelogType, SurrealType>(
    db: &Surreal<Client>,
    data: LifelogType,
    data_origin: &DataOrigin,
) -> surrealdb::Result<SurrealType>
where
    LifelogType: Into<SurrealType> + DataType,
    SurrealType: Serialize + DeserializeOwned + 'static,
{
    let uuid = data.uuid();
    let table = format!("{}", data_origin.get_table_name());
    ensure_table(db, data_origin).await.unwrap();
    let data: SurrealType = data.into();
    let record: Result<Option<SurrealType>, Error> = db
        .create((table.clone(), uuid.to_string()))
        .content(data)
        .await;
    println!("[SURREAL]: Created <{}:{}>", table, uuid);
    match record {
        Err(e) => {
            eprintln!("{}", e);
            Err(e)
        }
        Ok(record) => {
            let record = record.expect(format!("Unable to create row in table {}", table).as_str());
            Ok(record)
        }
    }
}

pub async fn get_tables(db: &Surreal<Client>) -> Result<Vec<String>, LifelogError> {
    #[derive(serde::Deserialize)]
    struct Info {
        tables: std::collections::HashMap<String, serde_json::Value>,
    }

    let mut resp = db
        .query("INFO FOR DB")
        .await
        .map_err(|e| LifelogError::Database(format!("{}", e)))?;

    let info: Option<Info> = resp
        .take(0)
        .map_err(|e| LifelogError::Database(format!("{}", e)))?;
    let info = info.ok_or_else(|| LifelogError::Database("INFO FOR DB failed!!!".to_string()))?;
    Ok(info.tables.keys().cloned().collect())
}

pub(crate) async fn get_origins_from_db(
    db: &Surreal<Client>,
) -> Result<Vec<DataOrigin>, LifelogError> {
    let tables = get_tables(db).await?;
    let origins = tables
        .iter()
        .map(|table| {
            let origin = DataOrigin::tryfrom_string(table.clone());
            origin
        })
        .filter(Result::is_ok)
        .map(|origin| origin.expect("this should never happen"))
        .collect::<Vec<DataOrigin>>();

    Ok(origins)
}
