use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Clear};

use crate::app::{App, get_theme};

pub mod overlays;
pub mod widgets;
pub mod utils;

pub use utils::{centered_rect, parse_markdown_to_lines};

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    let size = f.area();
    let theme = get_theme(app.dark_mode, app.accent_color);

    // 0. Terminal Size Layout Guard
    if size.width < 100 || size.height < 35 {
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
            Line::from("  Minimum Required Size: 100x35"),
            Line::from(""),
            Line::from(
                "  Please resize or maximize your terminal window to resume standard rendering.",
            ),
        ];
        let p = Paragraph::new(text)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

        let area = centered_rect(80, 50, size);
        f.render_widget(Clear, area);
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

    // 1. Draw Title Banner
    widgets::draw_title_banner(f, app, chunks[0], &theme);

    // 2. Main Content splitting vertically
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(14)])
        .split(chunks[1]);

    // Draw widgets
    widgets::draw_startup_list(f, app, content_chunks[0], &theme);
    widgets::draw_startup_details(f, app, content_chunks[1], &theme);

    // 3. Status Bar Footer
    widgets::draw_status_bar(f, app, chunks[2], &theme);

    // 4. Help Overlay Modal
    if app.show_help {
        overlays::draw_help_overlay(f, app, &theme);
    }

    // 5. Scrollable Markdown Document Viewer Modal
    if app.show_markdown.is_some() {
        overlays::draw_markdown_viewer(f, app, &theme);
    }

    // 5.5. Restore Backups Modal
    if app.show_backups {
        overlays::draw_backups_panel(f, app, &theme);
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

        // Draw Highlight
        for y in 0..height {
            for x in 0..width {
                if is_selected(x, y) {
                    let cell = &mut buf[(x, y)];
                    cell.set_bg(Color::Rgb(0, 120, 215));
                    cell.set_fg(Color::White);
                }
            }
        }

        // Perform Copy on Release
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
                let _ = crate::win32::copy_text_to_clipboard(&selected_text);
                let truncated = if selected_text.len() > 30 {
                    format!("{}...", &selected_text[..27].replace('\n', " "))
                } else {
                    selected_text.replace('\n', " ")
                };
                app.status_msg = format!("📋 Copied selection to clipboard: {}", truncated);
                app.status_timer = Some(std::time::Instant::now());
            }

            app.selection_start = None;
            app.selection_end = None;
            app.selection_pending_copy = false;
        }
    }
}
