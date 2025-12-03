use std::env;
use std::net::TcpListener as StdTcpListener;
use tracing::Level;
use zher::run_server;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use zher::desktop;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let args: Vec<String> = env::args().collect();
    let host = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "0.0.0.0".to_string());
    let port = args.get(2).cloned().unwrap_or_else(|| "4836".to_string());

    let display_host = if host == "0.0.0.0" || host == "::" {
        "localhost"
    } else {
        &host
    };
    let url = format!("http://{}:{}", display_host, port);

    let addr_str = format!("{}:{}", host, port);
    if StdTcpListener::bind(&addr_str).is_err() {
        tracing::error!("Failed to bind to {}", addr_str);
        return Err(format!("Port {} is already in use", port).into());
    }

    tracing::info!("Server starting at {}", url);

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    desktop::setup_desktop_features(url).await;

    run_server(host, port).await
}
