use axum::{body::Body, http::{Request, StatusCode}};
use clap::Parser;
use deckcraft_http_mcp::{config::Config, server};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn send(cfg: Config, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(uri);
    let b = match body {
        Some(v) => {
            req = req.header("content-type", "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    let resp = server::router(cfg).oneshot(req.body(b).unwrap()).await.unwrap();
    let st = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (st, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

fn cfg(args: &[&str]) -> Config {
    Config::parse_from(std::iter::once("x").chain(args.iter().copied()))
}

#[tokio::test]
async fn health_and_list() {
    let (s, v) = send(cfg(&[]), "GET", "/health", None).await;
    assert_eq!((s, v["status"].as_str()), (StatusCode::OK, Some("ok")));
    let (_, v) = send(cfg(&[]), "POST", "/mcp/tools/list", Some(json!({}))).await;
    assert_eq!(v["tools"][0]["name"], "run_command");
    let (_, v) = send(cfg(&[]), "POST", "/mcp/tools/list?format=openai", None).await;
    assert_eq!(v["tools"][0]["type"], "function");
}

#[tokio::test]
async fn call_formats() {
    let args = json!({"command":"slide.new","params":{"title":"Hi"}});
    let cases = [
        ("/mcp/tools/call", json!({"name":"run_command","arguments":args})),
        ("/mcp/tools/call", json!({"type":"function","id":"c1","function":{"name":"run_command","arguments":args.to_string()}})),
        ("/mcp/tools/call", json!({"function_call":{"name":"run_command","arguments":args.to_string()}})),
        ("/mcp/tools/call", json!({"functionCall":{"name":"run_command","args":args}})),
        ("/mcp/tools/call", json!({"jsonrpc":"2.0","id":7,"method":"workspace/executeCommand","params":{"command":"run_command","arguments":[args]}})),
        ("/mcp/tools/call", json!({"jsonrpc":"2.0","id":8,"params":{"name":"run_command","arguments":args}})),
        ("/antigravity/execute", json!({"command":"run_command","args":args})),
    ];
    for (uri, body) in cases {
        let (s, v) = send(cfg(&[]), "POST", uri, Some(body.clone())).await;
        assert_eq!(s, StatusCode::OK, "{body}");
        assert!(v.to_string().contains("slides"), "{v}");
    }
    let (_, v) = send(cfg(&[]), "POST", "/mcp/tools/call", Some(json!({"name":"run_command","arguments":args}))).await;
    assert_eq!(v["isError"], false);
}

#[tokio::test]
async fn errors_per_format() {
    let (_, v) = send(cfg(&[]), "POST", "/mcp/tools/call", Some(json!({"name":"nope","arguments":{}}))).await;
    assert_eq!(v["isError"], true);
    let (_, v) = send(cfg(&[]), "POST", "/antigravity/execute", Some(json!({"command":"nope"}))).await;
    assert_eq!(v["status"], "error");
    let (s, _) = send(cfg(&[]), "POST", "/mcp/tools/call", Some(json!({"x":1}))).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn disabled_format_rejected() {
    let (s, _) = send(cfg(&["--formats","mcp"]), "POST", "/antigravity/execute", Some(json!({"command":"list_commands"}))).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn describe_and_resources() {
    let (s, v) = send(cfg(&[]), "GET", "/mcp/describe/run_command", None).await;
    assert_eq!((s, v["name"].as_str()), (StatusCode::OK, Some("run_command")));
    let (s, _) = send(cfg(&[]), "GET", "/mcp/describe/zzz", None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    let (_, v) = send(cfg(&[]), "GET", "/mcp/resources", None).await;
    assert!(v["resources"].is_array());
}

#[tokio::test]
async fn file_confinement() {
    let dir = std::env::temp_dir().join(format!("dc-conf-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let d = dir.to_str().unwrap();
    let c = || cfg(&["--sample", "--automation-read-root", d, "--automation-write-root", d]);
    let call = |p: &str, t: &str| json!({"name":t,"arguments":{"path":p}});
    let (_, v) = send(c(), "POST", "/mcp/tools/call", Some(call("out.json", "save_deck"))).await;
    assert_eq!(v["isError"], false, "{v}");
    assert!(dir.join("out.json").exists());
    for p in ["../evil.json", "/etc/evil.json"] {
        let (_, v) = send(c(), "POST", "/mcp/tools/call", Some(call(p, "save_deck"))).await;
        assert_eq!(v["isError"], true, "{p}");
    }
    let (_, v) = send(c(), "POST", "/mcp/tools/call", Some(call("/etc/passwd", "open_deck"))).await;
    assert_eq!(v["isError"], true);
    std::fs::remove_dir_all(dir).ok();
}

#[tokio::test]
async fn connected_backend() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = l.local_addr().unwrap().port().to_string();
    tokio::spawn(async move {
        let (sock, _) = l.accept().await.unwrap();
        let (r, mut w) = sock.into_split();
        let mut line = String::new();
        BufReader::new(r).read_line(&mut line).await.unwrap();
        let req: Value = serde_json::from_str(&line).unwrap();
        let resp = json!({"id":req["id"],"result":{"content":[{"type":"text","text":"from-deckcraft"}]}});
        w.write_all(format!("{resp}\n").as_bytes()).await.unwrap();
    });
    let c = cfg(&["--mode", "connect", "--connect", &port]);
    let (_, v) = send(c, "POST", "/mcp/tools/call", Some(json!({"name":"list_commands"}))).await;
    assert_eq!(v["content"][0]["text"], "from-deckcraft");
}
