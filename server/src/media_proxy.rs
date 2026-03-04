use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use utils::cas::FsCas;

#[derive(Clone)]
pub struct ProxyState {
    pub cas: Arc<FsCas>,
}

pub async fn start_media_proxy(cas: Arc<FsCas>, port: u16) {
    let state = ProxyState { cas };

    let app = Router::new()
        .route("/media/:hash", get(get_media))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Media proxy listening on {}", addr);

    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
}

async fn get_media(Path(hash): Path<String>, State(state): State<ProxyState>) -> impl IntoResponse {
    match state.cas.get(&hash) {
        Ok(bytes) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "image/webp"),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}
