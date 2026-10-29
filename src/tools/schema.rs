use super::registry::ToolDef;
use crate::formats::Format;
use serde_json::{Value, json};

pub fn to_mcp(t: &ToolDef) -> Value {
    json!({"name": t.name, "description": t.description, "inputSchema": t.input_schema})
}

pub fn to_openai(t: &ToolDef) -> Value {
    json!({"type":"function","function":{"name":t.name,"description":t.description,"parameters":t.input_schema}})
}

pub fn to_function(t: &ToolDef) -> Value {
    json!({"name":t.name,"description":t.description,"parameters":t.input_schema})
}

/// Render the tool list in the shape expected by `format`.
pub fn list_for(format: Format, tools: &[ToolDef]) -> Value {
    match format {
        Format::OpenAi => json!({"tools": tools.iter().map(to_openai).collect::<Vec<_>>()}),
        Format::Codex => json!({"functions": tools.iter().map(to_function).collect::<Vec<_>>()}),
        Format::Gemini => json!({"tools": [{"functionDeclarations": tools.iter().map(to_function).collect::<Vec<_>>()}]}),
        Format::Antigravity => json!({"commands": tools.iter().map(|t| json!({"command":t.name,"description":t.description,"args":t.input_schema})).collect::<Vec<_>>()}),
        _ => json!({"tools": tools.iter().map(to_mcp).collect::<Vec<_>>()}),
    }
}

pub fn describe_for(format: Format, t: &ToolDef) -> Value {
    match format {
        Format::OpenAi => to_openai(t),
        Format::Codex | Format::Gemini => to_function(t),
        _ => to_mcp(t),
    }
}
