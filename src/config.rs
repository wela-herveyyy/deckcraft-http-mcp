use crate::formats::Format;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Mode {
    Headless,
    Connect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug, Parser)]
#[command(name = "deckcraft-http-mcp", about = "Remote HTTP MCP server for DeckCraft")]
pub struct Config {
    /// HTTP server port
    #[arg(long, default_value_t = 3000)]
    pub port: u16,
    /// Bind address
    #[arg(long, default_value = "127.0.0.1")]
    pub bind: String,
    /// Backend mode
    #[arg(long, value_enum, default_value_t = Mode::Headless)]
    pub mode: Mode,
    /// DeckCraft control port (connect mode)
    #[arg(long = "connect")]
    pub connect_port: Option<u16>,
    /// Load sample deck (headless mode)
    #[arg(long)]
    pub sample: bool,
    /// Confine reads to this directory
    #[arg(long)]
    pub automation_read_root: Option<PathBuf>,
    /// Confine writes to this directory
    #[arg(long)]
    pub automation_write_root: Option<PathBuf>,
    /// Logging level
    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,
    /// Enabled formats (comma separated; default: all)
    #[arg(long, value_delimiter = ',', default_values_t = Format::all_names())]
    pub formats: Vec<String>,
    /// CORS allowed origins (comma separated; "*" for any)
    #[arg(long, value_delimiter = ',')]
    pub allowed_origins: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config::parse_from(["deckcraft-http-mcp"])
    }
}

impl Config {
    pub fn enabled_formats(&self) -> Result<Vec<Format>, String> {
        self.formats
            .iter()
            .map(|n| Format::from_name(n.trim()).ok_or_else(|| format!("unknown format: {n}")))
            .collect()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.mode == Mode::Connect && self.connect_port.is_none() {
            return Err("--mode connect requires --connect PORT".into());
        }
        self.enabled_formats().map(|_| ())
    }

    pub fn log(&self, level: LogLevel, msg: &str) {
        if level >= self.log_level {
            eprintln!("[{level:?}] {msg}");
        }
    }
}

impl Config {
    /// Host used to reach a running DeckCraft control port.
    pub fn control_host(&self) -> &str {
        "127.0.0.1"
    }
}
