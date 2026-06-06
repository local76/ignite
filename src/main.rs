use std::{
    io,
    sync::mpsc::{Receiver, channel},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

mod config;
mod input;
mod logger;
mod reg;
mod widgets;
mod win32;
mod worker;

use input::TextBox;
use logger::log_message;
use widgets::{AccentGauge, AccentList};
use win32::{BorderlessConsole, ConsoleTitleGuard, SingleInstanceGuard};
use worker::WorkerEvent;

// ==========================================
// 1. Theme Configuration
// ==========================================

#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub border: Color,
    pub border_active: Color,
    pub text_main: Color,
    pub text_dim: Color,
    pub accent: Color,
}

fn get_theme(dark: bool, accent_color: Color) -> ThemeColors {
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

/// A lightweight, custom terminal markdown parser returning styled TUI Spans and Lines.
fn parse_markdown_to_lines(content: &str, theme: &ThemeColors) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut current_paragraph = String::new();

    // Helper closure to flush the accumulated paragraph text to a single TUI line.
    let flush_paragraph = |para: &mut String, lines: &mut Vec<Line<'static>>| {
        if !para.is_empty() {
            lines.push(Line::from(Span::styled(
                para.clone(),
                Style::default().fg(theme.text_main),
            )));
            para.clear();
        }
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(Color::Rgb(150, 240, 150)),
            )));
            continue;
        }

        if trimmed.is_empty() {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(""));
            continue;
        }

        if trimmed.starts_with("# ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            let header = trimmed[2..].to_string();
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("=== {} ===", header.to_uppercase()),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        } else if trimmed.starts_with("## ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            let header = trimmed[3..].to_string();
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("--- {} ---", header),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        } else if trimmed.starts_with("### ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            let header = trimmed[4..].to_string();
            lines.push(Line::from(Span::styled(
                header,
                Style::default().fg(theme.accent),
            )));
        } else if trimmed.starts_with("* ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            let item = trimmed[2..].to_string();
            lines.push(Line::from(vec![
                Span::styled(" • ", Style::default().fg(theme.accent)),
                Span::styled(item, Style::default().fg(theme.text_main)),
            ]));
        } else if trimmed.starts_with("- ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            let item = trimmed[2..].to_string();
            lines.push(Line::from(vec![
                Span::styled(" • ", Style::default().fg(theme.accent)),
                Span::styled(item, Style::default().fg(theme.text_main)),
            ]));
        } else if trimmed.starts_with("1. ")
            || trimmed.starts_with("2. ")
            || trimmed.starts_with("3. ")
            || trimmed.starts_with("4. ")
            || trimmed.starts_with("5. ")
        {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} ", &trimmed[..3]),
                    Style::default().fg(theme.accent),
                ),
                Span::styled(
                    trimmed[3..].to_string(),
                    Style::default().fg(theme.text_main),
                ),
            ]));
        } else if trimmed.starts_with("> ") {
            flush_paragraph(&mut current_paragraph, &mut lines);
            lines.push(Line::from(Span::styled(
                format!("  │ {}", &trimmed[2..]),
                Style::default()
                    .fg(theme.text_dim)
                    .add_modifier(Modifier::ITALIC),
            )));
        } else {
            // Append standard lines to the current paragraph block.
            if !current_paragraph.is_empty() {
                current_paragraph.push(' ');
            }
            current_paragraph.push_str(trimmed);
        }
    }
    flush_paragraph(&mut current_paragraph, &mut lines);
    lines
}

// ==========================================
// 2. CLI Command Handling & Diagnostics
// ==========================================

pub const RSTARTUP_LOGO: &str = r"
         _____ __                __               
   _____/ ___// /_____ _________/ /___  ______    
  / ___/\__ \/ __/ __ `/ ___/ __  / / / / __ \   
 / /   ___/ / /_/ /_/ / /  / /_/ / /_/ / /_/ /   
/_/   /____/\__/\__,_/_/   \__,_/\__,_/ .___/    
                                     /_/         
";

const README_CONTENT: &str = include_str!("../README.md");
const SUPPORT_CONTENT: &str = include_str!("../SUPPORT.md");
const LICENSE_CONTENT: &str = include_str!("../LICENSE.md");
const COPYRIGHT_CONTENT: &str = include_str!("../COPYRIGHT.md");
const PRIVACY_CONTENT: &str = include_str!("../PRIVACY.md");
const SECURITY_CONTENT: &str = include_str!("../SECURITY.md");
const CONTRIBUTING_CONTENT: &str = include_str!("../CONTRIBUTING.md");

fn print_help() {
    println!("{}", RSTARTUP_LOGO);
    println!(
        "rsta — Rust Startup Manager (v{})",
        env!("CARGO_PKG_VERSION")
    );
    println!("Usage:");
    println!("  rsta.exe [command]");
    println!();
    println!("Commands:");
    println!("  tui       Launch the interactive TUI dashboard (default)");
    println!("  doctor    Verify system registry, log paths, and console scaling");
    println!("  version   Print application version info");
    println!("  help      Print this help message");
}

/// Collect the raw text format of the currently selected diagnostic screen for clipboard storage.
fn get_diagnostic_details_text(app: &App) -> String {
    let mut details = String::new();
    match app.selected_diagnostic {
        0 => {
            details.push_str("--- BIOS Specifications ---\n");
            if let Some(bios) = win32::query_bios_info() {
                details.push_str(&format!("Manufacturer: {}\n", bios.manufacturer));
                details.push_str(&format!("Product:      {}\n", bios.product));
                details.push_str(&format!("Model/Board:  {}\n", bios.model));
            } else {
                details.push_str("No BIOS information detected.\n");
            }
        }
        1 => {
            details.push_str("--- Power & Battery Life ---\n");
            if let Some(power) = win32::query_power_status() {
                let source = if power.ac_online {
                    "AC Online"
                } else {
                    "Battery (Discharging)"
                };
                let pct = if power.battery_percent == 255 {
                    "Unknown".to_string()
                } else {
                    format!("{}%", power.battery_percent)
                };
                details.push_str(&format!("Power Source:  {}\n", source));
                details.push_str(&format!("Charge Level:  {}\n", pct));
            } else {
                details.push_str("No battery/power status detected.\n");
            }
        }
        2 => {
            details.push_str("--- Parent Environment ---\n");
            let (shell, term) = win32::query_shell_and_terminal();
            details.push_str(&format!("Active Shell:  {}\n", shell));
            details.push_str(&format!("Terminal Host: {}\n", term));
        }
        3 => {
            details.push_str("--- Monitor & Resolution ---\n");
            let (w, h) = win32::get_system_screen_resolution();
            let dpi = win32::get_console_window_dpi();
            details.push_str(&format!("Screen Size:   {}x{}\n", w, h));
            details.push_str(&format!(
                "Console DPI:   {}% ({} DPI)\n",
                (dpi as f32 / 96.0 * 100.0) as u32,
                dpi
            ));
        }
        4 => {
            details.push_str("--- Active Configuration ---\n");
            details.push_str(&format!("Theme Mode:    {}\n", app.theme_mode));
            details.push_str(&format!("Refresh Rate:  {}ms\n", app.refresh_rate_ms));
            let current_rate = if app.on_battery {
                app.refresh_rate_ms * 2
            } else {
                app.refresh_rate_ms
            };
            let throttle_status = if app.on_battery {
                " (Throttling Active)"
            } else {
                " (Full Speed)"
            };
            details.push_str(&format!(
                "Active Tick:   {}ms{}\n",
                current_rate, throttle_status
            ));
            details.push_str(&format!("Borderless:    {}\n", app.enable_borderless));
            details.push_str(&format!("Toast Alerts:  {}\n", app.enable_toasts));
            details.push_str(&format!("Event Log:     {}\n", app.enable_event_log));
        }
        5 => {
            details.push_str("--- Top 5 CPU Processes ---\n");
            details.push_str(&format!(
                "{:>6}  {:<20}  {:>8}  {:>10}\n",
                "PID", "Name", "CPU %", "Memory"
            ));
            for (pid, name, cpu, mem) in &app.top_processes {
                let mem_mb = *mem as f64 / 1024.0 / 1024.0;
                details.push_str(&format!(
                    "{:>6}  {:<20}  {:>7.1}%  {:>8.1} MB\n",
                    pid, name, cpu, mem_mb
                ));
            }
        }
        6 => {
            details.push_str("--- Logical Drive Storage ---\n");
            let drives = win32::query_disk_drives();
            for drive in drives {
                let total_gb = drive.total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
                let free_gb = drive.free_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
                let used_gb = total_gb - free_gb;
                let pct = if total_gb > 0.0 {
                    (used_gb / total_gb) * 100.0
                } else {
                    0.0
                };
                details.push_str(&format!(
                    "{} : {:.1}% used ({:.1} GB free / {:.1} GB total)\n",
                    drive.path, pct, free_gb, total_gb
                ));
            }
        }
        7 => {
            details.push_str("--- Network Adapter Metrics ---\n");
            if let Some(ip) = win32::query_local_ip() {
                details.push_str(&format!("Local IP Address:  {}\n", ip));
            }
            details.push_str(&format!(
                "{:<20}  {:>12}  {:>12}\n",
                "Interface", "Rx Total", "Tx Total"
            ));
            for (name, rx, tx) in &app.network_rates {
                let rx_mb = *rx as f64 / 1024.0 / 1024.0;
                let tx_mb = *tx as f64 / 1024.0 / 1024.0;
                details.push_str(&format!(
                    "{:<20}  {:>9.2} MB  {:>9.2} MB\n",
                    name, rx_mb, tx_mb
                ));
            }
        }
        8 => {
            details.push_str("--- Windows Services Status ---\n");
            let services = vec![
                ("wuauserv", "Windows Update"),
                ("Spooler", "Print Spooler"),
                ("EventLog", "Windows Event Log"),
                ("Dhcp", "DHCP Client"),
                ("Dnscache", "DNS Client"),
            ];
            for (service_id, display_name) in services {
                let status = win32::query_windows_service_status(service_id);
                details.push_str(&format!("{:<18} : {}\n", display_name, status));
            }
        }
        _ => {}
    }
    details
}

fn run_doctor() {
    println!("{}", RSTARTUP_LOGO);
    println!("rStartup Doctor — Diagnostic Report");
    println!("====================================");

    // 1. Check OS Version
    let os = win32::query_os_version();
    println!("Host OS:                  {}", os);

    // 2. Check Registry Access
    print!("Registry HKCU Access:     ");
    #[cfg(windows)]
    {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        match hkcu.open_subkey("Control Panel\\Desktop") {
            Ok(_) => println!("OK (Readable)"),
            Err(e) => println!("FAILED (Error: {})", e),
        }
    }
    #[cfg(not(windows))]
    println!("N/A (Not on Windows)");

    // 3. Check Log File Writable
    print!("Log Path Writable:        ");
    if let Some(path) = logger::get_appdata_log_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            Ok(_) => println!("OK ({:?})", path),
            Err(e) => println!("FAILED (Error: {})", e),
        }
    } else {
        println!("FAILED (Could not resolve log path)");
    }

    // 3.5 Check Clipboard Access
    print!("Windows Clipboard:        ");
    match win32::copy_text_to_clipboard("rStartup Diagnostic Test Connection") {
        Ok(_) => println!("OK (Writable)"),
        Err(e) => println!("FAILED (Error: {})", e),
    }

    // 4. Check Display Metrics
    {
        let (screen_w, screen_h) = win32::get_system_screen_resolution();
        println!("Display Resolution:       {}x{}", screen_w, screen_h);
        let dpi = win32::get_console_window_dpi();
        println!(
            "Console DPI Scale:        {}% ({} DPI)",
            (dpi as f32 / 96.0 * 100.0) as u32,
            dpi
        );
    }

    // 5. Check Power & Battery Status
    {
        print!("Power Status:             ");
        if let Some(power) = win32::query_power_status() {
            let source = if power.ac_online {
                "AC Power"
            } else {
                "Battery"
            };
            let percent = if power.battery_percent == 255 {
                "Unknown %".to_string()
            } else {
                format!("{}%", power.battery_percent)
            };
            println!("{} ({})", source, percent);
        } else {
            println!("N/A");
        }
    }

    // 6. Check Parent Shell & Terminal Emulator
    {
        let (shell, term) = win32::query_shell_and_terminal();
        println!("Parent Shell:             {}", shell);
        println!("Terminal Emulator:        {}", term);
    }

    // 7. Check BIOS Specifications
    {
        print!("Motherboard / BIOS:       ");
        if let Some(bios) = win32::query_bios_info() {
            println!(
                "{} {} (Board: {})",
                bios.manufacturer, bios.product, bios.model
            );
        } else {
            println!("N/A");
        }
    }

    println!("\nDiagnostics Complete.");
}

// ==========================================
// 3. Application State & Layout Panels
// ==========================================

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum FocusedSection {
    LeftPanel,
    RightPanel,
}

struct App {
    status_msg: String,
    status_timer: Option<Instant>,
    focus: FocusedSection,
    should_quit: bool,
    show_help: bool,
    glyphs: win32::GlyphMap,

    // Registry/Theme state variables
    dark_mode: bool,
    accent_color: Color,
    last_theme_check: Instant,

    // Interactive Input
    textbox: TextBox,

    // Background worker state
    worker_rx: Option<Receiver<WorkerEvent>>,
    worker_progress: f64,
    worker_running: bool,
    enable_toasts: bool,

    // Interactive Diagnostics selection
    selected_diagnostic: usize,
    theme_mode: String,
    refresh_rate_ms: u32,
    enable_borderless: bool,
    enable_event_log: bool,

    // Battery throttling status
    on_battery: bool,
    last_power_check: Instant,

    // Native Windows System diagnostics structures
    sys: sysinfo::System,
    networks: sysinfo::Networks,
    top_processes: Vec<(u32, String, f32, u64)>,
    network_rates: Vec<(String, u64, u64)>,
    last_metrics_refresh: Instant,

    // TUI Markdown Viewer modal status
    show_markdown: Option<String>,
    markdown_lines: Vec<Line<'static>>,
    markdown_scroll: usize,
    selection_start: Option<(u16, u16)>,
    selection_end: Option<(u16, u16)>,
    selection_pending_copy: bool,
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    if max_width == 0 {
        return vec![text.to_string()];
    }
    for paragraph in text.split('\n') {
        let mut current_line = String::new();
        for word in paragraph.split_whitespace() {
            if current_line.is_empty() {
                current_line.push_str(word);
            } else if current_line.len() + 1 + word.len() <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }
    lines
}

fn format_help_row(
    key: &str,
    description: &str,
    max_desc_width: usize,
    theme: &ThemeColors,
) -> Vec<Line<'static>> {
    let wrapped = wrap_text(description, max_desc_width);
    let mut lines = Vec::new();

    let key_col_width = 18;
    let key_str = format!("  {:<15} ", key);

    if wrapped.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                key_str,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": ", Style::default().fg(theme.text_main)),
        ]));
    } else {
        for (i, chunk) in wrapped.into_iter().enumerate() {
            if i == 0 {
                lines.push(Line::from(vec![
                    Span::styled(
                        key_str.clone(),
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(": ", Style::default().fg(theme.text_main)),
                    Span::styled(chunk, Style::default().fg(theme.text_main)),
                ]));
            } else {
                let padding = " ".repeat(key_col_width + 2);
                lines.push(Line::from(vec![
                    Span::styled(padding, Style::default().fg(theme.text_main)),
                    Span::styled(chunk, Style::default().fg(theme.text_main)),
                ]));
            }
        }
    }
    lines
}

impl App {
    fn new(config: &config::AppConfig) -> Self {
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
        Self {
            status_msg:
                "Press Tab to cycle panel focus. Use arrow keys to browse diagnostics. (h for help)"
                    .to_string(),
            status_timer: None,
            focus: FocusedSection::LeftPanel,
            should_quit: false,
            show_help: false,
            glyphs,
            dark_mode,
            accent_color,
            last_theme_check: Instant::now(),
            textbox: TextBox::new(),
            worker_rx: None,
            worker_progress: 0.0,
            worker_running: false,
            enable_toasts: config.enable_toasts,
            selected_diagnostic: 0,
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
        }
    }

    fn set_status(&mut self, msg: String) {
        self.status_msg = msg;
        self.status_timer = Some(Instant::now());
        log_message("INFO", &format!("Status updated: {}", self.status_msg));
    }

    fn select_next_diagnostic(&mut self) {
        self.selected_diagnostic = (self.selected_diagnostic + 1) % 9;
        self.set_status(format!(
            "Selected item: {}",
            self.get_diagnostic_name(self.selected_diagnostic)
        ));
    }

    fn select_prev_diagnostic(&mut self) {
        if self.selected_diagnostic == 0 {
            self.selected_diagnostic = 8;
        } else {
            self.selected_diagnostic -= 1;
        }
        self.set_status(format!(
            "Selected item: {}",
            self.get_diagnostic_name(self.selected_diagnostic)
        ));
    }

    fn get_diagnostic_name(&self, idx: usize) -> &'static str {
        match idx {
            0 => "System BIOS Info",
            1 => "Power & Battery",
            2 => "Shell & Terminal",
            3 => "Display Details",
            4 => "App Configuration",
            5 => "Top Processes (CPU)",
            6 => "Logical Drive Storage",
            7 => "Network Adapter Metrics",
            8 => "Windows Services Diagnostics",
            _ => "Unknown",
        }
    }

    /// Refresh process list and network adapters data dynamically.
    fn refresh_system_metrics(&mut self) {
        if self.last_metrics_refresh.elapsed() > Duration::from_millis(1500) {
            self.last_metrics_refresh = Instant::now();
            self.sys.refresh_processes();
            self.networks.refresh();

            // 1. Calculate top processes by CPU usage (normalized by logical CPU core count)
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
            procs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
            procs.truncate(5);
            self.top_processes = procs;

            // 2. Fetch network rates
            let mut rates = Vec::new();
            for (name, data) in &self.networks {
                rates.push((name.clone(), data.received(), data.transmitted()));
            }
            self.network_rates = rates;
        }
    }

    /// Parse and display an embedded markdown document for dynamic in-TUI modal rendering.
    fn open_embedded_markdown(&mut self, title: &str, content: &str) {
        self.markdown_lines =
            parse_markdown_to_lines(content, &get_theme(self.dark_mode, self.accent_color));
        self.show_markdown = Some(title.to_string());
        self.markdown_scroll = 0;
        self.set_status(format!("Opened document: {}", title));
    }

    fn check_status_decay(&mut self) {
        if let Some(t) = self.status_timer {
            if t.elapsed() > Duration::from_secs(4) {
                self.status_msg = if self.textbox.active {
                    "Typing mode active. Press ESC to stop editing.".to_string()
                } else {
                    "Press Tab to cycle panel focus. Press Enter to interact.".to_string()
                };
                self.status_timer = None;
            }
        }
    }

    /// Checks the Windows Registry for theme/color changes and syncs TUI in real-time.
    fn sync_theme_if_needed(&mut self, config: &config::AppConfig) {
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
    fn sync_power_status_if_needed(&mut self) {
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

    /// Poll for asynchronous background task events.
    fn poll_worker_channel(&mut self) {
        let mut completed = false;
        let mut status_update = None;

        if let Some(ref rx) = self.worker_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    WorkerEvent::Progress(progress) => {
                        self.worker_progress = progress;
                        status_update = Some(format!("Task progress: {:.0}%", progress * 100.0));
                    }
                    WorkerEvent::Success(message) => {
                        self.worker_progress = 1.0;
                        self.worker_running = false;
                        completed = true;
                        if self.enable_toasts {
                            win32::show_toast_notification("rsta Task Completed", &message);
                        }
                        status_update = Some(message);
                    }
                    WorkerEvent::Error(err) => {
                        self.worker_running = false;
                        completed = true;
                        if self.enable_toasts {
                            win32::show_toast_notification("rsta Task Failed", &err);
                        }
                        status_update = Some(format!("Task failed: {}", err));
                    }
                }
            }
        }

        if let Some(msg) = status_update {
            if completed {
                self.set_status(msg);
            } else {
                self.status_msg = msg;
            }
        }

        if completed {
            self.worker_rx = None;
        }
    }
}

// ==========================================
// 4. Main Entrypoint & Render Loop
// ==========================================

fn main() -> io::Result<()> {
    // Parse CLI arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "version" | "--version" | "-v" => {
                println!("rsta v{}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "help" | "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "doctor" => {
                run_doctor();
                return Ok(());
            }
            "tui" => {
                // Proceed to run TUI
            }
            other => {
                eprintln!("Unknown command: {}", other);
                print_help();
                std::process::exit(1);
            }
        }
    }

    // Load application configuration
    let config = config::AppConfig::load();

    // Initialize logging switch
    logger::set_event_log_enabled(config.enable_event_log);
    log_message(
        "START",
        &format!("Application initializing with config: {:?}", config),
    );

    // Restore terminal if application crashes/panics
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let msg = panic_info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| {
                panic_info
                    .payload()
                    .downcast_ref::<String>()
                    .map(|s| s.as_str())
            })
            .unwrap_or("unknown panic");
        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_default();
        log_message("PANIC", &format!("Panic occurred at {}: {}", location, msg));

        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));

    // Enforce single instance constraint
    let _instance_guard = match SingleInstanceGuard::try_new() {
        Ok(g) => g,
        Err(e) => {
            log_message(
                "ERROR",
                &format!("SingleInstanceGuard blocked launch: {}", e),
            );
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Set console tab title and clean up on exit
    let _title_guard = ConsoleTitleGuard::new("rSta");

    enable_raw_mode()?;
    let mut stdout = io::stdout();

    // Force scalable minimal size or custom sizing
    let _ = execute!(stdout, crossterm::terminal::SetSize(110, 38));
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Enable borderless console framing immediately after size adjustment if configured
    let _borderless = if config.enable_borderless {
        Some(BorderlessConsole::enable())
    } else {
        None
    };

    // Allow console size/style changes to propagate to the buffer
    std::thread::sleep(Duration::from_millis(50));

    if _borderless.is_none() {
        win32::center_console_window();
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(&config);
    let tick_rate = Duration::from_millis(config.refresh_rate_ms as u64);
    let mut last_tick = Instant::now();

    log_message("RUN", "Entering main event loop");

    while !app.should_quit {
        app.check_status_decay();
        app.sync_theme_if_needed(&config);
        app.sync_power_status_if_needed();
        app.refresh_system_metrics();
        app.poll_worker_channel();

        terminal.draw(|f| draw_ui(f, &mut app))?;

        let current_tick_rate = if app.on_battery {
            tick_rate * 2
        } else {
            tick_rate
        };

        let timeout = current_tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        log_message("KEYPRESS", &format!("Code: {:?}", key.code));

                        // Markdown viewer intercept keys
                        if app.show_markdown.is_some() {
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('q') => {
                                    app.show_markdown = None;
                                    app.set_status("Document viewer closed.".to_string());
                                }
                                KeyCode::F(1) => {
                                    app.open_embedded_markdown("README.md", README_CONTENT);
                                }
                                KeyCode::F(2) => {
                                    app.open_embedded_markdown("SUPPORT.md", SUPPORT_CONTENT);
                                }
                                KeyCode::F(3) => {
                                    app.open_embedded_markdown("LICENSE.md", LICENSE_CONTENT);
                                }
                                KeyCode::F(4) => {
                                    app.open_embedded_markdown("COPYRIGHT.md", COPYRIGHT_CONTENT);
                                }
                                KeyCode::F(5) => {
                                    app.open_embedded_markdown("PRIVACY.md", PRIVACY_CONTENT);
                                }
                                KeyCode::F(6) => {
                                    app.open_embedded_markdown("SECURITY.md", SECURITY_CONTENT);
                                }
                                KeyCode::F(7) => {
                                    app.open_embedded_markdown("CONTRIBUTING.md", CONTRIBUTING_CONTENT);
                                }
                                KeyCode::Up => {
                                    app.markdown_scroll = app.markdown_scroll.saturating_sub(1);
                                }
                                KeyCode::Down => {
                                    if app.markdown_scroll + 10 < app.markdown_lines.len() {
                                        app.markdown_scroll += 1;
                                    }
                                }
                                KeyCode::PageUp => {
                                    app.markdown_scroll = app.markdown_scroll.saturating_sub(15);
                                }
                                KeyCode::PageDown => {
                                    if app.markdown_scroll + 15 < app.markdown_lines.len() {
                                        app.markdown_scroll += 15;
                                    } else {
                                        app.markdown_scroll =
                                            app.markdown_lines.len().saturating_sub(10);
                                    }
                                }
                                _ => {}
                            }
                            continue;
                        }

                        // Help overlay intercept keys
                        if app.show_help {
                            match key.code {
                                KeyCode::Esc
                                | KeyCode::Char('q')
                                | KeyCode::Char('h') => {
                                    app.show_help = false;
                                    app.set_status("Help overlay closed.".to_string());
                                }
                                KeyCode::F(1) => {
                                    app.show_help = false;
                                    app.open_embedded_markdown("README.md", README_CONTENT);
                                }
                                KeyCode::F(2) => {
                                    app.show_help = false;
                                    app.open_embedded_markdown("SUPPORT.md", SUPPORT_CONTENT);
                                }
                                KeyCode::F(3) => {
                                    app.show_help = false;
                                    app.open_embedded_markdown("LICENSE.md", LICENSE_CONTENT);
                                }
                                KeyCode::F(4) => {
                                    app.show_help = false;
                                    app.open_embedded_markdown("COPYRIGHT.md", COPYRIGHT_CONTENT);
                                }
                                KeyCode::F(5) => {
                                    app.show_help = false;
                                    app.open_embedded_markdown("PRIVACY.md", PRIVACY_CONTENT);
                                }
                                KeyCode::F(6) => {
                                    app.show_help = false;
                                    app.open_embedded_markdown("SECURITY.md", SECURITY_CONTENT);
                                }
                                KeyCode::F(7) => {
                                    app.show_help = false;
                                    app.open_embedded_markdown("CONTRIBUTING.md", CONTRIBUTING_CONTENT);
                                }
                                _ => {}
                            }
                            continue;
                        }

                        // Textbox intercept keys
                        if app.textbox.active {
                            match key.code {
                                KeyCode::Esc => {
                                    app.textbox.active = false;
                                    app.set_status("TextBox exited.".to_string());
                                }
                                KeyCode::Enter => {
                                    app.textbox.active = false;
                                    app.set_status(format!("Saved: {}", app.textbox.text));
                                }
                                other => {
                                    app.textbox.handle_key(other);
                                }
                            }
                            continue;
                        }

                        // Standard hotkeys
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.should_quit = true;
                            }
                            KeyCode::Char('c') | KeyCode::Char('C') => {
                                let text = get_diagnostic_details_text(&app);
                                match win32::copy_text_to_clipboard(&text) {
                                    Ok(_) => app.set_status(
                                        "📋 Copied diagnostic details to Windows Clipboard!"
                                            .to_string(),
                                    ),
                                    Err(e) => {
                                        app.set_status(format!("❌ Clipboard copy failed: {}", e))
                                    }
                                }
                            }
                            KeyCode::F(1) => {
                                app.open_embedded_markdown("README.md", README_CONTENT);
                            }
                            KeyCode::F(2) => {
                                app.open_embedded_markdown("SUPPORT.md", SUPPORT_CONTENT);
                            }
                            KeyCode::F(3) => {
                                app.open_embedded_markdown("LICENSE.md", LICENSE_CONTENT);
                            }
                            KeyCode::F(4) => {
                                app.open_embedded_markdown("COPYRIGHT.md", COPYRIGHT_CONTENT);
                            }
                            KeyCode::F(5) => {
                                app.open_embedded_markdown("PRIVACY.md", PRIVACY_CONTENT);
                            }
                            KeyCode::F(6) => {
                                app.open_embedded_markdown("SECURITY.md", SECURITY_CONTENT);
                            }
                            KeyCode::F(7) => {
                                app.open_embedded_markdown("CONTRIBUTING.md", CONTRIBUTING_CONTENT);
                            }
                            KeyCode::Char('h') => {
                                app.show_help = true;
                                app.set_status(
                                    "Help overlay active. Press ESC/q to close.".to_string(),
                                );
                            }
                            KeyCode::Tab => {
                                app.focus = match app.focus {
                                    FocusedSection::LeftPanel => FocusedSection::RightPanel,
                                    FocusedSection::RightPanel => FocusedSection::LeftPanel,
                                };
                                app.set_status(format!(
                                    "Focused Section: {}",
                                    match app.focus {
                                        FocusedSection::LeftPanel => "Left Input Panel",
                                        FocusedSection::RightPanel => "Right Worker Panel",
                                    }
                                ));
                            }
                            KeyCode::Up => {
                                if app.focus == FocusedSection::LeftPanel {
                                    app.select_prev_diagnostic();
                                }
                            }
                            KeyCode::Down => {
                                if app.focus == FocusedSection::LeftPanel {
                                    app.select_next_diagnostic();
                                }
                            }
                            KeyCode::Enter => match app.focus {
                                FocusedSection::LeftPanel => {
                                    app.textbox.active = true;
                                    app.set_status(
                                        "TextBox edit active. Type text, press Enter to save."
                                            .to_string(),
                                    );
                                }
                                FocusedSection::RightPanel => {
                                    if app.worker_running {
                                        app.set_status("A task is already running.".to_string());
                                    } else {
                                        let (tx, rx) = channel();
                                        app.worker_rx = Some(rx);
                                        app.worker_progress = 0.0;
                                        app.worker_running = true;
                                        app.set_status("Spawning background thread...".to_string());
                                        worker::spawn_background_task(tx);
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Event::Mouse(mouse) => match mouse.kind {
                    event::MouseEventKind::Down(event::MouseButton::Left) => {
                        app.selection_start = Some((mouse.column, mouse.row));
                        app.selection_end = Some((mouse.column, mouse.row));
                        app.selection_pending_copy = false;
                    }
                    event::MouseEventKind::Drag(event::MouseButton::Left) => {
                        if app.selection_start.is_some() {
                            app.selection_end = Some((mouse.column, mouse.row));
                        }
                    }
                    event::MouseEventKind::Up(event::MouseButton::Left) => {
                        if let (Some(start), Some(end)) = (app.selection_start, app.selection_end) {
                            if start != end {
                                app.selection_pending_copy = true;
                            } else {
                                app.selection_start = None;
                                app.selection_end = None;
                            }
                        }
                    }
                    event::MouseEventKind::ScrollUp => {
                        if app.show_markdown.is_some() {
                            app.markdown_scroll = app.markdown_scroll.saturating_sub(3);
                        }
                    }
                    event::MouseEventKind::ScrollDown => {
                        if app.show_markdown.is_some() {
                            let max_scroll = app.markdown_lines.len().saturating_sub(10);
                            if app.markdown_scroll < max_scroll {
                                app.markdown_scroll = (app.markdown_scroll + 3).min(max_scroll);
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    log_message("EXIT", "Shutting down cleanly.");

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

/// Helper function to center a layout chunk for modal popups.
fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_ui(f: &mut ratatui::Frame, app: &mut App) {
    let size = f.area();
    let theme = get_theme(app.dark_mode, app.accent_color);

    // 0. Terminal Size Layout Guard
    if size.width < 110 || size.height < 38 {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(255, 85, 85)))
            .title(Span::styled(
                " ⚠️  Terminal Sizing Warning ",
                Style::default()
                    .fg(Color::Rgb(255, 85, 85))
                    .add_modifier(Modifier::BOLD),
            ));

        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Layout Constraints Not Met",
                Style::default()
                    .fg(Color::Rgb(255, 85, 85))
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!(
                "  Current Terminal Size: {}x{}",
                size.width, size.height
            )),
            Line::from("  Minimum Required Size: 110x38"),
            Line::from(""),
            Line::from(
                "  Please resize or maximize your terminal window to resume standard rendering.",
            ),
        ];
        let p = Paragraph::new(text)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

        let area = centered_rect(80, 50, size);
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(p, area);
        return;
    }

    // Core Layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title Banner
            Constraint::Min(10),   // Content panels
            Constraint::Length(3), // Status Bar
        ])
        .split(size);

    // 1. Title Banner
    let username = std::env::var("USERNAME")
        .unwrap_or_else(|_| std::env::var("USER").unwrap_or_else(|_| "user".to_string()));
    let host_name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "localhost".to_string());
    let os_str = win32::query_os_version();

    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " Rust Startup Manager ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let title_line = Line::from(vec![
        Span::styled(
            format!(" rSta v{} ", env!("CARGO_PKG_VERSION")),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(theme.border)),
        Span::styled(
            "Press h for help",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(theme.border)),
        Span::styled(
            format!("{}@{}", username, host_name),
            Style::default()
                .fg(Color::Rgb(255, 215, 0))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(theme.border)),
        Span::styled(os_str, Style::default().fg(theme.text_main)),
    ]);

    f.render_widget(Paragraph::new(title_line).block(title_block), chunks[0]);

    // 2. Main Content splitting horizontally
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Left panel: Text input box
    let left_active = app.focus == FocusedSection::LeftPanel;
    let left_border = if left_active {
        if app.textbox.active {
            Color::Rgb(0, 255, 127)
        } else {
            theme.border_active
        }
    } else {
        theme.border
    };
    let left_title = if app.textbox.active {
        " Left Input Panel (EDITING) "
    } else {
        " Left Input Panel "
    };
    let left_block = Block::default()
        .borders(Borders::ALL)
        .title(left_title)
        .title_style(
            Style::default()
                .fg(left_border)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(left_border));

    let left_inner = left_block.inner(content_chunks[0]);
    f.render_widget(left_block, content_chunks[0]);

    let left_sub_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9), // List of 9 items
            Constraint::Length(1), // Separator
            Constraint::Min(6),    // Details section
            Constraint::Length(1), // Separator
            Constraint::Length(2), // Text Input area
        ])
        .split(left_inner);

    // Render diagnostics list
    let items = vec![
        "System BIOS Info",
        "Power & Battery",
        "Shell & Terminal",
        "Display Details",
        "App Configuration",
        "Top Processes (CPU)",
        "Logical Drive Storage",
        "Network Adapter Metrics",
        "Windows Services Diagnostics",
    ];
    let accent_list = AccentList::new(
        items,
        app.selected_diagnostic,
        theme.accent,
        theme.text_dim,
        theme.text_main,
        if app.glyphs.status_ok == "[OK]" {
            ">"
        } else {
            "▶"
        },
    );
    f.render_widget(accent_list, left_sub_chunks[0]);

    // Render first separator
    let sep_char = if app.glyphs.status_ok == "[OK]" {
        "-"
    } else {
        "─"
    };
    let separator_text = sep_char.repeat(left_sub_chunks[1].width as usize);
    let sep1 = Paragraph::new(Line::from(Span::styled(
        separator_text.clone(),
        Style::default().fg(theme.border),
    )));
    f.render_widget(sep1, left_sub_chunks[1]);

    // Render details section
    let mut details_lines = Vec::new();
    match app.selected_diagnostic {
        0 => {
            details_lines.push(Line::from(Span::styled(
                "--- BIOS Specifications ---",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            if let Some(bios) = win32::query_bios_info() {
                details_lines.push(Line::from(format!(
                    "  Manufacturer:  {}",
                    bios.manufacturer
                )));
                details_lines.push(Line::from(format!("  Product:       {}", bios.product)));
                details_lines.push(Line::from(format!("  Model / Board: {}", bios.model)));
            } else {
                details_lines.push(Line::from("  No BIOS information detected."));
            }
        }
        1 => {
            details_lines.push(Line::from(Span::styled(
                "--- Power & Battery Life ---",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            if let Some(power) = win32::query_power_status() {
                let source = if power.ac_online {
                    "AC Online"
                } else {
                    "Battery (Discharging)"
                };
                let pct = if power.battery_percent == 255 {
                    "Unknown".to_string()
                } else {
                    format!("{}%", power.battery_percent)
                };
                details_lines.push(Line::from(format!("  Power Source:  {}", source)));
                details_lines.push(Line::from(format!("  Charge Level:  {}", pct)));
            } else {
                details_lines.push(Line::from("  No battery/power status detected."));
            }
        }
        2 => {
            details_lines.push(Line::from(Span::styled(
                "--- Parent Environment ---",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            let (shell, term) = win32::query_shell_and_terminal();
            details_lines.push(Line::from(format!("  Active Shell:  {}", shell)));
            details_lines.push(Line::from(format!("  Terminal Host: {}", term)));
        }
        3 => {
            details_lines.push(Line::from(Span::styled(
                "--- Monitor & Resolution ---",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            let (w, h) = win32::get_system_screen_resolution();
            let dpi = win32::get_console_window_dpi();
            details_lines.push(Line::from(format!("  Screen Size:   {}x{}", w, h)));
            details_lines.push(Line::from(format!(
                "  Console DPI:   {}% ({} DPI)",
                (dpi as f32 / 96.0 * 100.0) as u32,
                dpi
            )));
        }
        4 => {
            details_lines.push(Line::from(Span::styled(
                "--- Active Configuration ---",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            details_lines.push(Line::from(format!("  Theme Mode:    {}", app.theme_mode)));
            details_lines.push(Line::from(format!(
                "  Refresh Rate:  {}ms",
                app.refresh_rate_ms
            )));
            let current_rate = if app.on_battery {
                app.refresh_rate_ms * 2
            } else {
                app.refresh_rate_ms
            };
            let throttle_status = if app.on_battery {
                " (Throttling Active)"
            } else {
                " (Full Speed)"
            };
            details_lines.push(Line::from(format!(
                "  Active Tick:   {}ms{}",
                current_rate, throttle_status
            )));
            details_lines.push(Line::from(format!(
                "  Borderless:    {}",
                app.enable_borderless
            )));
            details_lines.push(Line::from(format!(
                "  Toast Alerts:  {}",
                app.enable_toasts
            )));
            details_lines.push(Line::from(format!(
                "  Event Log:     {}",
                app.enable_event_log
            )));
        }
        5 => {
            details_lines.push(Line::from(Span::styled(
                "--- Top 5 CPU Processes ---",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            details_lines.push(Line::from(format!(
                "  {:>6}  {:<20}  {:>8}  {:>10}",
                "PID", "Name", "CPU %", "Memory"
            )));
            for (pid, name, cpu, mem) in &app.top_processes {
                let mem_mb = *mem as f64 / 1024.0 / 1024.0;
                details_lines.push(Line::from(format!(
                    "  {:>6}  {:<20}  {:>7.1}%  {:>8.1} MB",
                    pid,
                    if name.len() > 20 { &name[..17] } else { name },
                    cpu,
                    mem_mb
                )));
            }
            if app.top_processes.is_empty() {
                details_lines.push(Line::from("  Querying active processes..."));
            }
        }
        6 => {
            details_lines.push(Line::from(Span::styled(
                "--- Logical Drive Storage ---",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            let drives = win32::query_disk_drives();
            for drive in drives {
                let total_gb = drive.total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
                let free_gb = drive.free_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
                let used_gb = total_gb - free_gb;
                let pct = if total_gb > 0.0 {
                    (used_gb / total_gb) * 100.0
                } else {
                    0.0
                };

                let bar_width = 15;
                let filled = ((pct / 100.0) * bar_width as f64).round() as usize;
                let bar: String = std::iter::repeat('■')
                    .take(filled)
                    .chain(std::iter::repeat('░').take(bar_width - filled))
                    .collect();

                details_lines.push(Line::from(format!(
                    "  {:<3} [{}] {:>5.1}% ({:.1} GB / {:.1} GB free)",
                    drive.path, bar, pct, free_gb, total_gb
                )));
            }
            if win32::query_disk_drives().is_empty() {
                details_lines.push(Line::from("  No active storage drives found."));
            }
        }
        7 => {
            details_lines.push(Line::from(Span::styled(
                "--- Network Adapter Metrics ---",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            if let Some(ip) = win32::query_local_ip() {
                details_lines.push(Line::from(format!("  Local IP Address:  {}", ip)));
            } else {
                details_lines.push(Line::from("  Local IP Address:  Not Connected"));
            }
            details_lines.push(Line::from(""));
            details_lines.push(Line::from(format!(
                "  {:<20}  {:>12}  {:>12}",
                "Interface", "Rx Total", "Tx Total"
            )));
            for (name, rx, tx) in &app.network_rates {
                let rx_mb = *rx as f64 / 1024.0 / 1024.0;
                let tx_mb = *tx as f64 / 1024.0 / 1024.0;
                details_lines.push(Line::from(format!(
                    "  {:<20}  {:>9.2} MB  {:>9.2} MB",
                    if name.len() > 20 { &name[..17] } else { name },
                    rx_mb,
                    tx_mb
                )));
            }
            if app.network_rates.is_empty() {
                details_lines.push(Line::from("  Scanning active network adapters..."));
            }
        }
        8 => {
            details_lines.push(Line::from(Span::styled(
                "--- Windows Services Status ---",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            let services = vec![
                ("wuauserv", "Windows Update"),
                ("Spooler", "Print Spooler"),
                ("EventLog", "Windows Event Log"),
                ("Dhcp", "DHCP Client"),
                ("Dnscache", "DNS Client"),
            ];
            for (service_id, display_name) in services {
                let status = win32::query_windows_service_status(service_id);
                let color = match status.as_str() {
                    "RUNNING" => Color::Rgb(0, 255, 127),
                    "STOPPED" => Color::Rgb(255, 85, 85),
                    _ => theme.text_dim,
                };
                details_lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {:<18} : ", display_name),
                        Style::default().fg(theme.text_main),
                    ),
                    Span::styled(
                        status,
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                ]));
            }
        }
        _ => {}
    }
    f.render_widget(Paragraph::new(details_lines), left_sub_chunks[2]);

    // Render second separator
    let sep2 = Paragraph::new(Line::from(Span::styled(
        separator_text,
        Style::default().fg(theme.border),
    )));
    f.render_widget(sep2, left_sub_chunks[3]);

    // Render text input block
    let cursor_indicator = if app.textbox.active
        && (Instant::now().duration_since(Instant::now()).as_millis() / 500) % 2 == 0
    {
        "|"
    } else {
        " "
    };
    let textbox_display = format!("> {}{}", app.textbox.text, cursor_indicator);
    let input_lines = vec![
        Line::from(Span::styled(
            if app.textbox.active {
                "Type value & press Enter"
            } else {
                "Press Enter to edit config title override"
            },
            Style::default().fg(theme.text_dim),
        )),
        Line::from(Span::styled(
            textbox_display,
            Style::default()
                .fg(theme.text_main)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    f.render_widget(Paragraph::new(input_lines), left_sub_chunks[4]);

    // Right panel: Async Worker progress
    let right_active = app.focus == FocusedSection::RightPanel;
    let right_border = if right_active {
        theme.border_active
    } else {
        theme.border
    };
    let right_block = Block::default()
        .borders(Borders::ALL)
        .title(" Right Worker Panel ")
        .title_style(
            Style::default()
                .fg(right_border)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(right_border));

    let right_inner = right_block.inner(content_chunks[1]);
    f.render_widget(right_block, content_chunks[1]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Instructions
            Constraint::Length(2), // Spacer
            Constraint::Length(1), // Status
            Constraint::Length(2), // Spacer
            Constraint::Length(1), // Progress bar/gauge (1 line height)
            Constraint::Min(2),
        ])
        .split(right_inner);

    f.render_widget(
        Paragraph::new(Line::from(
            "Press Enter to execute background worker thread:",
        )),
        right_chunks[0],
    );

    if app.worker_running {
        let status_p = Paragraph::new(Line::from(vec![
            Span::styled("  Status:  ", Style::default().fg(theme.text_dim)),
            Span::styled(
                "Running Task...",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        f.render_widget(status_p, right_chunks[2]);

        let gauge = AccentGauge::new(
            app.worker_progress,
            "Processing",
            theme.accent,
            theme.border,
            app.glyphs.status_ok != "[OK]",
        );
        f.render_widget(gauge, right_chunks[4]);
    } else {
        let status_p = Paragraph::new(Line::from(vec![
            Span::styled("  Status:  ", Style::default().fg(theme.text_dim)),
            Span::styled("Idle", Style::default().fg(theme.text_main)),
        ]));
        f.render_widget(status_p, right_chunks[2]);
    }

    // 3. Status Bar Footer
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " Status ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let footer_inner = footer_block.inner(chunks[2]);
    f.render_widget(footer_block, chunks[2]);

    let is_default_msg = app.status_msg
        == "Press Tab to cycle panel focus. Press Enter to interact."
        || app.status_msg == "Typing mode active. Press ESC to stop editing.";
    let (text_color, status_text) = if is_default_msg {
        (theme.text_dim, app.status_msg.clone())
    } else {
        let lower = app.status_msg.to_lowercase();
        let color = if lower.contains("failed") || lower.contains("error") {
            Color::Rgb(255, 85, 85)
        } else {
            theme.accent
        };
        (color, app.status_msg.clone())
    };

    let footer_p = Paragraph::new(Line::from(vec![Span::styled(
        status_text,
        Style::default().fg(text_color).add_modifier(Modifier::BOLD),
    )]));
    f.render_widget(footer_p, footer_inner);

    // 4. Help Overlay Modal
    if app.show_help {
        let area = centered_rect(65, 70, size);
        let popup_block = Block::default()
            .title(" Keyboard Shortcuts & TUI Commands ")
            .title_style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent));

        let key_col_width = 18;
        let border_padding = 2;
        let total_inner_width = area.width.saturating_sub(border_padding);
        let max_desc_width = (total_inner_width as usize)
            .saturating_sub(key_col_width)
            .saturating_sub(2); // for ": "

        let mut help_text = Vec::new();
        help_text.push(Line::from(""));

        help_text.extend(format_help_row(
            "Tab/Shift-Tab",
            "Cycle active panel focus",
            max_desc_width,
            &theme,
        ));
        help_text.extend(format_help_row(
            "Enter",
            "Edit textbox (Left Panel) or Run worker (Right Panel)",
            max_desc_width,
            &theme,
        ));
        help_text.extend(format_help_row(
            "Esc / q",
            "Close dialogs / Help Overlay, or Quit application",
            max_desc_width,
            &theme,
        ));
        help_text.extend(format_help_row(
            "h",
            "Toggle this help shortcut overlay modal",
            max_desc_width,
            &theme,
        ));
        help_text.extend(format_help_row(
            "c",
            "Copy active diagnostic details to Windows Clipboard",
            max_desc_width,
            &theme,
        ));

        help_text.push(Line::from(""));
        help_text.extend(format_help_row(
            "F1",
            "View README.md document",
            max_desc_width,
            &theme,
        ));
        help_text.extend(format_help_row(
            "F2",
            "View SUPPORT.md document",
            max_desc_width,
            &theme,
        ));
        help_text.extend(format_help_row(
            "F3",
            "View LICENSE.md document",
            max_desc_width,
            &theme,
        ));
        help_text.extend(format_help_row(
            "F4",
            "View COPYRIGHT.md document",
            max_desc_width,
            &theme,
        ));
        help_text.extend(format_help_row(
            "F5",
            "View PRIVACY.md document",
            max_desc_width,
            &theme,
        ));
        help_text.extend(format_help_row(
            "F6",
            "View SECURITY.md document",
            max_desc_width,
            &theme,
        ));
        help_text.extend(format_help_row(
            "F7",
            "View CONTRIBUTING.md document",
            max_desc_width,
            &theme,
        ));

        help_text.push(Line::from(""));
        help_text.extend(format_help_row(
            "CLI Subcommands",
            "rsta.exe [tui | doctor | version | help]",
            max_desc_width,
            &theme,
        ));

        help_text.push(Line::from(""));
        help_text.extend(format_help_row(
            "Terminal Sync",
            &format!(
                "Running in {} via {}",
                app.glyphs.terminal, app.glyphs.shell
            ),
            max_desc_width,
            &theme,
        ));
        help_text.extend(format_help_row(
            "Glyphs Status",
            &format!(
                "Config Sync Status {}  Logger Status {}",
                app.glyphs.status_ok, app.glyphs.status_ok
            ),
            max_desc_width,
            &theme,
        ));

        f.render_widget(ratatui::widgets::Clear, area);
        let paragraph = Paragraph::new(help_text).block(popup_block);
        f.render_widget(paragraph, area);
    }

    // 5. Scrollable Markdown Document Viewer Modal
    if let Some(ref filename) = app.show_markdown {
        let area = centered_rect(85, 80, size);
        let popup_block = Block::default()
            .title(format!(
                " Document Viewer: {} (Press Esc/q to Close) ",
                filename
            ))
            .title_style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent));

        // Render text lines scrollable
        let paragraph = Paragraph::new(app.markdown_lines.clone())
            .block(popup_block)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .alignment(ratatui::layout::Alignment::Left)
            .scroll((app.markdown_scroll as u16, 0));

        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(paragraph, area);
    }

    // 6. Handle Mouse Selection Highlights & Clipboard Copy
    if let (Some(start), Some(end)) = (app.selection_start, app.selection_end) {
        let buf = f.buffer_mut();
        let width = buf.area.width;
        let height = buf.area.height;

        let (col1, row1) = start;
        let (col2, row2) = end;

        let is_selected = |x: u16, y: u16| -> bool {
            let (c1, r1) = (col1, row1);
            let (c2, r2) = (col2, row2);
            if r1 == r2 {
                y == r1 && x >= c1.min(c2) && x <= c1.max(c2)
            } else if r1 < r2 {
                (y == r1 && x >= c1) || (y > r1 && y < r2) || (y == r2 && x <= c2)
            } else {
                (y == r2 && x >= c2) || (y > r2 && y < r1) || (y == r1 && x <= c1)
            }
        };

        // 1. Draw Highlight
        for y in 0..height {
            for x in 0..width {
                if is_selected(x, y) {
                    let cell = &mut buf[(x, y)];
                    cell.set_bg(Color::Rgb(0, 120, 215));
                    cell.set_fg(Color::White);
                }
            }
        }

        // 2. Perform Copy on Release
        if app.selection_pending_copy {
            let mut selected_text = String::new();
            let mut current_row: Option<u16> = None;
            let mut current_line = String::new();

            for y in 0..height {
                for x in 0..width {
                    if is_selected(x, y) {
                        let cell = &buf[(x, y)];
                        if current_row != Some(y) {
                            if current_row.is_some() {
                                selected_text.push_str(current_line.trim_end());
                                selected_text.push('\n');
                                current_line.clear();
                            }
                            current_row = Some(y);
                        }
                        current_line.push_str(cell.symbol());
                    }
                }
            }
            if !current_line.is_empty() {
                selected_text.push_str(current_line.trim_end());
            }

            if !selected_text.is_empty() {
                let _ = win32::copy_text_to_clipboard(&selected_text);
                let truncated = if selected_text.len() > 30 {
                    format!("{}...", &selected_text[..27].replace('\n', " "))
                } else {
                    selected_text.replace('\n', " ")
                };
                app.status_msg = format!("📋 Copied selection to clipboard: {}", truncated);
                app.status_timer = Some(Instant::now());
            }

            app.selection_start = None;
            app.selection_end = None;
            app.selection_pending_copy = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_text() {
        let text = "Hello world this is a test";
        let wrapped = wrap_text(text, 10);
        assert_eq!(wrapped, vec!["Hello", "world this", "is a test"]);
    }

    #[test]
    fn test_wrap_text_empty() {
        let wrapped = wrap_text("", 10);
        assert!(wrapped.is_empty());
    }

    #[test]
    fn test_parse_markdown_headers() {
        let theme = get_theme(true, Color::Blue);
        let lines = parse_markdown_to_lines("# Test Header\n## Subheader", &theme);
        assert!(lines.len() >= 2);
    }
}

