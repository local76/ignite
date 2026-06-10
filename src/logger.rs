use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

static EVENT_LOG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Enable or disable Windows Event Log syncing globally.
pub fn set_event_log_enabled(enabled: bool) {
    EVENT_LOG_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Check if Windows Event Log syncing is globally enabled.
pub fn is_event_log_enabled() -> bool {
    EVENT_LOG_ENABLED.load(Ordering::Relaxed)
}

/// Helper to resolve the per-app log file path.
/// Windows: `%APPDATA%\ignite\log.txt`
/// Linux / macOS: `$XDG_DATA_HOME/ignite/log.txt` (falls back to `~/.local/share/ignite/log.txt`)
pub fn get_appdata_log_path() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        let appdata = std::env::var("APPDATA").ok()?;
        Some(PathBuf::from(appdata).join("local76").join("app").join("ignite").join("log.txt"))
    } else {
        // Linux / macOS XDG_DATA_HOME fallback
        let base = std::env::var("XDG_DATA_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".local").join("share"))
            })
            .unwrap_or_else(|| PathBuf::from(".local/share"));
        Some(base.join("local76").join("app").join("ignite").join("log.txt"))
    }
}

/// Thread-safe silent logger helper that appends diagnostic logs to a local file.
pub fn log_message(level: &str, msg: &str) {
    if let Some(path) = get_appdata_log_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let epoch = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let _ = writeln!(file, "[{}] [{}] {}", epoch, level, msg);
        }
    }

    if is_event_log_enabled() {
        let event_type = match level {
            "ERROR" | "PANIC" => 0x0001, // EVENTLOG_ERROR_TYPE
            "WARNING" => 0x0002,         // EVENTLOG_WARNING_TYPE
            _ => 0x0004,                 // EVENTLOG_INFORMATION_TYPE
        };
        library::event_log::log_system_event("ignite", event_type, 1000, msg);
    }
}
