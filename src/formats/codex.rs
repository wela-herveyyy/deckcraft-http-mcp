use super::{Format, ParsedCall, parse_args};
use crate::backends::ToolOutput;
use serde_json::{Value, json};

/// Legacy `function_call` format: {"function_call": {"name", "arguments"}}
pub fn parse(v: &Value) -> Result<ParsedCall, String> {
    let f = v.get("function_call").ok_or("missing 'function_call'")?;
    Ok(ParsedCall {
        format: Format::Codex,
        id: None,
        name: f.get("name").and_then(Value::as_str).ok_or("missing function name")?.to_string(),
        arguments: parse_args(f.get("arguments"))?,
    })
}

pub fn render(c: &ParsedCall, out: &ToolOutput) -> Value {
    json!({"role":"function","name":c.name,"content":out.text,"is_error":out.is_error})
}
