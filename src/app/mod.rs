use std::time::{Duration, Instant};
use ratatui::style::Color;
use ratatui::text::Line;

use crate::config;
use crate::backend;
use crate::win32;

pub mod keys;
pub mod mouse;
mod actions;

pub use keys::handle_key;
pub use mouse::handle_mouse;

pub use crate::ui::theme::{ThemeColors, get_theme};

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
    pub startup_items: Vec<backend::StartupItem>,
    pub theme_mode: String,
    pub refresh_rate_ms: u32,
    pub enable_borderless: bool,
    pub enable_event_log: bool,

    // Battery throttling status
    pub on_battery: bool,
    pub last_power_check: Instant,

    // Native Windows System diagnostics structures
    pub sys: crate::backend::sysinfo_shim::System,
    pub networks: crate::backend::sysinfo_shim::Networks,
    pub top_processes: Vec<(u32, String, f32, u64)>,
    pub network_rates: Vec<(String, u64, u64)>,
    pub last_metrics_refresh: Instant,

    // Console Markdown Viewer modal status
    pub show_markdown: Option<String>,
    pub markdown_lines: Vec<Line<'static>>,
    pub markdown_scroll: usize,
    pub selection_start: Option<(u16, u16)>,
    pub selection_end: Option<(u16, u16)>,
    pub selection_pending_copy: bool,

    pub show_backups: bool,
    pub backup_db: backend::BackupDatabase,
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
        let mut sys = crate::backend::sysinfo_shim::System::new_all();
        sys.refresh_all();
        let networks = crate::backend::sysinfo_shim::Networks::new_with_refreshed_list();
        let username = crate::backend::identity::username();
        let host_name = crate::backend::identity::hostname();
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
            startup_items: backend::scan_startup_items(),
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
            backup_db: backend::BackupDatabase::load(),
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
}
