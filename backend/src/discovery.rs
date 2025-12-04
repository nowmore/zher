use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{error, info};

pub struct DiscoveryService {
    enabled: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl DiscoveryService {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(enabled)),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    pub fn start(&mut self) {
        // Don't start if already running
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(true, Ordering::SeqCst);
        let enabled = self.enabled.clone();
        let running = self.running.clone();
        
        let handle = tokio::task::spawn_blocking(move || {
            if let Err(e) = run_discovery_responder(enabled, running) {
                error!("Discovery responder error: {}", e);
            }
        });
        
        self.handle = Some(handle);
    }

    pub fn stop(&mut self) {
        info!("Stopping discovery service...");
        self.running.store(false, Ordering::SeqCst);
        
        if let Some(handle) = self.handle.take() {
            handle.abort();
            info!("Discovery service stopped");
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::SeqCst);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

fn run_discovery_responder(enabled: Arc<AtomicBool>, running: Arc<AtomicBool>) -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:4837")?;
    socket.set_read_timeout(Some(Duration::from_millis(500)))?;

    let mut buf = [0u8; 1024];

    info!("Discovery responder started on port 4837");

    loop {
        // Check if we should stop
        if !running.load(Ordering::SeqCst) {
            info!("Discovery responder stopping...");
            break;
        }

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
                // Timeout, continue loop to check running flag
                continue;
            }
            Err(e) => {
                error!("Failed to receive discovery packet: {}", e);
                // Don't break on error, continue trying
            }
        }
    }

    info!("Discovery responder stopped");
    Ok(())
}
