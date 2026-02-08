use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransformError {
    #[error("Unknown error occurred")]
    Unknown,
}

#[derive(Debug, Error)]
pub enum LifelogError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("database: table '{table}' already exists")]
    TableAlreadyExists { table: String },

    #[error("configuration parse error: {0}")]
    Config(#[from] toml::de::Error),

    #[error("database: {0}")]
    Database(String),

    #[error("JSON (de)serialisation error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("protobuf decode error: {0}")]
    ProstDecode(#[from] prost::DecodeError),

    #[error("gRPC transport error: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),

    #[error("gRPC status: {0}")]
    GrpcStatus(#[from] tonic::Status),

    #[error("background task join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("stream closed unexpectedly")]
    StreamClosed,

    #[error("unknown collector '{0}'")]
    UnknownCollector(String),

    #[error("transform '{name}' failed: {source}")]
    Transform {
        name: &'static str,
        #[source]
        source: anyhow::Error,
    },

    #[error("validation failed for field '{field}': {reason}")]
    Validation { field: String, reason: String },

    #[error("tried to parse invalid data modality: '{0}'")]
    InvalidDataModality(String),

    #[error("source '{0}' setup failed: {1}")]
    SourceSetup(String, String),

    #[error("not connected to server")]
    NotConnected,

    #[error("registration failed: {0}")]
    RegistrationFailed(String),

    #[error("task is already running")]
    AlreadyRunning,

    #[error("task is not running")]
    NotRunning,

    #[error("SQLite error: {0}")]
    Sqlite(String),

    #[error("buffer corrupt: {0}")]
    BufferCorrupt(String),

    #[error("logger error: {0}")]
    Logger(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
