use super::registry::ToolDef;
use serde_json::json;

pub fn all() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "run_command",
            description: "Run any command by id with JSON parameters",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "Command id, e.g. slide.new"},
                    "params": {"type": "object", "description": "Command parameters"}
                },
                "required": ["command"]
            }),
        },
        ToolDef {
            name: "list_commands",
            description: "List available command ids",
            input_schema: json!({"type": "object", "properties": {}}),
        },
        ToolDef {
            name: "open_deck",
            description: "Open a deck from a file (subject to the read root)",
            input_schema: json!({
                "type": "object",
                "properties": {"path": {"type": "string"}},
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "save_deck",
            description: "Save the current deck to a file (subject to the write root)",
            input_schema: json!({
                "type": "object",
                "properties": {"path": {"type": "string"}},
                "required": ["path"]
            }),
        },
    ]
}
