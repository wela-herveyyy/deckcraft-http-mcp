use super::{Format, ParsedCall, parse_args};
use crate::backends::ToolOutput;
use serde_json::{Value, json};

/// Gemini `functionCall`: {"functionCall": {"name", "args"}}
pub fn parse(v: &Value) -> Result<ParsedCall, String> {
    let f = v.get("functionCall").ok_or("missing 'functionCall'")?;
    Ok(ParsedCall {
        format: Format::Gemini,
        id: None,
        name: f.get("name").and_then(Value::as_str).ok_or("missing function name")?.to_string(),
        arguments: parse_args(f.get("args"))?,
    })
}

pub fn render(c: &ParsedCall, out: &ToolOutput) -> Value {
    json!({"functionResponse":{"name":c.name,"response":{"content":out.text,"isError":out.is_error}}})
}
