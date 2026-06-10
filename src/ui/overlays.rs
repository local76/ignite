use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Clear, Wrap};
use library::widgets::AccentList;

use crate::app::{App, ThemeColors};
use crate::ui::text_format::{centered_rect, format_help_row};

pub fn draw_help_overlay(f: &mut Frame, app: &mut App, theme: &ThemeColors) {
    let size = f.area();
    let area = centered_rect(65, 70, size);
    let popup_block = Block::default()
        .title(" Keyboard Shortcuts & App Commands ")
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
        "Up / Down",
        "Browse startup entries",
        max_desc_width,
        theme,
    ));
    help_text.extend(format_help_row(
        "Space",
        "Toggle enabled/disabled status of item",
        max_desc_width,
        theme,
    ));
    help_text.extend(format_help_row(
        "Delete",
        "Remove selected startup item",
        max_desc_width,
        theme,
    ));
    help_text.extend(format_help_row(
        "Esc / q",
        "Close dialogs / Help Overlay, or Quit application",
        max_desc_width,
        theme,
    ));
    help_text.extend(format_help_row(
        "h",
        "Toggle this help shortcut overlay modal",
        max_desc_width,
        theme,
    ));
    help_text.extend(format_help_row(
        "u / b",
        "Open Restore backups panel",
        max_desc_width,
        theme,
    ));
    help_text.extend(format_help_row(
        "c",
        "Copy active startup application details to Windows Clipboard",
        max_desc_width,
        theme,
    ));

    help_text.push(Line::from(""));
    help_text.extend(format_help_row(
        "F1",
        "View README.md document",
        max_desc_width,
        theme,
    ));
    help_text.extend(format_help_row(
        "F2",
        "View SUPPORT.md document",
        max_desc_width,
        theme,
    ));
    help_text.extend(format_help_row(
        "F3",
        "View LICENSE.md document",
        max_desc_width,
        theme,
    ));
    help_text.extend(format_help_row(
        "F4",
        "View COPYRIGHT.md document",
        max_desc_width,
        theme,
    ));
    help_text.extend(format_help_row(
        "F5",
        "View PRIVACY.md document",
        max_desc_width,
        theme,
    ));
    help_text.extend(format_help_row(
        "F6",
        "View SECURITY.md document",
        max_desc_width,
        theme,
    ));
    help_text.extend(format_help_row(
        "F7",
        "View CONTRIBUTING.md document",
        max_desc_width,
        theme,
    ));

    help_text.push(Line::from(""));
    help_text.extend(format_help_row(
        "CLI Subcommands",
        "ignite.exe [tui | doctor | version | help]",
        max_desc_width,
        theme,
    ));

    help_text.push(Line::from(""));
    help_text.extend(format_help_row(
        "Terminal Sync",
        &format!(
            "Running in {} via {}",
            app.glyphs.terminal, app.glyphs.shell
        ),
        max_desc_width,
        theme,
    ));
    help_text.extend(format_help_row(
        "Glyphs Status",
        &format!(
            "Config Sync Status {}  Logger Status {}",
            app.glyphs.status_ok, app.glyphs.status_ok
        ),
        max_desc_width,
        theme,
    ));

    f.render_widget(Clear, area);
    let paragraph = Paragraph::new(help_text).block(popup_block);
    f.render_widget(paragraph, area);
}

pub fn draw_markdown_viewer(f: &mut Frame, app: &App, theme: &ThemeColors) {
    if let Some(ref filename) = app.show_markdown {
        let size = f.area();
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

        let paragraph = Paragraph::new(app.markdown_lines.clone())
            .block(popup_block)
            .wrap(Wrap { trim: true })
            .alignment(ratatui::layout::Alignment::Left)
            .scroll((app.markdown_scroll as u16, 0));

        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }
}

pub fn draw_backups_panel(f: &mut Frame, app: &App, theme: &ThemeColors) {
    let size = f.area();
    let area = centered_rect(80, 75, size);
    let popup_block = Block::default()
        .title(Span::styled(
            " Restore Deleted Startup Applications (Esc: Close, Del: Purge, Enter: Restore) ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent));

    f.render_widget(Clear, area);

    let inner_area = popup_block.inner(area);
    f.render_widget(popup_block, area);

    if app.backup_db.entries.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No backups available.",
                Style::default().fg(theme.text_dim),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Deleted items will automatically be saved to backups.json.",
                Style::default().fg(theme.text_dim),
            )),
        ];
        let p = Paragraph::new(empty_text)
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(p, inner_area);
    } else {
        let left_inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Headers
                Constraint::Length(1), // Separator
                Constraint::Min(0),    // List itself
            ])
            .split(inner_area);

        // Render headers
        let headers_line = Line::from(vec![
            Span::styled("   ", Style::default().fg(theme.text_dim)),
            Span::styled(format!("{:<30} {:<28} {:<15}", "NAME", "DELETED TIMESTAMP", "LOCATION TYPE"), Style::default().fg(theme.text_dim).add_modifier(Modifier::BOLD)),
        ]);
        f.render_widget(Paragraph::new(headers_line), left_inner_chunks[0]);

        // Render separator
        let header_separator = Line::from(vec![
            Span::styled("   ", Style::default().fg(theme.border)),
            Span::styled(
                "─".repeat((inner_area.width as usize).saturating_sub(3)),
                Style::default().fg(theme.border),
            ),
        ]);
        f.render_widget(Paragraph::new(header_separator), left_inner_chunks[1]);

        let items_strings: Vec<String> = app.backup_db.entries.iter().map(|entry| {
            let name_trimmed = if entry.name.len() > 28 {
                format!("{}...", &entry.name[..25])
            } else {
                entry.name.clone()
            };
            let time_trimmed = if entry.timestamp.len() > 25 {
                &entry.timestamp[..25]
            } else {
                &entry.timestamp
            };
            format!("{:<30} {:<28} {:<15}", name_trimmed, time_trimmed, entry.location_type)
        }).collect();
        let items: Vec<&str> = items_strings.iter().map(|s| s.as_str()).collect();

        let accent_list = AccentList::new(
            items,
            app.selected_backup,
            theme.accent,
            theme.text_dim,
            theme.text_main,
            if app.glyphs.status_ok == "[OK]" {
                ">"
            } else {
                "▶"
            },
            true,
        );
        f.render_widget(accent_list, left_inner_chunks[2]);
    }
}
