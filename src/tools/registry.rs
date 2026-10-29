use super::defs;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
}

pub struct Registry {
    tools: Vec<ToolDef>,
}

impl Default for Registry {
    fn default() -> Self {
        Registry { tools: defs::all() }
    }
}

impl Registry {
    pub fn all(&self) -> &[ToolDef] {
        &self.tools
    }
    pub fn get(&self, name: &str) -> Option<&ToolDef> {
        self.tools.iter().find(|t| t.name == name)
    }
}
