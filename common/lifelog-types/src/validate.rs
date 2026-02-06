use crate::error::LifelogError;

/// Trait for validating proto-generated config types.
///
/// Types that implement `Validate` can check their invariants
/// and return a `LifelogError::Validation` on failure.
pub trait Validate {
    /// Check all invariants. Returns `Ok(())` if valid.
    fn validate(&self) -> Result<(), LifelogError>;
}

impl Validate for lifelog_proto::ServerConfig {
    fn validate(&self) -> Result<(), LifelogError> {
        if self.host.is_empty() {
            return Err(LifelogError::Validation {
                field: "host",
                reason: "must not be empty".to_string(),
            });
        }
        if self.port == 0 || self.port > 65535 {
            return Err(LifelogError::Validation {
                field: "port",
                reason: format!("must be between 1 and 65535, got {}", self.port),
            });
        }
        if self.database_endpoint.is_empty() {
            return Err(LifelogError::Validation {
                field: "database_endpoint",
                reason: "must not be empty".to_string(),
            });
        }
        if self.database_name.is_empty() {
            return Err(LifelogError::Validation {
                field: "database_name",
                reason: "must not be empty".to_string(),
            });
        }
        if self.server_name.is_empty() {
            return Err(LifelogError::Validation {
                field: "server_name",
                reason: "must not be empty".to_string(),
            });
        }
        Ok(())
    }
}

impl Validate for lifelog_proto::CollectorConfig {
    fn validate(&self) -> Result<(), LifelogError> {
        if self.id.is_empty() {
            return Err(LifelogError::Validation {
                field: "id",
                reason: "collector ID must not be empty".to_string(),
            });
        }
        if self.host.is_empty() {
            return Err(LifelogError::Validation {
                field: "host",
                reason: "must not be empty".to_string(),
            });
        }
        if self.port == 0 || self.port > 65535 {
            return Err(LifelogError::Validation {
                field: "port",
                reason: format!("must be between 1 and 65535, got {}", self.port),
            });
        }
        Ok(())
    }
}
