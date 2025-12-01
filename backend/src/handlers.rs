use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use futures::StreamExt;
use rust_embed::RustEmbed;
use serde::Serialize;
use socketioxide::SocketIo;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::warn;

use crate::state::SharedState;

#[derive(RustEmbed)]
#[folder = "../frontend/dist"]
struct Assets;

// POST /upload/:transfer_id
pub async fn upload_file(
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
pub async fn download_file(
    Path(file_id): Path<String>,
    headers: HeaderMap,
    State(state): State<SharedState>,
    axum::Extension(io): axum::Extension<SocketIo>,
) -> Response {
    let (tx, rx) = mpsc::channel::<Result<Bytes, std::io::Error>>(2);
    let transfer_id = uuid::Uuid::new_v4().to_string();

    let file_info = {
        let state_read = state.read().unwrap();
        state_read.file_owners.get(&file_id).cloned()
    };

    if let Some((sender_id, filename, filesize)) = file_info {
        {
            let mut state_write = state.write().unwrap();
            state_write.transfers.insert(transfer_id.clone(), tx);
        }

        // Parse Range Header
        let mut start_byte = 0;
        let mut end_byte = filesize - 1;
        let mut is_partial = false;

        if let Some(range_header) = headers.get(header::RANGE) {
            if let Ok(range_str) = range_header.to_str() {
                if range_str.starts_with("bytes=") {
                    let ranges: Vec<&str> =
                        range_str.trim_start_matches("bytes=").split('-').collect();
                    if ranges.len() >= 1 {
                        if let Ok(s) = ranges[0].parse::<u64>() {
                            start_byte = s;
                            is_partial = true;
                        }
                    }
                    if ranges.len() >= 2 && !ranges[1].is_empty() {
                        if let Ok(e) = ranges[1].parse::<u64>() {
                            end_byte = e;
                            is_partial = true;
                        }
                    }
                }
            }
        }

        // Ensure bounds
        if start_byte > end_byte || start_byte >= filesize {
            return (StatusCode::RANGE_NOT_SATISFIABLE, "Invalid Range").into_response();
        }
        if end_byte >= filesize {
            end_byte = filesize - 1;
        }

        let content_length = end_byte - start_byte + 1;

        #[derive(Serialize)]
        struct StartUploadData {
            #[serde(rename = "fileId")]
            file_id: String,
            #[serde(rename = "transferId")]
            transfer_id: String,
            offset: u64,
            end: u64,
        }

        if let Err(e) = io.to(sender_id).emit(
            "start-upload",
            StartUploadData {
                file_id,
                transfer_id,
                offset: start_byte,
                end: end_byte,
            },
        ) {
            warn!("Failed to emit start-upload: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        let stream = ReceiverStream::new(rx);
        let encoded_filename = urlencoding::encode(&filename);
        let content_disposition = format!("attachment; filename*=UTF-8''{}", encoded_filename);

        let status = if is_partial {
            StatusCode::PARTIAL_CONTENT
        } else {
            StatusCode::OK
        };
        let content_range = format!("bytes {}-{}/{}", start_byte, end_byte, filesize);

        let mut response = Body::from_stream(stream).into_response();
        *response.status_mut() = status;

        let headers = response.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            "application/octet-stream".parse().unwrap(),
        );
        headers.insert(
            header::CONTENT_DISPOSITION,
            content_disposition.parse().unwrap(),
        );
        headers.insert(
            header::CONTENT_LENGTH,
            content_length.to_string().parse().unwrap(),
        );
        headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());

        if is_partial {
            headers.insert(header::CONTENT_RANGE, content_range.parse().unwrap());
        }

        response
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            
            if path == "index.html" || path.is_empty() {
                let html = String::from_utf8_lossy(&content.data);
                let injected_html = inject_tauri_mock(&html);
                ([(header::CONTENT_TYPE, mime.as_ref())], injected_html).into_response()
            } else {
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
        }
        None => {
            if let Some(content) = Assets::get("index.html") {
                let mime = mime_guess::from_path("index.html").first_or_octet_stream();
                let html = String::from_utf8_lossy(&content.data);
                let injected_html = inject_tauri_mock(&html);
                ([(header::CONTENT_TYPE, mime.as_ref())], injected_html).into_response()
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        }
    }
}

fn inject_tauri_mock(html: &str) -> String {
    let tauri_mock_script = r#"<script>
(function() {
    if (window.parent !== window) {
        window.__TAURI__ = window.__TAURI__ || {};
        window.__TAURI__.core = window.__TAURI__.core || {};
        window.__TAURI__.core.invoke = async (cmd, args) => {
            console.log('[Tauri Mock] invoke:', cmd, args);
            if (cmd === 'download_file') {
                window.parent.postMessage({ 
                    type: 'download_request', 
                    url: args.url, 
                    fileName: args.fileName
                }, '*');
                return Promise.resolve();
            }
            return Promise.reject('Command not implemented: ' + cmd);
        };
        window.__TAURI__.invoke = window.__TAURI__.core.invoke;
        window.__TAURI_INTERNALS__ = { postMessage: () => {} };
        console.log('[Tauri Mock] Injected via server');
    }
})();
</script>"#;
    
    if let Some(head_pos) = html.find("<head>") {
        let insert_pos = head_pos + "<head>".len();
        format!("{}{}{}", &html[..insert_pos], tauri_mock_script, &html[insert_pos..])
    } else {
        html.to_string()
    }
}
