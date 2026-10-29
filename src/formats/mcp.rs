use super::{Format, ParsedCall, parse_args};
use crate::backends::ToolOutput;
use serde_json::{Value, json};

pub fn parse(v: &Value) -> Result<ParsedCall, String> {
    Ok(ParsedCall {
        format: Format::Mcp,
        id: None,
        name: v.get("name").and_then(Value::as_str).ok_or("missing 'name'")?.to_string(),
        arguments: parse_args(v.get("arguments"))?,
    })
}

pub fn result_value(out: &ToolOutput) -> Value {
    json!({"content":[{"type":"text","text":out.text}],"isError":out.is_error})
}

pub fn render(out: &ToolOutput) -> Value {
    result_value(out)
}

pub fn jsonrpc_ok(id: Option<Value>, result: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id.unwrap_or(Value::Null),"result":result})
}

pub fn jsonrpc_err(id: Option<Value>, code: i64, msg: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id.unwrap_or(Value::Null),"error":{"code":code,"message":msg}})
}

/// Generic JSON-RPC `tools/call` (also used for the `/mcp` endpoint).
pub fn parse_jsonrpc(v: &Value) -> Result<ParsedCall, String> {
    let p = v.get("params").ok_or("missing 'params'")?;
    let mut c = parse(p)?;
    c.format = Format::JsonRpc;
    c.id = v.get("id").cloned();
    Ok(c)
}

pub fn render_jsonrpc(c: &ParsedCall, out: &ToolOutput) -> Value {
    jsonrpc_ok(c.id.clone(), result_value(out))
}
