use super::AppState;
use crate::config::LogLevel;
use crate::formats::{Format, ParsedCall, mcp, translator};
use crate::tools::schema;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

type Reply = (StatusCode, Json<Value>);

fn bad(msg: &str) -> Reply {
    (StatusCode::BAD_REQUEST, Json(json!({"error": msg})))
}

pub async fn list(
    State(s): State<Arc<AppState>>,
    Query(q): Query<HashMap<String, String>>,
    body: Option<Json<Value>>,
) -> Reply {
    let body = body.map(|b| b.0).unwrap_or(Value::Null);
    let fmt = match q.get("format") {
        Some(f) => match Format::from_name(f) {
            Some(f) => f,
            None => return bad(&format!("unknown format: {f}")),
        },
        None if body.get("jsonrpc").is_some() => Format::JsonRpc,
        None => Format::Mcp,
    };
    if !s.formats.contains(&fmt) {
        return bad(&format!("format disabled: {fmt:?}"));
    }
    let result = schema::list_for(fmt, s.registry.all());
    let out = if body.get("jsonrpc").is_some() {
        mcp::jsonrpc_ok(body.get("id").cloned(), result)
    } else {
        result
    };
    (StatusCode::OK, Json(out))
}

async fn execute(s: &AppState, call: &ParsedCall) -> Value {
    s.config.log(LogLevel::Debug, &format!("call {} via {:?}", call.name, call.format));
    let out = if s.registry.get(&call.name).is_none() {
        crate::backends::ToolOutput::err(format!("unknown tool: {}", call.name))
    } else {
        s.backend.call(&call.name, call.arguments.clone()).await
    };
    translator::render(call, &out)
}

async fn call_with(s: Arc<AppState>, body: Value, hint: Option<Format>) -> Reply {
    let call = match translator::parse_call(&body, hint) {
        Ok(c) => c,
        Err(e) => return bad(&e),
    };
    if !s.formats.contains(&call.format) {
        return bad(&format!("format disabled: {:?}", call.format));
    }
    (StatusCode::OK, Json(execute(&s, &call).await))
}

pub async fn call(State(s): State<Arc<AppState>>, Json(body): Json<Value>) -> Reply {
    call_with(s, body, None).await
}

pub async fn antigravity(State(s): State<Arc<AppState>>, Json(body): Json<Value>) -> Reply {
    call_with(s, body, Some(Format::Antigravity)).await
}

/// Standard MCP JSON-RPC endpoint (`initialize`, `tools/list`, `tools/call`).
pub async fn rpc(State(s): State<Arc<AppState>>, Json(body): Json<Value>) -> Reply {
    let id = body.get("id").cloned();
    let ok = |v: Value| (StatusCode::OK, Json(mcp::jsonrpc_ok(id.clone(), v)));
    match body.get("method").and_then(Value::as_str) {
        Some("initialize") => ok(json!({
            "protocolVersion":"2025-03-26",
            "capabilities":{"tools":{},"resources":{}},
            "serverInfo":{"name":"deckcraft-http-mcp","version":env!("CARGO_PKG_VERSION")}})),
        Some("tools/list") => ok(schema::list_for(Format::Mcp, s.registry.all())),
        Some("tools/call") => call_with(s, body, Some(Format::JsonRpc)).await,
        Some("workspace/executeCommand") => call_with(s, body, Some(Format::Cursor)).await,
        Some("resources/list") => ok(json!({"resources":[{"uri":"deckcraft://deck","name":"Current deck"}]})),
        Some("ping") => ok(json!({})),
        Some(m) => (StatusCode::OK, Json(mcp::jsonrpc_err(id, -32601, &format!("method not found: {m}")))),
        None => (StatusCode::OK, Json(mcp::jsonrpc_err(id, -32600, "invalid request"))),
    }
}
