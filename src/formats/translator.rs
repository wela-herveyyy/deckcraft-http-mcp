use super::{Format, ParsedCall, antigravity, codex, cursor, gemini, mcp, openai};
use crate::backends::ToolOutput;
use serde_json::Value;

/// Detect the request format from the body shape.
pub fn detect(v: &Value) -> Format {
    if v.get("jsonrpc").is_some() {
        return if v.get("method").and_then(Value::as_str) == Some("workspace/executeCommand") {
            Format::Cursor
        } else {
            Format::JsonRpc
        };
    }
    if v.get("function").is_some() {
        Format::OpenAi
    } else if v.get("function_call").is_some() {
        Format::Codex
    } else if v.get("functionCall").is_some() {
        Format::Gemini
    } else if v.get("command").is_some() {
        Format::Antigravity
    } else {
        Format::Mcp
    }
}

/// Parse a tool call; `hint` overrides detection (e.g. the antigravity endpoint).
pub fn parse_call(v: &Value, hint: Option<Format>) -> Result<ParsedCall, String> {
    match hint.unwrap_or_else(|| detect(v)) {
        Format::Mcp => mcp::parse(v),
        Format::OpenAi => openai::parse(v),
        Format::Codex => codex::parse(v),
        Format::Gemini => gemini::parse(v),
        Format::Cursor => cursor::parse(v),
        Format::Antigravity => antigravity::parse(v),
        Format::JsonRpc => mcp::parse_jsonrpc(v),
    }
}

pub fn render(c: &ParsedCall, out: &ToolOutput) -> Value {
    match c.format {
        Format::Mcp => mcp::render(out),
        Format::OpenAi => openai::render(c, out),
        Format::Codex => codex::render(c, out),
        Format::Gemini => gemini::render(c, out),
        Format::Cursor => cursor::render(c, out),
        Format::Antigravity => antigravity::render(c, out),
        Format::JsonRpc => mcp::render_jsonrpc(c, out),
    }
}
