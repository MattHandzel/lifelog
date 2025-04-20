use dotenv::dotenv;
use std::sync::Arc;
use tokio::signal;
use tonic::transport::Server;
use lifelog_server::Database;
use lifelog_server::grpc::lifelog::lifelog_service_server::LifelogServiceServer;
use lifelog_server::grpc::LifelogGrpcService;
use tonic_web::GrpcWebLayer;
use tower_http::cors::CorsLayer;
use hyper::{Body, Request, Response, StatusCode};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use std::convert::Infallible;

async fn health_check(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .header("access-control-allow-origin", "*")
        .body(Body::from(r#"{"status":"ok"}"#))
        .unwrap();
    Ok(response)
}

async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();
    if path.starts_with("/healthz") {
        return health_check(req).await;
    }
    
    // Return a 404 for any other HTTP/1.1 path
    let response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("content-type", "application/json")
        .header("access-control-allow-origin", "*")
        .body(Body::from(r#"{"error":"not found"}"#))
        .unwrap();
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Set up tracing
    tracing_subscriber::fmt::init();
    
    // Get the server address from the environment or use a default
    let addr = std::env::var("GRPC_SERVER_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:50051".to_string())
        .parse()?;
    
    // Create the database connection
    let database = Arc::new(Database::new().await?);
    
    // Create the gRPC service
    let service = LifelogGrpcService::new(database);
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);
    
    // Start the gRPC server
    tracing::info!("Starting gRPC server with gRPC-Web support on {}", addr);
    
    Server::builder()
        .accept_http1(true) // This is required for gRPC-Web
        .layer(TraceLayer::new_for_http()) // Add tracing
        .layer(cors) // Add CORS support
        .layer(GrpcWebLayer::new()) // Add gRPC-Web support
        .add_service(LifelogServiceServer::new(service))
        .http1_router(router) // Handle regular HTTP/1.1 requests
        .serve_with_shutdown(addr, async {
            let ctrl_c = async {
                signal::ctrl_c()
                    .await
                    .expect("Failed to install Ctrl+C handler");
            };
            
            #[cfg(unix)]
            let terminate = async {
                signal::unix::signal(signal::unix::SignalKind::terminate())
                    .expect("Failed to install SIGTERM handler")
                    .recv()
                    .await;
            };
            
            #[cfg(not(unix))]
            let terminate = std::future::pending::<()>();
            
            tokio::select! {
                _ = ctrl_c => {},
                _ = terminate => {},
            }
            
            tracing::info!("Shutting down gRPC server");
        })
        .await?;
    
    Ok(())
} 