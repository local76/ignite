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
        None
    }
    pub fn load() -> Self {
        Self::default()
    }
    pub fn save(&self) -> std::io::Result<()> {
        Ok(())
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
