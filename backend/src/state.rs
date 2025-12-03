use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
    time::SystemTime,
};
use tokio::sync::mpsc;

use crate::discovery::DiscoveryService;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub color: String,
    pub device: String,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub user: User,
    pub disconnect_time: Option<SystemTime>,
    pub active_sockets: HashSet<String>,
}

pub struct AppState {
    // Map SessionID (String) -> Session
    pub sessions: HashMap<String, Session>,
    // Map SocketID -> SessionID
    pub socket_to_session: HashMap<String, String>,
    // Map SocketID -> Room Code (for validation)
    pub socket_room_codes: HashMap<String, Option<String>>,

    // file_id -> (sender_socket_id, filename, filesize)
    pub file_owners: HashMap<String, (String, String, u64)>,
    // transfer_id -> Sender Channel
    pub transfers: HashMap<String, mpsc::Sender<Result<Bytes, std::io::Error>>>,

    pub server_url: String,
    pub discovery: Arc<DiscoveryService>,
    
    // Room code settings
    pub room_code_enabled: bool,
    pub room_code: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            socket_to_session: HashMap::new(),
            socket_room_codes: HashMap::new(),
            file_owners: HashMap::new(),
            transfers: HashMap::new(),
            server_url: String::new(),
            discovery: Arc::new(DiscoveryService::new(true)),
            room_code_enabled: false,
            room_code: None,
        }
    }
}

pub type SharedState = Arc<RwLock<AppState>>;
