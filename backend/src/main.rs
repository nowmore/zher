use std::env;
use tracing::Level;
use zher::run_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let args: Vec<String> = env::args().collect();
    let host = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "0.0.0.0".to_string());
    let port = args.get(2).cloned().unwrap_or_else(|| "4836".to_string());

    run_server(host, port).await
}
