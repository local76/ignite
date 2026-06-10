use ratatui::style::Style;
use ratatui::text::{Line, Span};

// Re-export shared implementations from library design system to prevent code duplication.
#[allow(unused_imports)]
pub use library::interface::app::design::prelude::{
    centered_rect, format_help_row, parse_markdown_to_lines, wrap_text,
};

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
