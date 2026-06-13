use std::path::PathBuf;
use crate::backend::registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_WRITE, RegKey};
use crate::backend::startup::win32::{StartupItem, get_user_startup_dir, get_system_startup_dir};

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
            crate::backend::registry::write_binary(HKEY_CURRENT_USER, path, &item.key_name, &val)?;
        }
        "Registry (System)" => {
            let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
            crate::backend::registry::write_binary(HKEY_LOCAL_MACHINE, path, &item.key_name, &val)?;
        }
        "Registry (System 32-bit)" => {
            let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run32";
            crate::backend::registry::write_binary(HKEY_LOCAL_MACHINE, path, &item.key_name, &val)?;
        }
        "Startup Folder (User)" => {
            let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\StartupFolder";
            crate::backend::registry::write_binary(HKEY_CURRENT_USER, path, &item.key_name, &val)?;
        }
        "Startup Folder (System)" => {
            let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\StartupFolder";
            crate::backend::registry::write_binary(HKEY_LOCAL_MACHINE, path, &item.key_name, &val)?;
        }
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("unknown startup location_type: {}", item.location_type),
            ));
        }
    }
    item.enabled = enabled;
    Ok(())
}

/// Delete a startup item config completely.
pub fn delete_startup_item(item: &StartupItem) -> std::io::Result<()> {
    match item.location_type.as_str() {
        "Registry (User)" => {
            let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
            let root = RegKey::predef(HKEY_CURRENT_USER);
            let subkey = root.open_subkey_with_flags(path, KEY_WRITE)?;
            subkey.delete_value(&item.key_name)?;

            let app_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
            if let Ok(app_key) = root.open_subkey_with_flags(app_path, KEY_WRITE) {
                let _ = app_key.delete_value(&item.key_name);
            }
        }
        "Registry (System)" => {
            let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
            let root = RegKey::predef(HKEY_LOCAL_MACHINE);
            let subkey = root.open_subkey_with_flags(path, KEY_WRITE)?;
            subkey.delete_value(&item.key_name)?;

            let app_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
            if let Ok(app_key) = root.open_subkey_with_flags(app_path, KEY_WRITE) {
                let _ = app_key.delete_value(&item.key_name);
            }
        }
        "Registry (System 32-bit)" => {
            let path = "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Run";
            let root = RegKey::predef(HKEY_LOCAL_MACHINE);
            let subkey = root.open_subkey_with_flags(path, KEY_WRITE)?;
            subkey.delete_value(&item.key_name)?;

            let app_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run32";
            if let Ok(app_key) = root.open_subkey_with_flags(app_path, KEY_WRITE) {
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
            let root = RegKey::predef(HKEY_CURRENT_USER);
            let app_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\StartupFolder";
            if let Ok(app_key) = root.open_subkey_with_flags(app_path, KEY_WRITE) {
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
            let root = RegKey::predef(HKEY_LOCAL_MACHINE);
            let app_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\StartupFolder";
            if let Ok(app_key) = root.open_subkey_with_flags(app_path, KEY_WRITE) {
                let _ = app_key.delete_value(&item.key_name);
            }
        }
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("unknown startup location_type: {}", item.location_type),
            ));
        }
    }
    Ok(())
}

/// Create a new user registry startup item.
#[allow(dead_code)]
pub fn add_startup_item(name: &str, command: &str) -> std::io::Result<()> {
    let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
    crate::backend::registry::write_string(HKEY_CURRENT_USER, path, name, command)?;

    let app_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
    let val = vec![0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    crate::backend::registry::write_binary(HKEY_CURRENT_USER, app_path, name, &val)?;

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

/// File size thresholds for startup impact classification (in MB)
const HIGH_IMPACT_MB: f64 = 50.0;
const MEDIUM_IMPACT_MB: f64 = 15.0;
/// Bytes per MB (1024 * 1024)
const BYTES_PER_MB: f64 = 1_048_576.0;

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
        let size_mb = metadata.len() as f64 / BYTES_PER_MB;
        if size_mb > HIGH_IMPACT_MB {
            return "High".to_string();
        } else if size_mb > MEDIUM_IMPACT_MB {
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
