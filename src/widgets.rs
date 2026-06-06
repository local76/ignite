use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

/// A custom progress gauge styled with the system DWM accent color.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AccentGauge {
    pub progress: f64, // 0.0 to 1.0
    pub label: String,
    pub accent_color: Color,
    pub dim_color: Color,
    pub use_unicode: bool,
}

#[allow(dead_code)]
impl AccentGauge {
    pub fn new(
        progress: f64,
        label: &str,
        accent_color: Color,
        dim_color: Color,
        use_unicode: bool,
    ) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            label: label.to_string(),
            accent_color,
            dim_color,
            use_unicode,
        }
    }
}

impl Widget for AccentGauge {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        if area.height < 1 {
            return;
        }

        let total_width = area.width as usize;
        if total_width < 4 {
            return;
        }

        // Calculate filled characters
        let bar_width = total_width - 2; // Subtract brackets
        let filled_chars = (self.progress * bar_width as f64).round() as usize;

        let fill_symbol = if self.use_unicode { "█" } else { "#" };
        let empty_symbol = if self.use_unicode { "░" } else { "-" };

        let mut spans = vec![Span::styled("[", Style::default().fg(self.dim_color))];

        // Filled portion
        for _ in 0..filled_chars {
            spans.push(Span::styled(
                fill_symbol,
                Style::default().fg(self.accent_color),
            ));
        }
        // Empty portion
        for _ in filled_chars..bar_width {
            spans.push(Span::styled(
                empty_symbol,
                Style::default().fg(self.dim_color),
            ));
        }

        spans.push(Span::styled("]", Style::default().fg(self.dim_color)));

        // Add percentage overlay text in center of the gauge block
        let pct_text = format!(" {:.0}% ", self.progress * 100.0);
        let label_text = if !self.label.is_empty() {
            format!(" {} -{}", self.label, pct_text)
        } else {
            pct_text
        };

        let label_len = label_text.len();
        if label_len < total_width {
            let start_pos = (total_width - label_len) / 2;
            // Superimpose label characters on top of spans
            let mut char_idx = 0;
            for span in spans.iter_mut() {
                let content = span.content.as_ref();
                let content_len = content.chars().count();
                if char_idx >= start_pos && char_idx < start_pos + label_len {
                    // Need to override this segment
                    let override_start = char_idx - start_pos;
                    let override_char = label_text.chars().nth(override_start).unwrap_or(' ');
                    *span = Span::styled(
                        override_char.to_string(),
                        span.style.add_modifier(Modifier::BOLD),
                    );
                }
                char_idx += content_len;
            }
        }

        let line = Line::from(spans);
        line.render(area, buf);
    }
}

/// A custom selection list highlighted with the Windows DWM accent color.
#[derive(Debug, Clone)]
pub struct AccentList<'a> {
    pub items: Vec<&'a str>,
    pub selected_index: usize,
    pub accent_color: Color,
    pub dim_color: Color,
    pub active_text_color: Color,
    pub bullet_char: &'a str,
}

impl<'a> AccentList<'a> {
    pub fn new(
        items: Vec<&'a str>,
        selected_index: usize,
        accent_color: Color,
        dim_color: Color,
        active_text_color: Color,
        bullet_char: &'a str,
    ) -> Self {
        Self {
            items,
            selected_index,
            accent_color,
            dim_color,
            active_text_color,
            bullet_char,
        }
    }
}

impl<'a> Widget for AccentList<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = Block::default();
        let inner_area = block.inner(area);

        let mut lines = Vec::new();
        for (idx, item) in self.items.iter().enumerate() {
            if idx >= inner_area.height as usize {
                break;
            }

            if idx == self.selected_index {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" {} ", self.bullet_char),
                        Style::default()
                            .fg(self.accent_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        *item,
                        Style::default()
                            .fg(self.active_text_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::styled("   ", Style::default().fg(self.dim_color)),
                    Span::styled(*item, Style::default().fg(self.dim_color)),
                ]));
            }
        }

        Paragraph::new(lines).render(inner_area, buf);
    }
}

