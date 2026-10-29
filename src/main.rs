use clap::Parser;
use deckcraft_http_mcp::config::{Config, LogLevel};
use deckcraft_http_mcp::server;

#[tokio::main]
async fn main() {
    let config = Config::parse();
    if let Err(e) = config.validate() {
        eprintln!("error: {e}");
        std::process::exit(2);
    }
    let addr = format!("{}:{}", config.bind, config.port);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot bind {addr}: {e}");
            std::process::exit(1);
        }
    };
    config.log(LogLevel::Info, &format!("listening on http://{addr}"));
    if let Err(e) = axum::serve(listener, server::router(config)).await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
