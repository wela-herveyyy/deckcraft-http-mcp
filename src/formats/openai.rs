use super::{Format, ParsedCall, parse_args};
use crate::backends::ToolOutput;
use serde_json::{Value, json};

pub fn parse(v: &Value) -> Result<ParsedCall, String> {
    let f = v.get("function").ok_or("missing 'function'")?;
    Ok(ParsedCall {
        format: Format::OpenAi,
        id: v.get("id").cloned(),
        name: f.get("name").and_then(Value::as_str).ok_or("missing function name")?.to_string(),
        arguments: parse_args(f.get("arguments"))?,
    })
}

pub fn render(c: &ParsedCall, out: &ToolOutput) -> Value {
    json!({"role":"tool","tool_call_id":c.id.clone().unwrap_or(Value::Null),
           "name":c.name,"content":out.text,"is_error":out.is_error})
}
