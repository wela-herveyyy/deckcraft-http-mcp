use super::{Format, ParsedCall, parse_args};
use crate::backends::ToolOutput;
use serde_json::{Value, json};

/// LSP `workspace/executeCommand`: params {command, arguments: [args]}
pub fn parse(v: &Value) -> Result<ParsedCall, String> {
    let p = v.get("params").ok_or("missing 'params'")?;
    let first = p.get("arguments").and_then(Value::as_array).and_then(|a| a.first());
    Ok(ParsedCall {
        format: Format::Cursor,
        id: v.get("id").cloned(),
        name: p.get("command").and_then(Value::as_str).ok_or("missing 'command'")?.to_string(),
        arguments: parse_args(first)?,
    })
}

pub fn render(c: &ParsedCall, out: &ToolOutput) -> Value {
    if out.is_error {
        super::mcp::jsonrpc_err(c.id.clone(), -32000, &out.text)
    } else {
        super::mcp::jsonrpc_ok(c.id.clone(), json!({"text":out.text}))
    }
}
