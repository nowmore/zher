use std::net::UdpSocket;
use tokio::task;
use tracing::{error, info};

pub fn start_discovery_responder() {
    task::spawn_blocking(move || {
        if let Err(e) = run_discovery_responder() {
            error!("Discovery responder error: {}", e);
        }
    });
}

fn run_discovery_responder() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:4837")?;
    info!("Discovery responder listening on 0.0.0.0:4837");

    let mut buf = [0u8; 1024];
    
    loop {
        match socket.recv_from(&mut buf) {
            Ok((size, src)) => {
                let msg = String::from_utf8_lossy(&buf[..size]);
                
                if msg.trim() == "ZHER_DISCOVERY" {
                    info!("Received discovery request from {}", src);
                    
                    let response = b"ZHER_SERVICE:4836";
                    if let Err(e) = socket.send_to(response, src) {
                        error!("Failed to send discovery response: {}", e);
                    } else {
                        info!("Sent discovery response to {}", src);
                    }
                }
            }
            Err(e) => {
                error!("Failed to receive discovery packet: {}", e);
            }
        }
    }
}
