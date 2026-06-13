#![allow(deprecated)]
use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyEventKind};
use crate::logger as file_log;
use crate::bootstrap::{init, shutdown, is_app_shutting_down, Config as BootstrapConfig};

pub mod utils;
pub mod clipboard;
pub mod logger;
pub mod bootstrap;
pub mod bootstrap_guards;
pub mod chrome;

mod config;
pub mod backend;
mod win32;
mod app;
mod ui;
mod doctor;

#[cfg(test)]
mod tests_perf;

use app::App;

pub const IGNITE_LOGO: &str = r"
         _____ __                __               
    _____/ ___// /_____ _________/ /___  ______    
   / ___/\__ \/ __/ __ `/ ___/ __  / / / / __ \   
  / /   ___/ / /_/ /_/ / /  / /_/ / /_/ / /_/ /   
 /_/   /____/\__/\__,_/_/   \__,_/\__,_/ .___/    
                                       /_/         
";

fn print_help() {
    println!("{}", IGNITE_LOGO);
    println!(
        "ignite — Rust Startup Manager (v{})\nUsage:\n  ignite.exe [command]\n\nCommands:\n  ui        Launch the interactive app dashboard (default)\n  list      Search and list all active startup applications\n  doctor    Verify system registry, log paths, and console scaling\n  version   Print application version info\n  help      Print this help message",
        env!("CARGO_PKG_VERSION")
    );
}

fn run_doctor() {
    doctor::run();
}

#[derive(Debug, PartialEq, Eq)]
pub enum CliAction {
    Ui,
    Version,
    Help,
    Doctor,
    List,
    Unknown(String),
}

pub fn parse_cli_args(args: &[String]) -> CliAction {
    if args.len() < 2 {
        return CliAction::Ui;
    }
    match args[1].as_str() {
        "version" | "--version" | "-v" => CliAction::Version,
        "help" | "--help" | "-h" => CliAction::Help,
        "doctor" => CliAction::Doctor,
        "list" => CliAction::List,
        "ui" | "--relaunched" => CliAction::Ui,
        other => CliAction::Unknown(other.to_string()),
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match parse_cli_args(&args) {
        CliAction::Version => {
            println!("ignite v{}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        CliAction::Help => {
            print_help();
            return Ok(());
        }
        CliAction::Doctor => {
            run_doctor();
            return Ok(());
        }
        CliAction::List => {
            let items = backend::scan_startup_items();
            if items.is_empty() {
                println!("No startup items found.");
            } else {
                println!("{:<30} {:<10} {:<15} {:<12} {}", "Application Name", "Status", "Startup Type", "Impact", "Command");
                println!("{:-<120}", "");
                for item in items {
                    let status = if item.enabled { "Enabled" } else { "Disabled" };
                    let type_str = if item.location_type.to_lowercase().contains("user") {
                        "User"
                    } else {
                        "System"
                    };
                    println!("{:<30} {:<10} {:<15} {:<12} {}", item.name, status, type_str, item.impact, item.command);
                }
            }
            return Ok(());
        }
        CliAction::Ui => {}
        CliAction::Unknown(other) => {
            eprintln!("Unknown command: {}", other);
            print_help();
            std::process::exit(1);
        }
    }

    let config = config::AppConfig::load();
    win32::relaunch_in_conhost_if_needed();

    #[cfg(windows)]
    let _hwnd = win32::hide_console_at_startup();

    file_log::set_log_app_name("app/ignite");
    file_log::set_event_log_enabled(config.enable_event_log);
    file_log::log_message(
        "START",
        &format!("Application initializing with config: {:?}", config),
    );

    let mut tui_config = BootstrapConfig::new("ignite");
    tui_config.borderless = config.enable_borderless;
    tui_config.size = (100, 35);

    let (mut terminal, _guards) = init(tui_config)?;

    #[cfg(windows)]
    win32::show_console_window();

    let mut app = App::new(&config);
    let tick_rate = Duration::from_millis(config.refresh_rate_ms as u64);
    let mut last_tick = Instant::now();

    file_log::log_message("RUN", "Entering main event loop");

    while !app.should_quit {
        if is_app_shutting_down() {
            break;
        }
        app.check_status_decay();
        app.sync_theme_if_needed(&config);
        app.sync_power_status_if_needed();
        app.refresh_system_metrics();

        terminal.draw(|f| ui::draw_ui(f, &mut app))?;

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
                        file_log::log_message("KEYPRESS", &format!("Code: {:?}", key.code));
                        app::handle_key(&mut app, key);
                    }
                }
                Event::Mouse(mouse) => {
                    app::handle_mouse(&mut app, mouse);
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    file_log::log_message("EXIT", "Shutting down cleanly.");

    shutdown(&mut terminal)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;
    use crate::app::get_theme;
    use crate::ui::text_format::{wrap_text, parse_markdown_to_lines};

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

    #[test]
    fn test_backup_database_serialization() {
        let mut db = backend::BackupDatabase::default();
        let entry = backend::BackupEntry {
            uuid: "test-id".to_string(),
            timestamp: "2026-06-05T20:53:11".to_string(),
            name: "Test App".to_string(),
            command: "C:\\Windows\\system32\\cmd.exe".to_string(),
            location_type: "Registry (User)".to_string(),
            location_path: "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string(),
            key_name: "TestApp".to_string(),
        };
        db.entries.push(entry);

        let temp_dir = std::env::temp_dir().join(format!(
            "ignite_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let original_appdata = std::env::var("APPDATA").ok();
        unsafe {
            std::env::set_var("APPDATA", &temp_dir);
        }

        db.save().unwrap();

        let loaded = backend::BackupDatabase::load();
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].name, "Test App");

        if let Some(val) = original_appdata {
            unsafe { std::env::set_var("APPDATA", val); }
        }
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_parse_cli_args_defaults() {
        assert_eq!(parse_cli_args(&[]), CliAction::Ui);
        assert_eq!(parse_cli_args(&["ignite".to_string()]), CliAction::Ui);
        assert_eq!(parse_cli_args(&["ignite".to_string(), "ui".to_string()]), CliAction::Ui);
        assert_eq!(parse_cli_args(&["ignite".to_string(), "--relaunched".to_string()]), CliAction::Ui);
    }

    #[test]
    fn test_parse_cli_args_commands() {
        assert_eq!(parse_cli_args(&["ignite".to_string(), "version".to_string()]), CliAction::Version);
        assert_eq!(parse_cli_args(&["ignite".to_string(), "--version".to_string()]), CliAction::Version);
        assert_eq!(parse_cli_args(&["ignite".to_string(), "-v".to_string()]), CliAction::Version);

        assert_eq!(parse_cli_args(&["ignite".to_string(), "help".to_string()]), CliAction::Help);
        assert_eq!(parse_cli_args(&["ignite".to_string(), "--help".to_string()]), CliAction::Help);
        assert_eq!(parse_cli_args(&["ignite".to_string(), "-h".to_string()]), CliAction::Help);

        assert_eq!(parse_cli_args(&["ignite".to_string(), "doctor".to_string()]), CliAction::Doctor);
        assert_eq!(parse_cli_args(&["ignite".to_string(), "list".to_string()]), CliAction::List);

        assert_eq!(
            parse_cli_args(&["ignite".to_string(), "invalid_cmd".to_string()]),
            CliAction::Unknown("invalid_cmd".to_string())
        );
    }
}
