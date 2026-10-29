use super::AppState;
use axum::{Json, extract::State};
use serde_json::{Value, json};
use std::sync::Arc;

pub async fn health(State(s): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({"status":"ok","mode":s.backend.mode_name(),"version":env!("CARGO_PKG_VERSION")}))
}
