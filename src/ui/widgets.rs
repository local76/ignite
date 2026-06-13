use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::ui::accent_list::AccentList;

use crate::app::{App, ThemeColors};
use crate::ui::text_format::format_detail_line;

pub fn draw_startup_list(f: &mut Frame, app: &App, area: Rect, theme: &ThemeColors) {
    let left_border = theme.accent;
    let left_block = Block::default()
        .borders(Borders::ALL)
        .title(" Startup Applications ")
        .title_style(
            Style::default()
                .fg(left_border)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(left_border));

    let left_inner = left_block.inner(area);
    f.render_widget(left_block, area);

    // Maximum length for startup item names before truncation
    const MAX_NAME_LENGTH: usize = 32;
    // Length to keep when truncating (3 less than max to account for "...")
    const TRUNCATED_NAME_LENGTH: usize = 29;

    // Render startup applications list
    let items_strings: Vec<String> = app.startup_items.iter().map(|item| {
        let status_str = if item.enabled { "Enabled" } else { "Disabled" };
        let type_str = if item.location_type.to_lowercase().contains("user") {
            "User"
        } else {
            "System"
        };
        let name_trimmed = if item.name.len() > MAX_NAME_LENGTH {
            format!("{}...", &item.name[..TRUNCATED_NAME_LENGTH])
        } else {
            item.name.clone()
        };
        format!("{:<35} {:<15} {:<15} {:<15}", name_trimmed, status_str, type_str, item.impact)
    }).collect();
    let items: Vec<&str> = items_strings.iter().map(|s| s.as_str()).collect();

    let accent_list = AccentList::new(
        items,
        app.selected_startup,
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

    // Layout left inner to have headers and separator
    let left_inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Headers
            Constraint::Length(1), // Separator
            Constraint::Min(0),    // List itself
        ])
        .split(left_inner);

    // Render headers
    let headers_line = Line::from(vec![
        Span::styled("   ", Style::default().fg(theme.text_dim)),
        Span::styled(format!("{:<35} {:<15} {:<15} {:<15}", "NAME", "STATUS", "TYPE", "IMPACT"), Style::default().fg(theme.text_dim).add_modifier(Modifier::BOLD)),
    ]);
    f.render_widget(Paragraph::new(headers_line), left_inner_chunks[0]);

    // Render separator under headers
    let header_separator = Line::from(vec![
        Span::styled("   ", Style::default().fg(theme.border)),
        Span::styled(
            "─".repeat((left_inner.width as usize).saturating_sub(3)),
            Style::default().fg(theme.border),
        ),
    ]);
    f.render_widget(Paragraph::new(header_separator), left_inner_chunks[1]);

    // Render the list
    f.render_widget(accent_list, left_inner_chunks[2]);
}

pub fn draw_startup_details(f: &mut Frame, app: &App, area: Rect, theme: &ThemeColors) {
    let right_border = theme.border;
    let right_block = Block::default()
        .borders(Borders::ALL)
        .title(" Startup Application Details ")
        .title_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(right_border));

    let right_inner = right_block.inner(area);
    f.render_widget(right_block, area);

    // Margins inside right box for premium feel
    let right_inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top margin
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Bottom margin
        ])
        .split(right_inner);

    let right_content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2), // Left margin
            Constraint::Min(0),    // Content
            Constraint::Length(2), // Right margin
        ])
        .split(right_inner_chunks[1]);

    let mut details_lines = Vec::new();
    if app.startup_items.is_empty() {
        details_lines.push(Line::from("No startup items detected."));
    } else if let Some(item) = app.startup_items.get(app.selected_startup) {
        let content_width = right_content_layout[1].width as usize;

        details_lines.push(Line::from(vec![
            Span::styled("Name:          ", Style::default().fg(theme.text_dim)),
            Span::styled(&item.name, Style::default().fg(theme.text_main).add_modifier(Modifier::BOLD)),
            Span::styled("   │   ", Style::default().fg(theme.border)),
            Span::styled("Status:        ", Style::default().fg(theme.text_dim)),
            Span::styled(
                if item.enabled { "Enabled" } else { "Disabled" },
                Style::default().fg(if item.enabled { Color::Rgb(0, 255, 127) } else { Color::Rgb(255, 85, 85) }).add_modifier(Modifier::BOLD)
            ),
        ]));
        details_lines.push(Line::from(""));

        details_lines.extend(format_detail_line(
            "Location Type:",
            &item.location_type,
            content_width,
            Style::default().fg(theme.text_dim),
            Style::default().fg(theme.text_main),
        ));
        details_lines.push(Line::from(""));

        details_lines.extend(format_detail_line(
            "Startup Impact:",
            &item.impact,
            content_width,
            Style::default().fg(theme.text_dim),
            Style::default().fg(match item.impact.as_str() {
                "High" => Color::Rgb(255, 85, 85),
                "Medium" => Color::Rgb(255, 215, 0),
                _ => Color::Rgb(0, 255, 127),
            }).add_modifier(Modifier::BOLD),
        ));
        details_lines.push(Line::from(""));

        details_lines.extend(format_detail_line(
            "Registry Path:",
            &item.location_path,
            content_width,
            Style::default().fg(theme.text_dim),
            Style::default().fg(theme.text_main),
        ));
        details_lines.push(Line::from(""));

        details_lines.extend(format_detail_line(
            "Config Key:",
            &item.key_name,
            content_width,
            Style::default().fg(theme.text_dim),
            Style::default().fg(theme.text_main),
        ));
        details_lines.push(Line::from(""));

        details_lines.extend(format_detail_line(
            "Command:",
            &item.command,
            content_width,
            Style::default().fg(theme.text_dim),
            Style::default().fg(theme.text_main),
        ));
    }

    let details_p = Paragraph::new(details_lines)
        .alignment(ratatui::layout::Alignment::Left);

    f.render_widget(details_p, right_content_layout[1]);
}
