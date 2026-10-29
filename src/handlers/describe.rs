use super::AppState;
use crate::formats::Format;
use crate::tools::schema;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

pub async fn describe(
    State(s): State<Arc<AppState>>,
    Path(tool): Path<String>,
    Query(q): Query<HashMap<String, String>>,
) -> (StatusCode, Json<Value>) {
    let fmt = q.get("format").and_then(|f| Format::from_name(f)).unwrap_or(Format::Mcp);
    match s.registry.get(&tool) {
        Some(t) => (StatusCode::OK, Json(schema::describe_for(fmt, t))),
        None => (StatusCode::NOT_FOUND, Json(json!({"error": format!("unknown tool: {tool}")}))),
    }
}

pub async fn resources() -> Json<Value> {
    Json(json!({"resources":[{
        "uri":"deckcraft://deck","name":"Current deck","mimeType":"application/json",
        "description":"The deck currently loaded in the DeckCraft session"}]}))
}
