pub mod deckcraft;
pub mod session;

use crate::config::{Config, Mode};
use serde_json::Value;

/// Result of executing a tool.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolOutput {
    pub text: String,
    pub is_error: bool,
}

impl ToolOutput {
    pub fn ok(v: Value) -> Self {
        let text = match v {
            Value::String(s) => s,
            other => serde_json::to_string_pretty(&other).unwrap_or_default(),
        };
        ToolOutput { text, is_error: false }
    }
    pub fn err(msg: impl Into<String>) -> Self {
        ToolOutput { text: msg.into(), is_error: true }
    }
}

pub enum Backend {
    Headless(session::Session),
    Connected(deckcraft::DeckcraftClient),
}

impl Backend {
    pub fn from_config(config: &Config) -> Self {
        match config.mode {
            Mode::Headless => Backend::Headless(session::Session::new(config)),
            Mode::Connect => Backend::Connected(deckcraft::DeckcraftClient::new(
                config.control_host(),
                config.connect_port.unwrap_or(0),
            )),
        }
    }

    pub async fn call(&self, name: &str, args: Value) -> ToolOutput {
        match self {
            Backend::Headless(s) => s.call(name, args).await,
            Backend::Connected(c) => c.call(name, args).await,
        }
    }

    pub fn mode_name(&self) -> &'static str {
        match self {
            Backend::Headless(_) => "headless",
            Backend::Connected(_) => "connect",
        }
    }
}
