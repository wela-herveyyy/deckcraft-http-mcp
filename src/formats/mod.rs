pub mod antigravity;
pub mod codex;
pub mod cursor;
pub mod gemini;
pub mod mcp;
pub mod openai;
pub mod translator;

use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Mcp,
    OpenAi,
    Gemini,
    Cursor,
    Codex,
    Antigravity,
    JsonRpc,
}

const NAMES: &[(&str, Format)] = &[
    ("mcp", Format::Mcp),
    ("claude", Format::Mcp),
    ("openai", Format::OpenAi),
    ("chatgpt", Format::OpenAi),
    ("gemini", Format::Gemini),
    ("cursor", Format::Cursor),
    ("codex", Format::Codex),
    ("antigravity", Format::Antigravity),
    ("jsonrpc", Format::JsonRpc),
];

impl Format {
    pub fn from_name(n: &str) -> Option<Format> {
        let n = n.to_ascii_lowercase();
        NAMES.iter().find(|(k, _)| *k == n).map(|(_, f)| *f)
    }
    pub fn all_names() -> Vec<String> {
        ["mcp", "openai", "gemini", "cursor", "codex", "antigravity", "jsonrpc"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
}

/// A tool call parsed from any supported request format.
#[derive(Clone, Debug)]
pub struct ParsedCall {
    pub format: Format,
    /// Request id (JSON-RPC id or OpenAI tool_call id), echoed back.
    pub id: Option<Value>,
    pub name: String,
    pub arguments: Value,
}

/// Parse `arguments` that may be an object or a JSON-encoded string.
pub fn parse_args(v: Option<&Value>) -> Result<Value, String> {
    match v {
        None | Some(Value::Null) => Ok(Value::Object(Default::default())),
        Some(Value::String(s)) if s.trim().is_empty() => Ok(Value::Object(Default::default())),
        Some(Value::String(s)) => serde_json::from_str(s).map_err(|e| format!("invalid arguments JSON: {e}")),
        Some(other) => Ok(other.clone()),
    }
}
