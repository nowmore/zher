pub mod discovery;
pub mod handlers;
pub mod state;
pub mod utils;
pub mod ws;

use axum::{
    routing::{get, post},
    Router,
};
use local_ip_address::local_ip;
use socketioxide::SocketIo;
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::discovery::start_discovery_responder;
use crate::handlers::{download_file, static_handler, upload_file};
use crate::state::AppState;
use crate::ws::on_connect;

pub async fn run_server(host: String, port: String) -> Result<(), Box<dyn std::error::Error>> {
    let addr_str = format!("{}:{}", host, port);

    // Determine server URL for QR code
    let my_local_ip = local_ip().unwrap_or("127.0.0.1".parse().unwrap());
    
    let display_host = if host == "0.0.0.0" {
        my_local_ip.to_string()
    } else {
        host.clone()
    };
    let server_url = format!("http://{}:{}", display_host, port);

    let mut state_val = AppState::default();
    state_val.server_url = server_url.clone();
    let state = Arc::new(RwLock::new(state_val));

    let (layer, io) = SocketIo::builder()
        .with_state(state.clone())
        .max_payload(100 * 1024 * 1024) // 100MB
        .build_layer();

    io.ns("/", on_connect);

    let app = Router::new()
        .route("/api/upload/:transfer_id", post(upload_file))
        .route("/api/download/:file_id", get(download_file))
        .fallback(static_handler)
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .layer(layer)
                .layer(axum::Extension(io)),
        )
        .with_state(state);

    let listener = TcpListener::bind(&addr_str).await?;
    info!("Server running on {}", server_url);
    info!("Listening on {}", addr_str);

    start_discovery_responder();
    info!("Discovery responder started");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
