use std::time::{Duration, Instant};
use ratatui::style::Color;
use ratatui::text::Line;

use crate::config;
use crate::logger::log_message;
use crate::startup;
use crate::win32;

pub mod keys;
pub mod mouse;

pub use keys::handle_key;
pub use mouse::handle_mouse;

#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub border: Color,
    pub border_active: Color,
    pub text_main: Color,
    pub text_dim: Color,
    pub accent: Color,
}

pub fn get_theme(dark: bool, accent_color: Color) -> ThemeColors {
    if dark {
        ThemeColors {
            border: Color::Rgb(68, 68, 84),
            border_active: accent_color,
            text_main: Color::Rgb(248, 248, 242),
            text_dim: Color::Rgb(136, 136, 153),
            accent: accent_color,
        }
    } else {
        ThemeColors {
            border: Color::Rgb(180, 180, 190),
            border_active: accent_color,
            text_main: Color::Rgb(40, 42, 54),
            text_dim: Color::Rgb(100, 100, 115),
            accent: accent_color,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FocusedSection {
    LeftPanel,
    RightPanel,
}

#[allow(dead_code)]
pub struct App {
    pub status_msg: String,
    pub status_timer: Option<Instant>,
    pub focus: FocusedSection,
    pub should_quit: bool,
    pub show_help: bool,
    pub glyphs: win32::GlyphMap,

    // Registry/Theme state variables
    pub dark_mode: bool,
    pub accent_color: Color,
    pub last_theme_check: Instant,

    pub enable_toasts: bool,

    // Interactive Startup Items selection
    pub selected_startup: usize,
    pub startup_items: Vec<startup::StartupItem>,
    pub theme_mode: String,
    pub refresh_rate_ms: u32,
    pub enable_borderless: bool,
    pub enable_event_log: bool,

    // Battery throttling status
    pub on_battery: bool,
    pub last_power_check: Instant,

    // Native Windows System diagnostics structures
    pub sys: sysinfo::System,
    pub networks: sysinfo::Networks,
    pub top_processes: Vec<(u32, String, f32, u64)>,
    pub network_rates: Vec<(String, u64, u64)>,
    pub last_metrics_refresh: Instant,

    // TUI Markdown Viewer modal status
    pub show_markdown: Option<String>,
    pub markdown_lines: Vec<Line<'static>>,
    pub markdown_scroll: usize,
    pub selection_start: Option<(u16, u16)>,
    pub selection_end: Option<(u16, u16)>,
    pub selection_pending_copy: bool,

    pub show_backups: bool,
    pub backup_db: startup::BackupDatabase,
    pub selected_backup: usize,
    pub quit_btn_bounds: Option<(u16, u16, u16)>,
    pub help_btn_bounds: Option<(u16, u16, u16)>,
    pub drag_active: bool,
    pub drag_start_cursor: Option<(i32, i32)>,
    pub drag_start_window: Option<(i32, i32)>,
    pub username: String,
    pub host_name: String,
    pub os_version: String,
}

impl App {
    pub fn new(config: &config::AppConfig) -> Self {
        let dark_mode = match config.theme_mode.as_str() {
            "dark" => true,
            "light" => false,
            _ => win32::query_dark_mode(),
        };
        let accent_color = win32::get_dwm_accent_color();
        let glyphs = win32::GlyphMap::load();
        let on_battery = if let Some(power) = win32::query_power_status() {
            !power.ac_online
        } else {
            false
        };
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        let networks = sysinfo::Networks::new_with_refreshed_list();
        let username = std::env::var("USERNAME")
            .unwrap_or_else(|_| std::env::var("USER").unwrap_or_else(|_| "user".to_string()));
        let host_name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "localhost".to_string());
        let os_version = win32::query_os_version();
        Self {
            status_msg:
                "Use arrow keys to browse startup entries. Press Space to toggle, Delete to remove. (h for help)"
                    .to_string(),
            status_timer: None,
            focus: FocusedSection::LeftPanel,
            should_quit: false,
            show_help: false,
            glyphs,
            dark_mode,
            accent_color,
            last_theme_check: Instant::now(),
            enable_toasts: config.enable_toasts,
            selected_startup: 0,
            startup_items: startup::scan_startup_items(),
            theme_mode: config.theme_mode.clone(),
            refresh_rate_ms: config.refresh_rate_ms,
            enable_borderless: config.enable_borderless,
            enable_event_log: config.enable_event_log,
            on_battery,
            last_power_check: Instant::now(),
            sys,
            networks,
            top_processes: Vec::new(),
            network_rates: Vec::new(),
            last_metrics_refresh: Instant::now() - Duration::from_secs(10), // Force immediate refresh
            show_markdown: None,
            markdown_lines: Vec::new(),
            markdown_scroll: 0,
            selection_start: None,
            selection_end: None,
            selection_pending_copy: false,
            show_backups: false,
            backup_db: startup::BackupDatabase::load(),
            selected_backup: 0,
            quit_btn_bounds: None,
            help_btn_bounds: None,
            drag_active: false,
            drag_start_cursor: None,
            drag_start_window: None,
            username,
            host_name,
            os_version,
        }
    }

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

    /// Refresh process list and network adapters data dynamically.
    pub fn refresh_system_metrics(&mut self) {
        if self.last_metrics_refresh.elapsed() > Duration::from_millis(1500) {
            self.last_metrics_refresh = Instant::now();
            self.sys.refresh_processes();
            self.networks.refresh();

            // Calculate top processes by CPU usage (normalized by logical CPU core count)
            let core_count = self.sys.cpus().len().max(1) as f32;
            let mut procs: Vec<_> = self
                .sys
                .processes()
                .values()
                .map(|p| {
                    (
                        p.pid().as_u32(),
                        p.name().to_string(),
                        p.cpu_usage() / core_count,
                        p.memory(),
                    )
                })
                .collect();
            procs.sort_by(|a, b| b.2.total_cmp(&a.2));
            procs.truncate(5);
            self.top_processes = procs;

            // Fetch network rates
            let mut rates = Vec::new();
            for (name, data) in &self.networks {
                rates.push((name.clone(), data.received(), data.transmitted()));
            }
            self.network_rates = rates;
        }
    }

    /// Parse and display an embedded markdown document for dynamic in-TUI modal rendering.
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

    /// Checks the Windows Registry for theme/color changes and syncs TUI in real-time.
    pub fn sync_theme_if_needed(&mut self, config: &config::AppConfig) {
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

    /// Checks system power status periodically and adjusts throttling state.
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

    /// Collect the raw text format of the currently selected diagnostic screen for clipboard storage.
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
