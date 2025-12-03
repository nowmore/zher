use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::task;
use tracing::{error, info};

pub struct DiscoveryService {
    enabled: Arc<AtomicBool>,
}

impl DiscoveryService {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(enabled)),
        }
    }

    pub fn start(&self) {
        let enabled = self.enabled.clone();
        task::spawn_blocking(move || {
            if let Err(e) = run_discovery_responder(enabled) {
                error!("Discovery responder error: {}", e);
            }
        });
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }
}

fn run_discovery_responder(enabled: Arc<AtomicBool>) -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:4837")?;
    socket.set_read_timeout(Some(Duration::from_secs(1)))?;

    let mut buf = [0u8; 1024];

    loop {
        if !enabled.load(Ordering::SeqCst) {
            // Sleep when disabled to avoid busy loop
            std::thread::sleep(Duration::from_millis(100));
            continue;
        }

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
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                // Timeout, continue loop
                continue;
            }
            Err(e) => {
                error!("Failed to receive discovery packet: {}", e);
            }
        }
    }
}
