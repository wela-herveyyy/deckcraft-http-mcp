use super::ToolOutput;
use crate::config::Config;
use serde_json::{Value, json};
use std::path::{Component, Path, PathBuf};
use tokio::sync::Mutex;

/// Standalone in-process deck session (headless mode).
pub struct Session {
    deck: Mutex<Vec<Value>>,
    read_root: Option<PathBuf>,
    write_root: Option<PathBuf>,
}

fn sample_deck() -> Vec<Value> {
    vec![
        json!({"layout":"titleOnly","title":"Welcome to DeckCraft"}),
        json!({"layout":"titleOnly","title":"Sample Slide"}),
    ]
}

/// Resolve `path` inside `root`. Relative paths are joined to the root;
/// `..` components and paths escaping the root are rejected.
pub fn confine(root: &Option<PathBuf>, path: &str) -> Result<PathBuf, String> {
    let p = Path::new(path);
    let Some(root) = root else { return Ok(p.to_path_buf()) };
    let root = root.canonicalize().map_err(|e| format!("invalid root: {e}"))?;
    let joined = if p.is_absolute() { p.to_path_buf() } else { root.join(p) };
    if joined.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(format!("path escapes allowed root: {path}"));
    }
    // canonicalize the deepest existing ancestor to defeat symlink escapes
    let mut existing = joined.as_path();
    while !existing.exists() {
        existing = existing.parent().ok_or("invalid path")?;
    }
    let canon = existing.canonicalize().map_err(|e| e.to_string())?;
    if !canon.starts_with(&root) {
        return Err(format!("path escapes allowed root: {path}"));
    }
    Ok(joined)
}

impl Session {
    pub fn new(config: &Config) -> Self {
        Session {
            deck: Mutex::new(if config.sample { sample_deck() } else { vec![] }),
            read_root: config.automation_read_root.clone(),
            write_root: config.automation_write_root.clone(),
        }
    }

    pub async fn call(&self, name: &str, args: Value) -> ToolOutput {
        match self.dispatch(name, &args).await {
            Ok(v) => ToolOutput::ok(v),
            Err(e) => ToolOutput::err(e),
        }
    }

    async fn dispatch(&self, name: &str, args: &Value) -> Result<Value, String> {
        match name {
            "run_command" => {
                let cmd = args.get("command").and_then(Value::as_str).ok_or("missing 'command'")?;
                let params = args.get("params").cloned().unwrap_or_else(|| json!({}));
                self.run_command(cmd, &params).await
            }
            "list_commands" => Ok(json!(COMMANDS)),
            "open_deck" => {
                let path = args.get("path").and_then(Value::as_str).ok_or("missing 'path'")?;
                let p = confine(&self.read_root, path)?;
                let data = tokio::fs::read_to_string(&p).await.map_err(|e| e.to_string())?;
                let slides: Vec<Value> = serde_json::from_str(&data).map_err(|e| format!("invalid deck file: {e}"))?;
                let n = slides.len();
                *self.deck.lock().await = slides;
                Ok(json!({"opened": path, "slides": n}))
            }
            "save_deck" => {
                let path = args.get("path").and_then(Value::as_str).ok_or("missing 'path'")?;
                let p = confine(&self.write_root, path)?;
                let data = serde_json::to_string_pretty(&*self.deck.lock().await).map_err(|e| e.to_string())?;
                tokio::fs::write(&p, data).await.map_err(|e| e.to_string())?;
                Ok(json!({"saved": path}))
            }
            other => Err(format!("unknown tool: {other}")),
        }
    }

    async fn run_command(&self, cmd: &str, params: &Value) -> Result<Value, String> {
        let mut deck = self.deck.lock().await;
        let index = |deck: &Vec<Value>| -> Result<usize, String> {
            let i = params.get("index").and_then(Value::as_u64).ok_or("missing 'index'")? as usize;
            if i < deck.len() { Ok(i) } else { Err(format!("slide index out of range: {i}")) }
        };
        match cmd {
            "slide.new" => {
                let layout = params.get("layout").and_then(Value::as_str).unwrap_or("titleOnly");
                let title = params.get("title").and_then(Value::as_str).unwrap_or("");
                deck.push(json!({"layout":layout,"title":title}));
                Ok(json!({"index": deck.len() - 1, "slides": deck.len()}))
            }
            "slide.list" | "deck.get" => Ok(json!({"slides": *deck})),
            "slide.delete" => {
                let i = index(&deck)?;
                deck.remove(i);
                Ok(json!({"slides": deck.len()}))
            }
            "slide.set_title" => {
                let i = index(&deck)?;
                let t = params.get("title").and_then(Value::as_str).ok_or("missing 'title'")?;
                deck[i]["title"] = json!(t);
                Ok(json!({"index": i, "title": t}))
            }
            other => Err(format!("unknown command: {other}")),
        }
    }
}

pub const COMMANDS: &[&str] = &["slide.new", "slide.list", "slide.delete", "slide.set_title", "deck.get"];
