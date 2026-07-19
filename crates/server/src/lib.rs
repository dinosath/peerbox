pub mod routes;
pub mod state;
pub mod types;

#[cfg(test)]
mod tests;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use state::AppState;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", axum::routing::get(routes::health))
        .route(
            "/objects",
            axum::routing::get(routes::list_objects).post(routes::upload_file),
        )
        .route("/objects/{id}", axum::routing::get(routes::get_object))
        .route("/manifest/{id}", axum::routing::get(routes::get_manifest))
        .route(
            "/activitypub/inbox",
            axum::routing::post(routes::activitypub_inbox),
        )
        .route(
            "/.well-known/webfinger",
            axum::routing::get(routes::webfinger),
        )
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

pub async fn run_server(state: Arc<AppState>, addr: SocketAddr) -> anyhow::Result<()> {
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("server listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
