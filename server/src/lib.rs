// Library module exports and type re-exports
pub mod policy;
pub mod server;

// Internal modules (accessible to server.rs and each other within the crate)
pub(crate) mod data_retrieval;
pub mod db;
pub(crate) mod grpc_service;
pub(crate) mod ingest;
pub(crate) mod query;
pub(crate) mod schema;
pub(crate) mod sync;
pub(crate) mod transform;

/// Test-only utilities exposed for integration tests.
#[cfg(test)]
pub mod test_support {
    pub fn reset_table_cache() {
        crate::db::reset_table_cache();
    }
}
