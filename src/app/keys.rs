use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::App;
use crate::backend;
use crate::win32;

const README_CONTENT: &str = include_str!("../../README.md");
const SUPPORT_CONTENT: &str = include_str!("../../SUPPORT.md");
const LICENSE_CONTENT: &str = include_str!("../../LICENSE.md");
const COPYRIGHT_CONTENT: &str = include_str!("../../COPYRIGHT.md");
const PRIVACY_CONTENT: &str = include_str!("../../PRIVACY.md");
const SECURITY_CONTENT: &str = include_str!("../../SECURITY.md");
const CONTRIBUTING_CONTENT: &str = include_str!("../../CONTRIBUTING.md");

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Markdown viewer intercept keys
    if app.show_markdown.is_some() {
        // F1..F7 -> swap to a different doc
        if let Some(name) = library::apps::chrome::open_embedded_markdown(key.code) {
            // Validate the name against the canonical DOC_FILES list before
            // matching. This makes the per-app content lookup explicit
            // rather than implicit. The library's `doc()` is the single
            // source of truth for which doc names exist. (Drift fix for B7.)
            let content = if library::apps::chrome::doc(name).is_some() {
                match name {
                    "README.md" => README_CONTENT,
                    "SUPPORT.md" => SUPPORT_CONTENT,
                    "LICENSE.md" => LICENSE_CONTENT,
                    "COPYRIGHT.md" => COPYRIGHT_CONTENT,
                    "PRIVACY.md" => PRIVACY_CONTENT,
                    "SECURITY.md" => SECURITY_CONTENT,
                    "CONTRIBUTING.md" => CONTRIBUTING_CONTENT,
                    _ => "",
                }
            } else {
                ""
            };
            app.open_embedded_markdown(name, content);
            return;
        }
        // Up/Down/PageUp/PageDown -> scroll the markdown
        if let Some(new_scroll) = library::apps::chrome::scroll_for_key(
            key.code,
            app.markdown_scroll,
            app.markdown_lines.len(),
            10,
        ) {
            app.markdown_scroll = new_scroll;
            return;
        }
        // Esc/q close the viewer
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
            app.show_markdown = None;
            app.set_status("Document viewer closed.".to_string());
        }
        return;
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
        return;
    }

    // Backups view intercept keys
    if app.show_backups {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('u') | KeyCode::Char('U') | KeyCode::Char('b') | KeyCode::Char('B') => {
                app.show_backups = false;
                app.set_status("Backups view closed.".to_string());
            }
            KeyCode::Up => {
                if app.selected_backup > 0 {
                    app.selected_backup -= 1;
                }
            }
            KeyCode::Down => {
                if !app.backup_db.entries.is_empty() && app.selected_backup < app.backup_db.entries.len() - 1 {
                    app.selected_backup += 1;
                }
            }
            KeyCode::Enter => {
                if !app.backup_db.entries.is_empty() {
                    let entry = app.backup_db.entries[app.selected_backup].clone();
                    match backend::restore_startup_item(&entry) {
                        Ok(_) => {
                            app.backup_db.entries.remove(app.selected_backup);
                            let _ = app.backup_db.save();
                            app.startup_items = backend::scan_startup_items();
                            app.selected_startup = 0;
                            app.show_backups = false;
                            app.set_status(format!("Successfully restored: {}", entry.name));
                        }
                        Err(e) => {
                            app.set_status(format!("Error restoring: {}", e));
                        }
                    }
                }
            }
            KeyCode::Delete if !app.backup_db.entries.is_empty() => {
                let entry = app.backup_db.entries[app.selected_backup].clone();
                app.backup_db.entries.remove(app.selected_backup);
                let _ = app.backup_db.save();
                app.selected_backup = 0;
                app.set_status(format!("Deleted backup entry: {}", entry.name));
            }
            _ => {}
        }
        return;
    }

    // Standard hotkeys
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            let text = app.get_diagnostic_details_text();
            match win32::copy_text_to_clipboard(&text) {
                Ok(_) => app.set_status(
                    "📋 Copied startup details to Windows Clipboard!"
                        .to_string(),
                ),
                Err(e) => {
                    app.set_status(format!("❌ Clipboard copy failed: {}", e))
                }
            }
        }
        KeyCode::F(1..=7) => {
            // F1..F7 -> embedded docs. Delegated to library's chrome::open_embedded_markdown
            // which returns the filename if the key is an F1..F7 doc key.
            if let Some(name) = library::apps::chrome::open_embedded_markdown(key.code) {
                let content = match name {
                    "README.md" => README_CONTENT,
                    "SUPPORT.md" => SUPPORT_CONTENT,
                    "LICENSE.md" => LICENSE_CONTENT,
                    "COPYRIGHT.md" => COPYRIGHT_CONTENT,
                    "PRIVACY.md" => PRIVACY_CONTENT,
                    "SECURITY.md" => SECURITY_CONTENT,
                    "CONTRIBUTING.md" => CONTRIBUTING_CONTENT,
                    _ => "",
                };
                app.open_embedded_markdown(name, content);
            }
        }
        KeyCode::Char('h') => {
            app.show_help = true;
            app.set_status(
                "Help overlay active. Press ESC/q to close.".to_string(),
            );
        }
        KeyCode::Tab => {
            // Tab focus cycling is disabled since Right Panel is purely informational details
        }
        KeyCode::Up => {
            app.select_prev_startup();
        }
        KeyCode::Down => {
            app.select_next_startup();
        }
        KeyCode::Char(' ') => {
            if !app.startup_items.is_empty() {
                let mut item = app.startup_items[app.selected_startup].clone();
                match backend::toggle_startup_item(&mut item) {
                    Ok(_) => {
                        app.startup_items[app.selected_startup] = item.clone();
                        let state = if item.enabled { "Enabled" } else { "Disabled" };
                        app.set_status(format!("Toggled {}: {}", item.name, state));
                    }
                    Err(e) => {
                        app.set_status(format!("Error toggling item: {}", e));
                    }
                }
            }
        }
        KeyCode::Char('u') | KeyCode::Char('U') | KeyCode::Char('b') | KeyCode::Char('B') => {
            app.backup_db = backend::BackupDatabase::load();
            app.show_backups = true;
            app.selected_backup = 0;
            app.set_status("Backups view active. Press Esc to close, Enter to restore, Del to delete entry.".to_string());
        }
        KeyCode::Delete if !app.startup_items.is_empty() => {
            let item = app.startup_items[app.selected_startup].clone();
            let mut db = backend::BackupDatabase::load();
            if let Err(e) = db.add_item(&item) {
                app.set_status(format!("Failed to backup item: {}", e));
            } else {
                match backend::delete_startup_item(&item) {
                    Ok(_) => {
                        app.startup_items = backend::scan_startup_items();
                        app.selected_startup = 0;
                        app.set_status(format!("Deleted and backed up: {}", item.name));
                    }
                    Err(e) => {
                        app.set_status(format!("Error deleting item: {}", e));
                    }
                }
            }
        }
        _ => {}
    }
}
