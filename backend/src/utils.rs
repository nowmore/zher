use rand::Rng;

pub fn generate_room_code() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(0..1000000))
}

pub fn validate_room_code(code: &str) -> bool {
    code.len() == 6 && code.chars().all(|c| c.is_ascii_digit())
}

pub fn get_random_color() -> String {
    let mut rng = rand::thread_rng();
    let colors = [
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#FFA07A", "#98D8C8",
        "#F7DC6F", "#BB8FCE", "#85C1E2", "#F8B739", "#52B788",
    ];
    colors[rng.gen_range(0..colors.len())].to_string()
}

pub fn get_device_type(user_agent: &str) -> String {
    let ua = user_agent.to_lowercase();
    if ua.contains("mobile") || ua.contains("android") || ua.contains("iphone") {
        "mobile".to_string()
    } else if ua.contains("tablet") || ua.contains("ipad") {
        "tablet".to_string()
    } else {
        "desktop".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_room_code() {
        let code = generate_room_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_validate_room_code_valid() {
        assert!(validate_room_code("123456"));
        assert!(validate_room_code("000000"));
        assert!(validate_room_code("999999"));
    }

    #[test]
    fn test_validate_room_code_invalid() {
        assert!(!validate_room_code("12345")); // Too short
        assert!(!validate_room_code("1234567")); // Too long
        assert!(!validate_room_code("12345a")); // Contains letter
        assert!(!validate_room_code("12-456")); // Contains dash
        assert!(!validate_room_code("")); // Empty
    }

    #[test]
    fn test_get_random_color() {
        let color = get_random_color();
        assert!(color.starts_with('#'));
        assert_eq!(color.len(), 7);
    }

    #[test]
    fn test_get_device_type() {
        assert_eq!(get_device_type("Mozilla/5.0 (iPhone)"), "mobile");
        assert_eq!(get_device_type("Mozilla/5.0 (Android)"), "mobile");
        assert_eq!(get_device_type("Mozilla/5.0 (iPad)"), "tablet");
        assert_eq!(get_device_type("Mozilla/5.0 (Windows NT)"), "desktop");
        assert_eq!(get_device_type("Mozilla/5.0 (Macintosh)"), "desktop");
    }
}
