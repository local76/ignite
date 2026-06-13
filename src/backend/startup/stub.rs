#![allow(dead_code)]
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct StartupItem {
    pub name: String,
    pub command: String,
    pub location_type: String,
    pub location_path: String,
    pub enabled: bool,
    pub key_name: String,
    pub impact: String,
}

#[derive(Debug, Clone)]
pub struct BackupEntry {
    pub uuid: String,
    pub timestamp: String,
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

    pub fn add_item(&mut self, _item: &StartupItem) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn get_user_startup_dir() -> Option<PathBuf> {
    None
}

pub fn get_system_startup_dir() -> Option<PathBuf> {
    None
}

pub fn scan_startup_items() -> Vec<StartupItem> {
    vec![]
}

pub fn toggle_startup_item(_item: &mut StartupItem) -> std::io::Result<()> {
    Ok(())
}

pub fn delete_startup_item(_item: &StartupItem) -> std::io::Result<()> {
    Ok(())
}

pub fn add_startup_item(_name: &str, _command: &str) -> std::io::Result<()> {
    Ok(())
}

pub fn estimate_startup_impact(_command: &str) -> String {
    "Low".to_string()
}

pub fn restore_startup_item(_entry: &BackupEntry) -> std::io::Result<()> {
    Ok(())
}
