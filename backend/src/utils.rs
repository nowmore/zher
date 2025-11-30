use rand::seq::SliceRandom;

const COLORS: &[&str] = &[
    "#ef4444", "#f97316", "#f59e0b", "#84cc16", "#10b981", "#06b6d4", "#3b82f6", "#6366f1",
    "#8b5cf6", "#d946ef",
];

pub fn get_random_color() -> String {
    let mut rng = rand::thread_rng();
    COLORS.choose(&mut rng).unwrap().to_string()
}

pub fn get_device_type(ua: Option<&str>) -> String {
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
