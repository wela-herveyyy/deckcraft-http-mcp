use super::{Format, ParsedCall, parse_args};
use crate::backends::ToolOutput;
use serde_json::{Value, json};

/// {"command": "<tool>", "args": {...}}
pub fn parse(v: &Value) -> Result<ParsedCall, String> {
    Ok(ParsedCall {
        format: Format::Antigravity,
        id: v.get("id").cloned(),
        name: v.get("command").and_then(Value::as_str).ok_or("missing 'command'")?.to_string(),
        arguments: parse_args(v.get("args"))?,
    })
}

pub fn render(_c: &ParsedCall, out: &ToolOutput) -> Value {
    if out.is_error {
        json!({"status":"error","error":out.text})
    } else {
        json!({"status":"ok","result":out.text})
    }
}
