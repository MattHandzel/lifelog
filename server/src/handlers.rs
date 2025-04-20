// API endpoint handlers for logger operations
use actix_web::{web, HttpResponse, Responder, Error};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use crate::db::{LoggerDb, LoggerType};
use crate::error::ApiError;

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub start_time: Option<f64>,
    pub end_time: Option<f64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub filter: Option<String>,
}

// Handlers for SurrealDB-backed API endpoints

/// Get data from a specific logger
pub async fn get_logger_data<T>(
    name: web::Path<String>,
    query: web::Query<QueryParams>,
    _data: web::Data<T>,
) -> Result<HttpResponse, ApiError> {
    let logger_type = match name.as_str() {
        "screen" => LoggerType::Screen,
        "camera" => LoggerType::Camera,
        "microphone" => LoggerType::Microphone,
        "processes" => LoggerType::Processes,
        "hyprland" => LoggerType::Hyprland,
        _ => return Err(ApiError::NotFound(format!("Logger '{}' not found", name))),
    };
    
    // Mock implementation for now
    let results: Vec<serde_json::Value> = vec![];
    let total_count = 0;
    
    // Create the response manually without using the HeaderValue from method
    Ok(HttpResponse::Ok()
        .insert_header(("Total-Count", format!("{}", total_count)))
        .json(results))
}

/// Insert camera frame data
pub async fn insert_camera_frame(
    frame_data: web::Json<serde_json::Value>,
) -> Result<HttpResponse, Error> {
    let result = LoggerDb::insert_record(LoggerType::Camera, frame_data.into_inner())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;
    
    Ok(HttpResponse::Ok().json(json!({"status": "success", "id": result})))
}

/// Insert screen capture data
pub async fn insert_screen_capture(
    capture_data: web::Json<serde_json::Value>,
) -> Result<HttpResponse, Error> {
    let result = LoggerDb::insert_record(LoggerType::Screen, capture_data.into_inner())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;
    
    Ok(HttpResponse::Ok().json(json!({"status": "success", "id": result})))
}

/// Insert microphone recording data
pub async fn insert_microphone_recording(
    recording_data: web::Json<serde_json::Value>,
) -> Result<HttpResponse, Error> {
    let result = LoggerDb::insert_record(LoggerType::Microphone, recording_data.into_inner())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;
    
    Ok(HttpResponse::Ok().json(json!({"status": "success", "id": result})))
}

/// Insert process data
pub async fn insert_process_data(
    process_data: web::Json<serde_json::Value>,
) -> Result<HttpResponse, Error> {
    let result = LoggerDb::insert_record(LoggerType::Processes, process_data.into_inner())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;
    
    Ok(HttpResponse::Ok().json(json!({"status": "success", "id": result})))
}

/// Get the total count of records for a logger
pub async fn get_logger_count(
    name: web::Path<String>,
    query: web::Query<QueryParams>,
) -> Result<HttpResponse, ApiError> {
    let logger_type = match name.as_str() {
        "screen" => LoggerType::Screen,
        "camera" => LoggerType::Camera,
        "microphone" => LoggerType::Microphone,
        "processes" => LoggerType::Processes,
        "hyprland" => LoggerType::Hyprland,
        _ => return Err(ApiError::NotFound(format!("Logger '{}' not found", name))),
    };
    
    // Mock implementation for count
    let count = LoggerDb::count_records(
        logger_type,
        query.start_time,
        query.end_time,
    )
    .await
    .map_err(|e| ApiError::InternalError(format!("Database error: {}", e)))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({"count": count})))
} 