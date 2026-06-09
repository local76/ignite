use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use crate::app::ThemeColors;

/// Helper function to center a layout chunk for modal popups.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
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

pub fn wrap_text_aligned(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut result = Vec::new();
    let mut current_line = String::new();

    for word in text.split(' ') {
        let mut w = word;
        while !w.is_empty() {
            let space_left = width.saturating_sub(
                current_line.len() + if current_line.is_empty() { 0 } else { 1 },
            );
            if w.len() <= space_left {
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(w);
                break;
            } else if space_left >= 5 || current_line.is_empty() {
                let chunk_size = if current_line.is_empty() { width } else { space_left };
                let (chunk, rest) = w.split_at(chunk_size.min(w.len()));
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(chunk);
                result.push(current_line);
                current_line = String::new();
                w = rest;
            } else {
                result.push(current_line);
                current_line = String::new();
            }
        }
    }
    if !current_line.is_empty() {
        result.push(current_line);
    }
    if result.is_empty() {
        result.push(String::new());
    }
    result
}

pub fn format_detail_line(
    label: &str,
    value: &str,
    width: usize,
    label_style: Style,
    value_style: Style,
) -> Vec<Line<'static>> {
    let value_width = width.saturating_sub(15);
    let wrapped = wrap_text_aligned(value, value_width);
    let mut lines = Vec::new();

    if let Some(first_val) = wrapped.first() {
        lines.push(Line::from(vec![
            Span::styled(format!("{:<15}", label), label_style),
            Span::styled(first_val.clone(), value_style),
        ]));
    }

    for val in wrapped.iter().skip(1) {
        lines.push(Line::from(vec![
            Span::styled("               ", label_style),
            Span::styled(val.clone(), value_style),
        ]));
    }

    lines
}

pub fn format_help_row(
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

/// A lightweight, custom terminal markdown parser returning styled TUI Spans and Lines.
pub fn parse_markdown_to_lines(content: &str, theme: &ThemeColors) -> Vec<Line<'static>> {
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
