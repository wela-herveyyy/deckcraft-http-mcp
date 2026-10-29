use super::ToolOutput;
use serde_json::{Value, json};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

/// Client for a running `deckcraft --control PORT` instance.
/// Speaks newline-delimited JSON-RPC (`tools/call`) over TCP.
pub struct DeckcraftClient {
    addr: String,
}

impl DeckcraftClient {
    pub fn new(host: &str, port: u16) -> Self {
        DeckcraftClient { addr: format!("{host}:{port}") }
    }

    pub async fn call(&self, name: &str, args: Value) -> ToolOutput {
        match tokio::time::timeout(Duration::from_secs(30), self.round_trip(name, args)).await {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => ToolOutput::err(format!("DeckCraft backend error: {e}")),
            Err(_) => ToolOutput::err("DeckCraft backend timed out"),
        }
    }

    async fn round_trip(&self, name: &str, args: Value) -> Result<ToolOutput, String> {
        let stream = TcpStream::connect(&self.addr)
            .await
            .map_err(|e| format!("cannot connect to {}: {e}", self.addr))?;
        let (r, mut w) = stream.into_split();
        let req = json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":name,"arguments":args}});
        let mut line = req.to_string();
        line.push('\n');
        w.write_all(line.as_bytes()).await.map_err(|e| e.to_string())?;
        let mut resp = String::new();
        BufReader::new(r).read_line(&mut resp).await.map_err(|e| e.to_string())?;
        let v: Value = serde_json::from_str(resp.trim()).map_err(|e| format!("bad response: {e}"))?;
        if let Some(e) = v.get("error") {
            let msg = e.get("message").and_then(Value::as_str).unwrap_or("unknown error");
            return Ok(ToolOutput::err(msg));
        }
        let result = v.get("result").cloned().unwrap_or(Value::Null);
        if let Some(items) = result.get("content").and_then(Value::as_array) {
            let text = items
                .iter()
                .filter_map(|i| i.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("\n");
            let is_error = result.get("isError").and_then(Value::as_bool).unwrap_or(false);
            return Ok(ToolOutput { text, is_error });
        }
        Ok(ToolOutput::ok(result))
    }
}
