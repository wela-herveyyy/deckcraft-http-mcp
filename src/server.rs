use crate::backends::Backend;
use crate::config::Config;
use crate::handlers::{AppState, describe, health, tools};
use axum::{
    Router,
    http::HeaderValue,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub fn router(config: Config) -> Router {
    let formats = config.enabled_formats().unwrap_or_default();
    let cors = if config.allowed_origins.iter().any(|o| o == "*") {
        CorsLayer::new().allow_origin(Any)
    } else {
        let origins: Vec<HeaderValue> =
            config.allowed_origins.iter().filter_map(|o| o.parse().ok()).collect();
        CorsLayer::new().allow_origin(origins)
    }
    .allow_methods(Any)
    .allow_headers(Any);

    let state = Arc::new(AppState {
        backend: Backend::from_config(&config),
        registry: Default::default(),
        formats,
        config,
    });
    Router::new()
        .route("/health", get(health::health))
        .route("/mcp", post(tools::rpc))
        .route("/mcp/tools/list", post(tools::list).get(tools::list))
        .route("/mcp/tools/call", post(tools::call))
        .route("/mcp/describe/{tool}", get(describe::describe))
        .route("/mcp/resources", get(describe::resources))
        .route("/antigravity/execute", post(tools::antigravity))
        .layer(cors)
        .with_state(state)
}
