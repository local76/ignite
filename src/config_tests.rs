use super::*;

#[test]
fn test_default_config() {
    let cfg = AppConfig::default();
    assert_eq!(cfg.theme_mode, "auto");
    assert_eq!(cfg.refresh_rate_ms, 100);
    assert!(!cfg.enable_borderless);
    assert!(cfg.enable_toasts);
    assert!(cfg.enable_event_log);
}

#[test]
fn test_parse_fields() {
    let mut cfg = AppConfig::default();
    cfg.parse_field("theme_mode", "dark");
    cfg.parse_field("refresh_rate_ms", "250");
    cfg.parse_field("enable_borderless", "true");
    cfg.parse_field("enable_toasts", "false");
    cfg.parse_field("enable_event_log", "false");

    assert_eq!(cfg.theme_mode, "dark");
    assert_eq!(cfg.refresh_rate_ms, 250);
    assert!(cfg.enable_borderless);
    assert!(!cfg.enable_toasts);
    assert!(!cfg.enable_event_log);
}

#[test]
fn test_serialize_fields() {
    let cfg = AppConfig {
        theme_mode: "light".to_string(),
        refresh_rate_ms: 50,
        enable_borderless: true,
        enable_toasts: false,
        enable_event_log: true,
    };

    let serialized = cfg.serialize_fields();
    let find_val = |k: &str| {
        serialized.iter()
            .find(|(key, _)| key == k)
            .map(|(_, val)| val.clone())
    };

    assert_eq!(find_val("theme_mode"), Some("light".to_string()));
    assert_eq!(find_val("refresh_rate_ms"), Some("50".to_string()));
    assert_eq!(find_val("enable_borderless"), Some("true".to_string()));
    assert_eq!(find_val("enable_toasts"), Some("false".to_string()));
    assert_eq!(find_val("enable_event_log"), Some("true".to_string()));
}
