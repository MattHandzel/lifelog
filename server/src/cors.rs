// CORS configuration and middleware setup
use actix_cors::Cors;
use actix_web::http::header;
use std::env;

/// Configure CORS for the API
/// 
/// This function creates a CORS configuration based on environment variables.
/// In development mode, it allows all origins, headers and methods for easier debugging.
/// In production, it follows strict CORS rules based on the configured allowed origins.
pub fn cors_config() -> Cors {
    let allowed_origins = env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://localhost:8080,http://localhost:1420".to_string());
    
    let development_mode = env::var("DEVELOPMENT_MODE")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);
    
    // In development mode, allow all origins for easier debugging
    if development_mode {
        return Cors::permissive();
    }
    
    // In production, configure strict CORS
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
        .max_age(3600);
    
    // Add the allowed origins
    for origin in allowed_origins.split(',') {
        cors = cors.allowed_origin(origin.trim());
    }
    
    cors
}

/// Configure CORS for static file serving
/// 
/// This function creates a less restrictive CORS configuration for static files.
pub fn configure_static_cors() -> Cors {
    Cors::default()
        .allowed_methods(vec!["GET"])
        .allowed_headers(vec![
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::IF_NONE_MATCH,
            header::IF_MODIFIED_SINCE,
        ])
        .max_age(86400) // 1 day
} 