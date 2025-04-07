// API endpoint handlers for logger operations
use actix_web::{web, HttpResponse, Responder, Error};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

use crate::db::{LoggerDb, LoggerType};

#[derive(Deserialize)]
pub struct LoggerQuery {
    start_time: Option<f64>,
    end_time: Option<f64>,
    limit: Option<u32>,
    filter: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

// Handlers for SurrealDB-backed API endpoints

/// Get data from a specific logger
pub async fn get_logger_data(
    name: web::Path<String>,
    query: web::Query<LoggerQuery>,
) -> Result<HttpResponse, Error> {
    let logger_type = LoggerType::from_str(&name)
        .ok_or_else(|| actix_web::error::ErrorNotFound(format!("Logger '{}' not found", name)))?;
    
    let query = query.into_inner();
    
    let records = LoggerDb::get_records(
        logger_type,
        query.start_time,
        query.end_time,
        query.limit,
        query.filter,
        query.page,
        query.page_size,
    )
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;
    
    if query.page.is_some() && query.page_size.is_some() {
        let total_count = LoggerDb::count_records(logger_type)
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;
        
        let mut response = HttpResponse::Ok().json(records);
        response.headers_mut().insert(
            actix_web::http::header::HeaderName::from_static("x-total-count"),
            actix_web::http::header::HeaderValue::from(total_count.to_string()),
        );
        
        Ok(response)
    } else {
        Ok(HttpResponse::Ok().json(records))
    }
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
) -> Result<HttpResponse, Error> {
    let logger_type = LoggerType::from_str(&name)
        .ok_or_else(|| actix_web::error::ErrorNotFound(format!("Logger '{}' not found", name)))?;
    
    let count = LoggerDb::count_records(logger_type)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;
    
    Ok(HttpResponse::Ok().json(json!({"count": count})))
} 