use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use bytes::Bytes;
use futures::StreamExt;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socketioxide::{
    extract::{Data, SocketRef, State as SocketState},
    SocketIo,
};
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, Level};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
    id: String, // Stable ID based on session/IP persistence
    name: String,
    color: String,
    device: String,
}

#[derive(Clone, Debug)]
struct Session {
    user: User,
    disconnect_time: Option<SystemTime>,
    active_sockets: HashSet<String>,
}

#[derive(Default)]
struct AppState {
    // Map SessionID (String) -> Session
    sessions: HashMap<String, Session>,
    // Map SocketID -> SessionID
    socket_to_session: HashMap<String, String>,

    // file_id -> (sender_socket_id, filename)
    file_owners: HashMap<String, (String, String)>,
    // transfer_id -> Sender Channel
    transfers: HashMap<String, mpsc::Sender<Result<Bytes, std::io::Error>>>,
}

type SharedState = Arc<RwLock<AppState>>;

#[derive(RustEmbed)]
#[folder = "../frontend/dist"]
struct Assets;

const COLORS: &[&str] = &[
    "#ef4444", "#f97316", "#f59e0b", "#84cc16", "#10b981", "#06b6d4", "#3b82f6", "#6366f1",
    "#8b5cf6", "#d946ef",
];

fn get_random_color() -> String {
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    COLORS.choose(&mut rng).unwrap().to_string()
}

fn get_device_type(ua: Option<&str>) -> String {
    match ua {
        Some(ua) => {
            let ua = ua.to_lowercase();
            if ua.contains("mobile")
                || ua.contains("android")
                || ua.contains("iphone")
                || ua.contains("ipad")
                || ua.contains("ipod")
            {
                "mobile".to_string()
            } else {
                "desktop".to_string()
            }
        }
        None => "desktop".to_string(),
    }
}

#[derive(Debug, Deserialize)]
struct Auth {
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

// POST /upload/:transfer_id
async fn upload_file(
    Path(transfer_id): Path<String>,
    State(state): State<SharedState>,
    body: Body,
) -> impl IntoResponse {
    let tx = {
        let mut state_write = state.write().unwrap();
        state_write.transfers.remove(&transfer_id)
    };

    if let Some(tx) = tx {
        let mut stream = body.into_data_stream();
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    if tx.send(Ok(bytes)).await.is_err() {
                        // Receiver dropped
                        break;
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            e.to_string(),
                        )))
                        .await;
                    break;
                }
            }
        }
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

// GET /download/:file_id
async fn download_file(
    Path(file_id): Path<String>,
    State(state): State<SharedState>,
    axum::Extension(io): axum::Extension<SocketIo>,
) -> Response {
    let (tx, rx) = mpsc::channel::<Result<Bytes, std::io::Error>>(2);
    let transfer_id = uuid::Uuid::new_v4().to_string();

    let file_info = {
        let state_read = state.read().unwrap();
        state_read.file_owners.get(&file_id).cloned()
    };

    if let Some((sender_id, filename)) = file_info {
        {
            let mut state_write = state.write().unwrap();
            state_write.transfers.insert(transfer_id.clone(), tx);
        }

        #[derive(Serialize)]
        struct StartUploadData {
            #[serde(rename = "fileId")]
            file_id: String,
            #[serde(rename = "transferId")]
            transfer_id: String,
        }

        if let Err(e) = io.to(sender_id).emit(
            "start-upload",
            StartUploadData {
                file_id,
                transfer_id,
            },
        ) {
            warn!("Failed to emit start-upload: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        let stream = ReceiverStream::new(rx);
        let encoded_filename = urlencoding::encode(&filename);
        let content_disposition = format!("attachment; filename*=UTF-8''{}", encoded_filename);

        (
            [
                (header::CONTENT_TYPE, "application/octet-stream".to_string()),
                (header::CONTENT_DISPOSITION, content_disposition),
            ],
            Body::from_stream(stream),
        )
            .into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if let Some(content) = Assets::get("index.html") {
                let mime = mime_guess::from_path("index.html").first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        }
    }
}

async fn on_connect(socket: SocketRef, Data(auth): Data<Auth>, state: SocketState<SharedState>) {
    // Get IP from Axum ConnectInfo
    let ip: String = socket
        .req_parts()
        .extensions
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Use session_id from auth, or fallback to generated ID
    let session_key = auth
        .session_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let _ = socket.join(socket.id.to_string()); // Join own room

    info!(
        "Socket connected: {} from IP {} with SessionID {}",
        socket.id, ip, session_key
    );

    let user_profile: User;

    {
        let mut state_write = state.write().unwrap();

        // Check if session exists for this SessionID
        let session_exists = if let Some(session) = state_write.sessions.get_mut(&session_key) {
            // Check if session expired (disconnected > 10 mins ago)
            if let Some(disconnect_time) = session.disconnect_time {
                if SystemTime::now()
                    .duration_since(disconnect_time)
                    .unwrap_or(Duration::ZERO)
                    > Duration::from_secs(600)
                {
                    // Expired
                    false
                } else {
                    // Valid, reactivate
                    session.disconnect_time = None;
                    session.active_sockets.insert(socket.id.to_string());
                    true
                }
            } else {
                // Already active (another tab/device with same IP)
                session.active_sockets.insert(socket.id.to_string());
                true
            }
        } else {
            false
        };

        if session_exists {
            // Reuse existing profile
            user_profile = state_write.sessions.get(&session_key).unwrap().user.clone();
        } else {
            // Create new session/user
            let name = format!(
                "{}",
                uuid::Uuid::new_v4().simple().to_string()[..6].to_string()
            );
            let color = get_random_color();
            let ua = socket
                .req_parts()
                .headers
                .get("user-agent")
                .and_then(|h| h.to_str().ok());
            let device = get_device_type(ua);

            user_profile = User {
                id: uuid::Uuid::new_v4().to_string(), // Stable ID
                name,
                color,
                device,
            };

            let mut active_sockets = HashSet::new();
            active_sockets.insert(socket.id.to_string());

            let session = Session {
                user: user_profile.clone(),
                disconnect_time: None,
                active_sockets,
            };

            state_write.sessions.insert(session_key.clone(), session);
        }

        // Update socket_to_session map
        state_write
            .socket_to_session
            .insert(socket.id.to_string(), session_key.clone());

        // Collect all users to send welcome
        let all_users: Vec<User> = state_write
            .sessions
            .values()
            .map(|s| s.user.clone())
            .collect();

        #[derive(Serialize)]
        struct WelcomeData {
            user: User,
            #[serde(rename = "allUsers")]
            all_users: Vec<User>,
        }

        let _ = socket.emit(
            "welcome",
            WelcomeData {
                user: user_profile.clone(),
                all_users,
            },
        );

        // Only broadcast user-joined if this is a truly new user (session created or re-activated from offline state)
        // However, if session existed but had 0 sockets, we effectively re-joined.
        // But wait, if we don't remove them from list on disconnect (only mark offline), we don't need to emit joined?
        // If we remove them from list on disconnect, then we must emit joined.
        // Decision: Remove from list on disconnect (visually offline). So re-appearing means joined.
        // Check: if session was newly created OR (it existed but active_sockets was empty before this insertion).
        // We can check active_sockets.len() == 1.
        if state_write
            .sessions
            .get(&session_key)
            .unwrap()
            .active_sockets
            .len()
            == 1
        {
            let _ = socket.broadcast().emit("user-joined", user_profile.clone());
        }
    }

    socket.on(
        "request-name-change",
        |socket: SocketRef, Data::<String>(new_name), state: SocketState<SharedState>| async move {
            let final_name = new_name.trim();
            if final_name.is_empty() || final_name.chars().count() > 24 {
                let _ = socket.emit("name-change-fail", "名字无效或太长");
                return;
            }

            let mut state_write = state.write().unwrap();

            // Find session by socket
            if let Some(session_key) = state_write
                .socket_to_session
                .get(&socket.id.to_string())
                .cloned()
            {
                // Check name collision
                let is_taken = state_write.sessions.values().any(|s| {
                    s.user.name == final_name
                        && s.user.id != state_write.sessions[&session_key].user.id
                });
                let final_name = if is_taken {
                    format!("{}1", final_name)
                } else {
                    final_name.to_string()
                };

                if let Some(session) = state_write.sessions.get_mut(&session_key) {
                    session.user.name = final_name.clone();
                    let _ = socket.emit("name-change-success", &final_name);

                    let all_users: Vec<User> = state_write
                        .sessions
                        .values()
                        .map(|s| s.user.clone())
                        .collect();
                    let _ = socket
                        .broadcast()
                        .emit("update-user-list", (all_users.clone(),));
                    // Also update current socket's list (redundant if broadcast hits all?) broadcast excludes sender
                    let _ = socket.emit("update-user-list", (all_users,));
                }
            }
        },
    );

    socket.on(
        "text-message",
        |socket: SocketRef, Data::<String>(text), state: SocketState<SharedState>| async move {
            let state_read = state.read().unwrap();
            if let Some(session_key) = state_read.socket_to_session.get(&socket.id.to_string()) {
                if let Some(session) = state_read.sessions.get(session_key) {
                    let sender = &session.user;
                    #[derive(Serialize)]
                    struct Message {
                        id: u128,
                        #[serde(rename = "senderId")]
                        sender_id: String,
                        #[serde(rename = "senderName")]
                        sender_name: String,
                        #[serde(rename = "senderColor")]
                        sender_color: String,
                        #[serde(rename = "senderDevice")]
                        sender_device: String,
                        #[serde(rename = "type")]
                        msg_type: String,
                        text: String,
                    }

                    use std::time::{SystemTime, UNIX_EPOCH};
                    let id = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis();

                    let msg = Message {
                        id,
                        sender_id: sender.id.clone(), // Use Stable ID
                        sender_name: sender.name.clone(),
                        sender_color: sender.color.clone(),
                        sender_device: sender.device.clone(),
                        msg_type: "text".to_string(),
                        text,
                    };

                    let _ = socket.broadcast().emit("message", &msg);
                    let _ = socket.emit("message", &msg);
                }
            }
        },
    );

    socket.on(
        "file-meta",
        |socket: SocketRef, Data::<Value>(mut meta), state: SocketState<SharedState>| async move {
            let mut state_write = state.write().unwrap();
            let session_key = state_write
                .socket_to_session
                .get(&socket.id.to_string())
                .cloned();

            if let Some(session_key) = session_key {
                if let Some(session) = state_write.sessions.get(&session_key).cloned() {
                    // Clone user to avoid borrow check
                    let sender = &session.user;
                    if let Some(obj) = meta.as_object_mut() {
                        let file_id = obj
                            .get("fileId")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

                        let file_name = obj
                            .get("fileName")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown_file")
                            .to_string();

                        obj.insert("fileId".to_string(), Value::String(file_id.clone()));

                        use std::time::{SystemTime, UNIX_EPOCH};
                        let id = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                        obj.insert(
                            "id".to_string(),
                            Value::Number(serde_json::Number::from(id as u64)),
                        );
                        // Use Stable ID
                        obj.insert("senderId".to_string(), Value::String(sender.id.clone()));
                        obj.insert("senderName".to_string(), Value::String(sender.name.clone()));
                        obj.insert(
                            "senderColor".to_string(),
                            Value::String(sender.color.clone()),
                        );
                        obj.insert(
                            "senderDevice".to_string(),
                            Value::String(sender.device.clone()),
                        );
                        obj.insert("type".to_string(), Value::String("file-meta".to_string()));

                        state_write
                            .file_owners
                            .insert(file_id, (socket.id.to_string(), file_name));
                        // Keep socket ID for file transfer routing
                    }
                    let _ = socket.broadcast().emit("message", &meta);
                    let _ = socket.emit("message", &meta);
                }
            }
        },
    );

    socket.on_disconnect(
        |socket: SocketRef, state: SocketState<SharedState>| async move {
            let mut state_write = state.write().unwrap();
            // Get SessionID
            if let Some(session_key) = state_write.socket_to_session.remove(&socket.id.to_string())
            {
                let mut remove_user = false;
                let mut user_id = String::new();

                if let Some(session) = state_write.sessions.get_mut(&session_key) {
                    session.active_sockets.remove(&socket.id.to_string());
                    if session.active_sockets.is_empty() {
                        // All tabs closed
                        session.disconnect_time = Some(SystemTime::now());
                        remove_user = true;
                        user_id = session.user.id.clone();
                    }
                }

                if remove_user {
                    // Emit user-left.
                    // NOTE: We do NOT remove from sessions map yet, we keep it for 10 mins.
                    // But we tell frontend they left so they disappear from list.
                    let _ = socket.broadcast().emit("user-left", user_id);
                }
            }

            // Remove files owned by this socket (since socket is gone, transfer impossible)
            state_write
                .file_owners
                .retain(|_, v| v.0 != socket.id.to_string());
        },
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let state = Arc::new(RwLock::new(AppState::default()));

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

    let listener = TcpListener::bind("0.0.0.0:4836").await?;
    info!("Server running on http://0.0.0.0:4836");

    // Ensure ConnectInfo is provided
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
