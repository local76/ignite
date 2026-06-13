use std::time::{Duration, Instant};
use crate::app::{App, get_theme};
use crate::logger::log_message;
use crate::win32;

impl App {
    pub fn set_status(&mut self, msg: String) {
        self.status_msg = msg;
        self.status_timer = Some(Instant::now());
        log_message("INFO", &format!("Status updated: {}", self.status_msg));
    }

    pub fn select_next_startup(&mut self) {
        if self.startup_items.is_empty() {
            self.selected_startup = 0;
            return;
        }
        self.selected_startup = (self.selected_startup + 1) % self.startup_items.len();
        self.set_status(format!(
            "Selected item: {}",
            self.startup_items[self.selected_startup].name
        ));
    }

    pub fn select_prev_startup(&mut self) {
        if self.startup_items.is_empty() {
            self.selected_startup = 0;
            return;
        }
        if self.selected_startup == 0 {
            self.selected_startup = self.startup_items.len() - 1;
        } else {
            self.selected_startup -= 1;
        }
        self.set_status(format!(
            "Selected item: {}",
            self.startup_items[self.selected_startup].name
        ));
    }

    pub fn open_embedded_markdown(&mut self, title: &str, content: &str) {
        self.markdown_lines =
            crate::ui::parse_markdown_to_lines(content, &get_theme(self.dark_mode, self.accent_color));
        self.show_markdown = Some(title.to_string());
        self.markdown_scroll = 0;
        self.set_status(format!("Opened document: {}", title));
    }

    pub fn check_status_decay(&mut self) {
        if let Some(t) = self.status_timer {
            if t.elapsed() > Duration::from_secs(4) {
                self.status_msg = "Use arrow keys to browse startup entries. Press Space to toggle, Delete to remove. (h for help)".to_string();
                self.status_timer = None;
            }
        }
    }

    pub fn sync_theme_if_needed(&mut self, config: &crate::config::AppConfig) {
        if self.last_theme_check.elapsed() > Duration::from_millis(2500) {
            self.last_theme_check = Instant::now();
            let current_dark = match config.theme_mode.as_str() {
                "dark" => true,
                "light" => false,
                _ => win32::query_dark_mode(),
            };
            let current_accent = win32::get_dwm_accent_color();
            if current_dark != self.dark_mode || current_accent != self.accent_color {
                self.dark_mode = current_dark;
                self.accent_color = current_accent;
                log_message(
                    "THEME_SYNC",
                    &format!(
                        "Color theme updated. Dark Mode: {}, Accent: {:?}",
                        current_dark, current_accent
                    ),
                );
            }
        }
    }

    pub fn sync_power_status_if_needed(&mut self) {
        if self.last_power_check.elapsed() > Duration::from_millis(5000) {
            self.last_power_check = Instant::now();
            if let Some(power) = win32::query_power_status() {
                let current_on_battery = !power.ac_online;
                if current_on_battery != self.on_battery {
                    self.on_battery = current_on_battery;
                    let state = if current_on_battery {
                        "Battery (Power-Saving Throttling Enabled)"
                    } else {
                        "AC Power (Full Speed)"
                    };
                    log_message(
                        "POWER_SYNC",
                        &format!("Power source changed. Status: {}", state),
                    );
                    self.set_status(format!("Power Source Changed: {}", state));
                }
            }
        }
    }

    pub fn get_diagnostic_details_text(&self) -> String {
        let mut details = String::new();
        if self.startup_items.is_empty() {
            details.push_str("No startup items detected.\n");
            return details;
        }
        if let Some(item) = self.startup_items.get(self.selected_startup) {
            details.push_str("--- Startup Application Specifications ---\n");
            details.push_str(&format!("Name:          {}\n", item.name));
            details.push_str(&format!("Command:       {}\n", item.command));
            details.push_str(&format!("Status:        {}\n", if item.enabled { "Enabled" } else { "Disabled" }));
            details.push_str(&format!("Location Type: {}\n", item.location_type));
            details.push_str(&format!("Location Path: {}\n", item.location_path));
            details.push_str(&format!("Config Key:    {}\n", item.key_name));
            details.push_str(&format!("Startup Impact: {}\n", item.impact));
        }
        details
    }
}
