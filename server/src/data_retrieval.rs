use crate::postgres::PostgresPool;
use lifelog_core::*;
use utils::cas::FsCas;

pub(crate) async fn get_data_by_key(
    pool: &PostgresPool,
    cas: &FsCas,
    key: &LifelogFrameKey,
) -> Result<lifelog_types::LifelogData, LifelogError> {
    let uuid = uuid::Uuid::from_bytes(key.uuid.into_bytes());
    crate::frames::get_by_id(pool, cas, uuid).await
}
