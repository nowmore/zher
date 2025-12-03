use std::env;

pub fn is_desktop_environment() -> bool {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        return false;
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        env::var("DISPLAY").is_ok()
            || env::var("WAYLAND_DISPLAY").is_ok()
            || cfg!(target_os = "windows")
            || cfg!(target_os = "macos")
    }
}

pub fn open_browser(url: &str) {
    if let Err(e) = webbrowser::open(url) {
        tracing::warn!("Failed to open browser: {}", e);
    }
}

pub async fn setup_desktop_features(url: String) {
    if !is_desktop_environment() {
        return;
    }

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        open_browser(&url);
    });
}
