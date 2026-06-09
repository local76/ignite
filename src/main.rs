#![allow(deprecated)]
use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
};

mod config;
mod logger;
mod startup;
mod win32;
mod app;
mod ui;

use logger::log_message;
use app::App;
use win32::{ConsoleTitleGuard, SingleInstanceGuard};

pub const ignite_LOGO: &str = r"
         _____ __                __               
    _____/ ___// /_____ _________/ /___  ______    
   / ___/\__ \/ __/ __ `/ ___/ __  / / / / __ \   
  / /   ___/ / /_/ /_/ / /  / /_/ / /_/ / /_/ /   
 /_/   /____/\__/\__,_/_/   \__,_/\__,_/ .___/    
                                      /_/         
";

fn print_help() {
    println!("{}", ignite_LOGO);
    println!(
        "rstart — Rust Startup Manager (v{})",
        env!("CARGO_PKG_VERSION")
    );
    println!("Usage:");
    println!("  rstart.exe [command]");
    println!();
    println!("Commands:");
    println!("  tui       Launch the interactive TUI dashboard (default)");
    println!("  list      Search and list all active startup applications");
    println!("  doctor    Verify system registry, log paths, and console scaling");
    println!("  version   Print application version info");
    println!("  help      Print this help message");
}

fn run_doctor() {
    println!("{}", ignite_LOGO);
    println!("rStart Doctor — Diagnostic Report");
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
    match win32::copy_text_to_clipboard("rStart Diagnostic Test Connection") {
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

fn main() -> io::Result<()> {
    // Parse CLI arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "version" | "--version" | "-v" => {
                println!("rstart v{}", env!("CARGO_PKG_VERSION"));
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
            "list" => {
                let items = startup::scan_startup_items();
                if items.is_empty() {
                    println!("No startup items found.");
                } else {
                    println!("{:<30} {:<10} {:<15} {:<12} {}", "Name", "Status", "Type", "Impact", "Command");
                    println!("{}", "-".repeat(100));
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
            "tui" | "--relaunched" => {
                // Proceed to run TUI
            }
            other => {
                if other == "--relaunched" {
                    // Fallback just in case
                } else {
                    eprintln!("Unknown command: {}", other);
                    print_help();
                    std::process::exit(1);
                }
            }
        }
    }

    // Load application configuration
    let config = config::AppConfig::load();
    win32::relaunch_in_conhost_if_needed();

    #[cfg(windows)]
    let _hwnd = win32::hide_console_at_startup();

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
    let _title_guard = ConsoleTitleGuard::new("ignite");

    enable_raw_mode()?;
    let mut stdout = io::stdout();

    // Force scalable minimal size or custom sizing
    let _ = execute!(stdout, crossterm::terminal::SetSize(100, 35));
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Enable borderless console framing immediately after size adjustment if configured
    let _borderless = if config.enable_borderless {
        Some(win32::BorderlessConsole::enable())
    } else {
        None
    };

    // Allow console size/style changes to propagate to the buffer
    std::thread::sleep(Duration::from_millis(50));

    if _borderless.is_none() {
        win32::center_console_window();
    }

    #[cfg(windows)]
    win32::show_console_window();

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
                        log_message("KEYPRESS", &format!("Code: {:?}", key.code));
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

    log_message("EXIT", "Shutting down cleanly.");

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;
    use crate::app::get_theme;
    use crate::ui::utils::{wrap_text, parse_markdown_to_lines};

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
        let mut db = startup::BackupDatabase::default();
        let entry = startup::BackupEntry {
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
            "rstart_test_{}",
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

        let loaded = startup::BackupDatabase::load();
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].name, "Test App");

        if let Some(val) = original_appdata {
            unsafe { std::env::set_var("APPDATA", val); }
        }
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
