//! Windows startup entry scanner, toggle, and delete implementation.
//!
//! **Taxonomy Classification**: Platform (Startup / Windows Native).

use std::path::PathBuf;
use crate::backend::registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

#[derive(Debug, Clone)]
pub struct StartupItem {
    pub name: String,
    pub command: String,
    pub location_type: String, // "Registry (User)", "Registry (System)", "Registry (System 32-bit)", "Startup Folder (User)", "Startup Folder (System)"
    pub location_path: String,
    pub enabled: bool,
    pub key_name: String, // Name of value in registry or file name in folder
    pub impact: String, // "Low", "Medium", "High"
}

/// Helper to check if the binary data from StartupApproved indicates the item is enabled.
fn is_startup_approved_enabled(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return true;
    }
    // 0x02 at byte 0 means Enabled. 0x03 or other values mean Disabled.
    bytes[0] == 0x02
}

/// Resolve the current user's Startup folder path.
pub fn get_user_startup_dir() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    Some(PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Startup"))
}

/// Resolve the all-users Startup folder path.
pub fn get_system_startup_dir() -> Option<PathBuf> {
    let programdata = std::env::var("ProgramData")
        .ok()
        .or_else(|| std::env::var("ALLUSERSPROFILE").ok())
        .unwrap_or_else(|| "C:\\ProgramData".to_string());
    Some(PathBuf::from(programdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Startup"))
}

/// Scan all startup locations (Registry + Directories).
pub fn scan_startup_items() -> Vec<StartupItem> {
    let mut items = Vec::new();

    // 1. Registry: HKCU Run Key
    let hkcu_run_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    let hkcu_approved_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
    if let Some(values) = crate::backend::registry::list_values(HKEY_CURRENT_USER, hkcu_run_path) {
        for (name, command) in values {
            let enabled = crate::backend::registry::read_binary(HKEY_CURRENT_USER, hkcu_approved_path, &name)
                .map(|bytes| is_startup_approved_enabled(&bytes))
                .unwrap_or(true);
            let impact = estimate_startup_impact(&command);
            items.push(StartupItem {
                name: name.clone(),
                command,
                location_type: "Registry (User)".to_string(),
                location_path: format!("HKCU\\{}", hkcu_run_path),
                enabled,
                key_name: name,
                impact,
            });
        }
    }

    // 2. Registry: HKLM Run Key
    let hklm_run_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    let hklm_approved_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
    if let Some(values) = crate::backend::registry::list_values(HKEY_LOCAL_MACHINE, hklm_run_path) {
        for (name, command) in values {
            let enabled = crate::backend::registry::read_binary(HKEY_LOCAL_MACHINE, hklm_approved_path, &name)
                .map(|bytes| is_startup_approved_enabled(&bytes))
                .unwrap_or(true);
            let impact = estimate_startup_impact(&command);
            items.push(StartupItem {
                name: name.clone(),
                command,
                location_type: "Registry (System)".to_string(),
                location_path: format!("HKLM\\{}", hklm_run_path),
                enabled,
                key_name: name,
                impact,
            });
        }
    }

    // 3. Registry: HKLM WOW6432Node Run Key
    let wow_run_path = "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Run";
    let wow_approved_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run32";
    if let Some(values) = crate::backend::registry::list_values(HKEY_LOCAL_MACHINE, wow_run_path) {
        for (name, command) in values {
            let enabled = crate::backend::registry::read_binary(HKEY_LOCAL_MACHINE, wow_approved_path, &name)
                .map(|bytes| is_startup_approved_enabled(&bytes))
                .unwrap_or(true);
            let impact = estimate_startup_impact(&command);
            items.push(StartupItem {
                name: name.clone(),
                command,
                location_type: "Registry (System 32-bit)".to_string(),
                location_path: format!("HKLM\\{}", wow_run_path),
                enabled,
                key_name: name,
                impact,
            });
        }
    }

    // 4. Folder: User Startup Folder
    if let Some(dir) = get_user_startup_dir() {
        let approved_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\StartupFolder";
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                        if filename.to_lowercase() == "desktop.ini" {
                            continue;
                        }
                        let command = path.to_string_lossy().to_string();
                        let enabled = crate::backend::registry::read_binary(HKEY_CURRENT_USER, approved_path, filename)
                            .map(|bytes| is_startup_approved_enabled(&bytes))
                            .unwrap_or(true);
                        let impact = estimate_startup_impact(&command);
                        items.push(StartupItem {
                            name: filename.to_string(),
                            command,
                            location_type: "Startup Folder (User)".to_string(),
                            location_path: dir.to_string_lossy().to_string(),
                            enabled,
                            key_name: filename.to_string(),
                            impact,
                        });
                    }
            }
        }
    }

    // 5. Folder: System Startup Folder
    if let Some(dir) = get_system_startup_dir() {
        let approved_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\StartupFolder";
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                        if filename.to_lowercase() == "desktop.ini" {
                            continue;
                        }
                        let command = path.to_string_lossy().to_string();
                        let enabled = crate::backend::registry::read_binary(HKEY_LOCAL_MACHINE, approved_path, filename)
                            .map(|bytes| is_startup_approved_enabled(&bytes))
                            .unwrap_or(true);
                        let impact = estimate_startup_impact(&command);
                        items.push(StartupItem {
                            name: filename.to_string(),
                            command,
                            location_type: "Startup Folder (System)".to_string(),
                            location_path: dir.to_string_lossy().to_string(),
                            enabled,
                            key_name: filename.to_string(),
                            impact,
                        });
                    }
            }
        }
    }

    items
}

#[path = "win32_actions.rs"]
pub mod actions;
pub use actions::{toggle_startup_item, delete_startup_item, add_startup_item, estimate_startup_impact};
