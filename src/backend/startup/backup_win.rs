//! Windows startup entry backup database and restore logic.
//!
//! **Taxonomy Classification**: Platform (Startup / Windows Backup).

use std::path::PathBuf;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use crate::backend::{StartupItem, get_user_startup_dir, get_system_startup_dir};

#[allow(non_snake_case, clippy::upper_case_acronyms)]
#[repr(C)]
struct SYSTEMTIME {
    wYear: u16,
    wMonth: u16,
    wDayOfWeek: u16,
    wDay: u16,
    wHour: u16,
    wMinute: u16,
    wSecond: u16,
    wMilliseconds: u16,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetLocalTime(lp_system_time: *mut SYSTEMTIME);
}

fn get_rfc3339_timestamp() -> String {
    let mut st: SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe { GetLocalTime(&mut st); }
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}",
        st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond, st.wMilliseconds
    )
}

#[derive(Debug, Clone)]
pub struct BackupEntry {
    pub uuid: String,
    pub timestamp: String, // ISO 8601 string
    pub name: String,
    pub command: String,
    pub location_type: String,
    pub location_path: String,
    pub key_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct BackupDatabase {
    pub entries: Vec<BackupEntry>,
}

impl BackupDatabase {
    pub fn file_path() -> Option<PathBuf> {
        std::env::var("APPDATA").ok().map(|appdata| {
            PathBuf::from(appdata)
                .join("local76")
                .join("app")
                .join("ignite")
                .join("backups.yaml")
        })
    }

    pub fn load() -> Self {
        let Some(path) = Self::file_path() else { return Self::default(); };
        if !path.exists() { return Self::default(); }
        
        let Ok(content) = std::fs::read_to_string(&path) else {
            return Self::default();
        };

        let mut entries = Vec::new();
        let mut current_entry: Option<BackupEntry> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(pos) = line.find(':') {
                let key = line[..pos].trim();
                let val = line[pos + 1..].trim();
                match key {
                    "uuid" => {
                        if let Some(entry) = current_entry.take() {
                            entries.push(entry);
                        }
                        current_entry = Some(BackupEntry {
                            uuid: val.to_string(),
                            timestamp: String::new(),
                            name: String::new(),
                            command: String::new(),
                            location_type: String::new(),
                            location_path: String::new(),
                            key_name: String::new(),
                        });
                    }
                    "timestamp" => {
                        if let Some(ref mut entry) = current_entry {
                            entry.timestamp = val.to_string();
                        }
                    }
                    "name" => {
                        if let Some(ref mut entry) = current_entry {
                            entry.name = val.to_string();
                        }
                    }
                    "command" => {
                        if let Some(ref mut entry) = current_entry {
                            entry.command = val.to_string();
                        }
                    }
                    "location_type" => {
                        if let Some(ref mut entry) = current_entry {
                            entry.location_type = val.to_string();
                        }
                    }
                    "location_path" => {
                        if let Some(ref mut entry) = current_entry {
                            entry.location_path = val.to_string();
                        }
                    }
                    "key_name" => {
                        if let Some(ref mut entry) = current_entry {
                            entry.key_name = val.to_string();
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(entry) = current_entry {
            entries.push(entry);
        }

        BackupDatabase { entries }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let Some(path) = Self::file_path() else {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "AppData not found"));
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut content = String::new();
        content.push_str("# ignite Backups Database\n# ---------------------------\n\n");
        for entry in &self.entries {
            content.push_str(&format!(
                "uuid: {}\n\
                 timestamp: {}\n\
                 name: {}\n\
                 command: {}\n\
                 location_type: {}\n\
                 location_path: {}\n\
                 key_name: {}\n\n",
                entry.uuid,
                entry.timestamp,
                entry.name,
                entry.command,
                entry.location_type,
                entry.location_path,
                entry.key_name,
            ));
        }
        crate::backend::config::write_file_atomic(path, content)
    }

    pub fn add_item(&mut self, item: &StartupItem) -> std::io::Result<()> {
        let timestamp_now = get_rfc3339_timestamp();
        let epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let simple_id = format!("{}-{}", item.name, epoch);
        let entry = BackupEntry {
            uuid: simple_id,
            timestamp: timestamp_now,
            name: item.name.clone(),
            command: item.command.clone(),
            location_type: item.location_type.clone(),
            location_path: item.location_path.clone(),
            key_name: item.key_name.clone(),
        };
        self.entries.push(entry);
        self.save()
    }
}

pub fn restore_startup_item(entry: &BackupEntry) -> std::io::Result<()> {
    match entry.location_type.as_str() {
        "Registry (User)" => {
            let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
            crate::backend::registry::write_string(HKEY_CURRENT_USER, path, &entry.key_name, &entry.command)?;
            
            // Re-create enabled startup approved binary value
            let app_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
            let val = vec![0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
            crate::backend::registry::write_binary(HKEY_CURRENT_USER, app_path, &entry.key_name, &val)?;
        }
        "Registry (System)" => {
            let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
            crate::backend::registry::write_string(HKEY_LOCAL_MACHINE, path, &entry.key_name, &entry.command)?;
            
            let app_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
            let val = vec![0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
            crate::backend::registry::write_binary(HKEY_LOCAL_MACHINE, app_path, &entry.key_name, &val)?;
        }
        "Registry (System 32-bit)" => {
            let path = "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Run";
            crate::backend::registry::write_string(HKEY_LOCAL_MACHINE, path, &entry.key_name, &entry.command)?;
            
            let app_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run32";
            let val = vec![0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
            crate::backend::registry::write_binary(HKEY_LOCAL_MACHINE, app_path, &entry.key_name, &val)?;
        }
        "Startup Folder (User)" => {
            if let Some(mut dir) = get_user_startup_dir() {
                dir.push(&entry.key_name);
                let cmd_str = format!("@echo off\nstart \"\" \"{}\"", entry.command);
                let mut path = dir.clone();
                if path.extension().is_some_and(|ext| ext == "lnk") {
                    path.set_extension("bat");
                }
                std::fs::write(&path, cmd_str)?;
            }
        }
        "Startup Folder (System)" => {
            if let Some(mut dir) = get_system_startup_dir() {
                dir.push(&entry.key_name);
                let cmd_str = format!("@echo off\nstart \"\" \"{}\"", entry.command);
                let mut path = dir.clone();
                if path.extension().is_some_and(|ext| ext == "lnk") {
                    path.set_extension("bat");
                }
                std::fs::write(&path, cmd_str)?;
            }
        }
        _ => {}
    }
    Ok(())
}
