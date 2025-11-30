use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
    time::SystemTime,
};
use tokio::sync::mpsc;

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

#[derive(Default)]
pub struct AppState {
    // Map SessionID (String) -> Session
    pub sessions: HashMap<String, Session>,
    // Map SocketID -> SessionID
    pub socket_to_session: HashMap<String, String>,

    // file_id -> (sender_socket_id, filename, filesize)
    pub file_owners: HashMap<String, (String, String, u64)>,
    // transfer_id -> Sender Channel
    pub transfers: HashMap<String, mpsc::Sender<Result<Bytes, std::io::Error>>>,

    pub server_url: String,
}

pub type SharedState = Arc<RwLock<AppState>>;
