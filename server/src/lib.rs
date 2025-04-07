// Library module exports and type re-exports
pub mod auth;
pub mod auth_handlers;
pub mod embed;
pub mod install;
pub mod cors;
pub mod db;
pub mod handlers;
pub mod db_config;

// Re-export key types for convenience
pub use auth::{Claims, JwtAuth};
pub use auth_handlers::{login, get_profile};
pub use cors::*;
pub use db::*;
pub use handlers::*;
pub use db_config::*;
