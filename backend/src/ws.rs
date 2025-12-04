use axum::extract::ConnectInfo;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socketioxide::extract::{Data, SocketRef, State as SocketState};
use std::{
    collections::HashSet,
    net::SocketAddr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tracing::info;

use crate::state::{Session, SharedState, User};
use crate::utils::{get_device_type, get_random_color};

#[derive(Debug, Deserialize)]
pub struct Auth {
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    #[serde(rename = "roomCode")]
    pub room_code: Option<String>,
}

pub async fn on_connect(
    socket: SocketRef,
    Data(auth): Data<Auth>,
    state: SocketState<SharedState>,
) {
    // Validate room code if enabled
    {
        let state_read = state.read().unwrap();
        if state_read.room_code_enabled {
            if let Some(ref expected_code) = state_read.room_code {
                match &auth.room_code {
                    Some(provided_code) if provided_code == expected_code => {
                        // Valid room code
                    }
                    Some(_) => {
                        // Invalid room code
                        let socket_id = socket.id.to_string();
                        let _ = socket.disconnect();
                        info!("Connection rejected: Invalid room code from {}", socket_id);
                        return;
                    }
                    None => {
                        // Room code required but not provided
                        let socket_id = socket.id.to_string();
                        let _ = socket.disconnect();
                        info!(
                            "Connection rejected: Room code required but not provided from {}",
                            socket_id
                        );
                        return;
                    }
                }
            }
        }
    }

    // Get IP from Axum ConnectInfo
    let ip: String = socket
        .req_parts()
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
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
    let server_url: String;

    {
        let mut state_write = state.write().unwrap();
        server_url = state_write.server_url.clone();

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
            let device = get_device_type(ua.unwrap_or(""));

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

        // Store the room code used for this socket connection
        state_write
            .socket_room_codes
            .insert(socket.id.to_string(), auth.room_code.clone());

        // Collect all users to send welcome
        let all_users: Vec<User> = state_write
            .sessions
            .values()
            .filter(|s| s.disconnect_time.is_none())
            .map(|s| s.user.clone())
            .collect();

        #[derive(Serialize)]
        struct WelcomeData {
            user: User,
            #[serde(rename = "allUsers")]
            all_users: Vec<User>,
            #[serde(rename = "serverUrl")]
            server_url: String,
        }

        let _ = socket.emit(
            "welcome",
            WelcomeData {
                user: user_profile.clone(),
                all_users,
                server_url: server_url.clone(),
            },
        );

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
                        .filter(|s| s.disconnect_time.is_none())
                        .map(|s| s.user.clone())
                        .collect();
                    let _ = socket
                        .broadcast()
                        .emit("update-user-list", (all_users.clone(),));
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

                    let id = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis();

                    let msg = Message {
                        id,
                        sender_id: sender.id.clone(),
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

                        let file_size = obj.get("fileSize").and_then(|v| v.as_u64()).unwrap_or(0);

                        obj.insert("fileId".to_string(), Value::String(file_id.clone()));

                        let id = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                        obj.insert(
                            "id".to_string(),
                            Value::Number(serde_json::Number::from(id as u64)),
                        );
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
                            .insert(file_id, (socket.id.to_string(), file_name, file_size));
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
                    let _ = socket.broadcast().emit("user-left", user_id);
                }
            }

            // Remove files owned by this socket (since socket is gone, transfer impossible)
            state_write
                .file_owners
                .retain(|_, v| v.0 != socket.id.to_string());

            // Remove room code tracking for this socket
            state_write.socket_room_codes.remove(&socket.id.to_string());
        },
    );
}
