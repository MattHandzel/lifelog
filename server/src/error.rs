// API error handling
use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use serde_json::json;

#[derive(Debug, Display)]
pub enum ApiError {
    #[display(fmt = "Internal Server Error: {}", _0)]
    InternalError(String),
    #[display(fmt = "Not Found: {}", _0)]
    NotFound(String),
    #[display(fmt = "Unauthorized: {}", _0)]
    Unauthorized(String),
    #[display(fmt = "Bad Request: {}", _0)]
    BadRequest(String),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::InternalError(message) => {
                HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": message
                }))
            }
            ApiError::NotFound(message) => {
                HttpResponse::NotFound().json(json!({
                    "status": "error",
                    "message": message
                }))
            }
            ApiError::Unauthorized(message) => {
                HttpResponse::Unauthorized().json(json!({
                    "status": "error",
                    "message": message
                }))
            }
            ApiError::BadRequest(message) => {
                HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": message
                }))
            }
        }
    }
} 