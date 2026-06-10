use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::Block;
use crate::app::App;
use crate::win32;
use library::interface::tui::design::prelude::centered_rect;

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    let (term_w, term_h) = crossterm::terminal::size().unwrap_or((100, 35));
    if term_w < 100 || term_h < 35 {
        return;
    }
    let size = Rect::new(0, 0, term_w, term_h);

    // Reconstruct UI layouts
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title Banner
            Constraint::Min(10),   // Content panels
            Constraint::Length(3), // Status Bar
        ])
        .split(size);

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(14)])
        .split(chunks[1]);

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let mut clicked_btn = false;

            // Check Quit button
            if let Some((btn_y, btn_start, btn_end)) = app.quit_btn_bounds {
                if mouse.row == btn_y && mouse.column >= btn_start && mouse.column < btn_end {
                    app.should_quit = true;
                    clicked_btn = true;
                }
            }

            // Check Help button
            if !clicked_btn {
                if let Some((btn_y, btn_start, btn_end)) = app.help_btn_bounds {
                    if mouse.row == btn_y && mouse.column >= btn_start && mouse.column < btn_end {
                        app.show_help = !app.show_help;
                        app.set_status(if app.show_help {
                            "Help overlay active. Press ESC/q to close.".to_string()
                        } else {
                            "Help overlay closed.".to_string()
                        });
                        clicked_btn = true;
                    }
                }
            }

            // Check Click inside Backups Modal list
            if !clicked_btn && app.show_backups && !app.backup_db.entries.is_empty() {
                let area = centered_rect(80, 75, size);
                let popup_block = Block::default().borders(ratatui::widgets::Borders::ALL);
                let inner_area = popup_block.inner(area);
                let left_inner_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1), // Headers
                        Constraint::Length(1), // Separator
                        Constraint::Min(0),    // List itself
                    ])
                    .split(inner_area);
                let list_area = left_inner_chunks[2];

                let is_inside_list = mouse.column >= list_area.x
                    && mouse.column < list_area.x + list_area.width
                    && mouse.row >= list_area.y
                    && mouse.row < list_area.y + list_area.height;

                if is_inside_list {
                    let clicked_row = (mouse.row - list_area.y) as usize;
                    if clicked_row < app.backup_db.entries.len() {
                        app.selected_backup = clicked_row;
                    }
                    clicked_btn = true;
                }
            }

            // Check Click inside Startup Applications List
            if !clicked_btn && !app.show_help && app.show_markdown.is_none() && !app.show_backups {
                let left_block = Block::default().borders(ratatui::widgets::Borders::ALL);
                let left_inner = left_block.inner(content_chunks[0]);
                let left_inner_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1), // Headers
                        Constraint::Length(1), // Separator
                        Constraint::Min(0),    // List itself
                    ])
                    .split(left_inner);
                let list_area = left_inner_chunks[2];

                let is_inside_list = mouse.column >= list_area.x
                    && mouse.column < list_area.x + list_area.width
                    && mouse.row >= list_area.y
                    && mouse.row < list_area.y + list_area.height;

                if is_inside_list {
                    let clicked_row = (mouse.row - list_area.y) as usize;
                    if clicked_row < app.startup_items.len() {
                        app.selected_startup = clicked_row;
                        app.focus = crate::app::FocusedSection::LeftPanel;
                    }
                    clicked_btn = true;
                }
            }

            if !clicked_btn {
                if mouse.row <= 2 {
                    if let Some(cursor_pos) = win32::query_cursor_pos() {
                        if let Some(rect) = win32::get_window_rect() {
                            app.drag_active = true;
                            app.drag_start_cursor = Some(cursor_pos);
                            app.drag_start_window = Some((rect.left, rect.top));
                        }
                    }
                } else {
                    app.selection_start = Some((mouse.column, mouse.row));
                    app.selection_end = Some((mouse.column, mouse.row));
                    app.selection_pending_copy = false;
                }
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if app.drag_active {
                if let (Some(start_cursor), Some(start_window)) = (app.drag_start_cursor, app.drag_start_window) {
                    if let Some(curr_cursor) = win32::query_cursor_pos() {
                        let dx = curr_cursor.0 - start_cursor.0;
                        let dy = curr_cursor.1 - start_cursor.1;
                        win32::set_window_pos(start_window.0 + dx, start_window.1 + dy);
                    }
                }
            } else if app.selection_start.is_some() {
                app.selection_end = Some((mouse.column, mouse.row));
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            if app.drag_active {
                app.drag_active = false;
                app.drag_start_cursor = None;
                app.drag_start_window = None;
            } else if let (Some(start), Some(end)) = (app.selection_start, app.selection_end) {
                let dx = (start.0 as i32 - end.0 as i32).abs();
                let dy = (start.1 as i32 - end.1 as i32).abs();

                // Set copy only if drag is greater than 1 horizontal cell or 0 vertical cells (micro-jitter guard)
                if dx > 1 || dy > 0 {
                    app.selection_pending_copy = true;
                } else {
                    app.selection_start = None;
                    app.selection_end = None;
                }
            }
        }
        MouseEventKind::ScrollUp => {
            if app.show_markdown.is_some() {
                app.markdown_scroll = app.markdown_scroll.saturating_sub(3);
            }
        }
        MouseEventKind::ScrollDown => {
            if app.show_markdown.is_some() {
                let inner_h = ((term_h * 80) / 100).saturating_sub(2) as usize;
                let max_scroll = app.markdown_lines.len().saturating_sub(inner_h);
                if app.markdown_scroll < max_scroll {
                    app.markdown_scroll = (app.markdown_scroll + 3).min(max_scroll);
                }
            }
        }
        _ => {}
    }
}
