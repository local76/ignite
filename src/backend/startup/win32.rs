//! Windows startup entry scanner, toggle, and delete implementation.
//!
//! **Taxonomy Classification**: Platform (Startup / Windows Native).

use std::path::PathBuf;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

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
    if let Some(values) = library::reg::list_values(HKEY_CURRENT_USER, hkcu_run_path) {
        for (name, command) in values {
            let enabled = library::reg::read_binary(HKEY_CURRENT_USER, hkcu_approved_path, &name)
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
    if let Some(values) = library::reg::list_values(HKEY_LOCAL_MACHINE, hklm_run_path) {
        for (name, command) in values {
            let enabled = library::reg::read_binary(HKEY_LOCAL_MACHINE, hklm_approved_path, &name)
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
    if let Some(values) = library::reg::list_values(HKEY_LOCAL_MACHINE, wow_run_path) {
        for (name, command) in values {
            let enabled = library::reg::read_binary(HKEY_LOCAL_MACHINE, wow_approved_path, &name)
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
                        let enabled = library::reg::read_binary(HKEY_CURRENT_USER, approved_path, filename)
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
                        let enabled = library::reg::read_binary(HKEY_LOCAL_MACHINE, approved_path, filename)
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

/// Toggle enabled/disabled status of a startup item.
pub fn toggle_startup_item(item: &mut StartupItem) -> std::io::Result<()> {
    let enabled = !item.enabled;
    let new_byte: u8 = if enabled { 0x02 } else { 0x03 };

    // Standard Windows Task Manager 12-byte status block
    let mut val = vec![0x00; 12];
    val[0] = new_byte;

    match item.location_type.as_str() {
        "Registry (User)" => {
            let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
            library::reg::write_binary(HKEY_CURRENT_USER, path, &item.key_name, &val)?;
        }
        "Registry (System)" => {
            let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
            library::reg::write_binary(HKEY_LOCAL_MACHINE, path, &item.key_name, &val)?;
        }
        "Registry (System 32-bit)" => {
            let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run32";
            library::reg::write_binary(HKEY_LOCAL_MACHINE, path, &item.key_name, &val)?;
        }
        "Startup Folder (User)" => {
            let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\StartupFolder";
            library::reg::write_binary(HKEY_CURRENT_USER, path, &item.key_name, &val)?;
        }
        "Startup Folder (System)" => {
            let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\StartupFolder";
            library::reg::write_binary(HKEY_LOCAL_MACHINE, path, &item.key_name, &val)?;
        }
        _ => {}
    }
    item.enabled = enabled;
    Ok(())
}

/// Delete a startup item config completely.
pub fn delete_startup_item(item: &StartupItem) -> std::io::Result<()> {
    match item.location_type.as_str() {
        "Registry (User)" => {
            let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
            let root = winreg::RegKey::predef(HKEY_CURRENT_USER);
            let subkey = root.open_subkey_with_flags(path, winreg::enums::KEY_WRITE)?;
            subkey.delete_value(&item.key_name)?;

            let app_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
            if let Ok(app_key) = root.open_subkey_with_flags(app_path, winreg::enums::KEY_WRITE) {
                let _ = app_key.delete_value(&item.key_name);
            }
        }
        "Registry (System)" => {
            let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
            let root = winreg::RegKey::predef(HKEY_LOCAL_MACHINE);
            let subkey = root.open_subkey_with_flags(path, winreg::enums::KEY_WRITE)?;
            subkey.delete_value(&item.key_name)?;

            let app_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
            if let Ok(app_key) = root.open_subkey_with_flags(app_path, winreg::enums::KEY_WRITE) {
                let _ = app_key.delete_value(&item.key_name);
            }
        }
        "Registry (System 32-bit)" => {
            let path = "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Run";
            let root = winreg::RegKey::predef(HKEY_LOCAL_MACHINE);
            let subkey = root.open_subkey_with_flags(path, winreg::enums::KEY_WRITE)?;
            subkey.delete_value(&item.key_name)?;

            let app_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run32";
            if let Ok(app_key) = root.open_subkey_with_flags(app_path, winreg::enums::KEY_WRITE) {
                let _ = app_key.delete_value(&item.key_name);
            }
        }
        "Startup Folder (User)" => {
            if let Some(mut dir) = get_user_startup_dir() {
                dir.push(&item.key_name);
                if dir.exists() {
                    std::fs::remove_file(dir)?;
                }
            }
            let root = winreg::RegKey::predef(HKEY_CURRENT_USER);
            let app_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\StartupFolder";
            if let Ok(app_key) = root.open_subkey_with_flags(app_path, winreg::enums::KEY_WRITE) {
                let _ = app_key.delete_value(&item.key_name);
            }
        }
        "Startup Folder (System)" => {
            if let Some(mut dir) = get_system_startup_dir() {
                dir.push(&item.key_name);
                if dir.exists() {
                    std::fs::remove_file(dir)?;
                }
            }
            let root = winreg::RegKey::predef(HKEY_LOCAL_MACHINE);
            let app_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\StartupFolder";
            if let Ok(app_key) = root.open_subkey_with_flags(app_path, winreg::enums::KEY_WRITE) {
                let _ = app_key.delete_value(&item.key_name);
            }
        }
        _ => {}
    }
    Ok(())
}

/// Create a new user registry startup item.
#[allow(dead_code)]
pub fn add_startup_item(name: &str, command: &str) -> std::io::Result<()> {
    let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    library::reg::write_string(HKEY_CURRENT_USER, path, name, command)?;

    let app_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
    let val = vec![0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    library::reg::write_binary(HKEY_CURRENT_USER, app_path, name, &val)?;

    Ok(())
}

/// Helper to parse clean executable path from command line
fn parse_exe_path(command: &str) -> Option<std::path::PathBuf> {
    let mut cmd = command.trim();

    // Strip quotes if present
    if cmd.starts_with('"') {
        if let Some(end_idx) = cmd[1..].find('"') {
            cmd = &cmd[1..end_idx + 1];
        }
    } else {
        // Find the first space that is not followed by an argument or is part of a path.
        let parts: Vec<&str> = cmd.split(' ').collect();
        let mut resolved_path = None;
        let mut current_prefix = String::new();
        for part in parts {
            if !current_prefix.is_empty() {
                current_prefix.push(' ');
            }
            current_prefix.push_str(part);
            let path_test = std::path::Path::new(&current_prefix);
            if path_test.exists() && path_test.is_file() {
                resolved_path = Some(path_test.to_path_buf());
                break;
            }
        }
        if let Some(path) = resolved_path {
            return Some(path);
        }

        // Fallback to splitting by space
        if let Some(space_idx) = cmd.find(' ') {
            cmd = &cmd[..space_idx];
        }
    }

    // Expand environment variables
    let mut resolved = cmd.to_string();
    if resolved.contains('%') {
        let mut expanded = String::new();
        let mut parts = resolved.split('%');
        if let Some(first) = parts.next() {
            expanded.push_str(first);
        }
        while let Some(var_name) = parts.next() {
            if let Ok(var_val) = std::env::var(var_name) {
                expanded.push_str(&var_val);
            } else {
                expanded.push('%');
                expanded.push_str(var_name);
                expanded.push('%');
            }
            if let Some(rest) = parts.next() {
                expanded.push_str(rest);
            }
        }
        resolved = expanded;
    }

    let path = std::path::PathBuf::from(resolved);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Heuristically estimate boot performance startup impact
pub fn estimate_startup_impact(command: &str) -> String {
    let path_opt = parse_exe_path(command);
    let Some(path) = path_opt else {
        // Check for common names in command string if file path not found directly
        let cmd_lower = command.to_lowercase();
        if cmd_lower.contains("steam") || cmd_lower.contains("discord") || cmd_lower.contains("vesktop") || cmd_lower.contains("razer") {
            return "High".to_string();
        }
        if cmd_lower.contains("tailscale") || cmd_lower.contains("synology") {
            return "Medium".to_string();
        }
        return "Low".to_string();
    };

    // 1. Check file size
    if let Ok(metadata) = std::fs::metadata(&path) {
        let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        if size_mb > 50.0 {
            return "High".to_string();
        } else if size_mb > 15.0 {
            return "Medium".to_string();
        }
    }

    // 2. Classify based on location/known directories
    let path_str = path.to_string_lossy().to_lowercase();
    if path_str.contains("steam") || path_str.contains("discord") || path_str.contains("vesktop") || path_str.contains("razer") {
        return "High".to_string();
    }
    if path_str.contains("tailscale") || path_str.contains("synology") {
        return "Medium".to_string();
    }
    if path_str.contains("system32") || path_str.contains("windows defender") {
        return "Low".to_string();
    }

    "Low".to_string()
}
