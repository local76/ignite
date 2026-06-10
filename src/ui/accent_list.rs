use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Block, Widget},
};
use library::ui::colors::AccentColors;

/// A custom selection list highlighted with the Windows DWM accent color.
/// Supports a `focused` flag so the bright accent bullet + bold is only used
/// when the containing panel has focus. When !focused the current item is
/// still indicated (dim bullet + normal text color) so the "current" selection
/// remains visible even in inactive panels. This makes AccentList first-class
/// for tab/focus-based UIs.
#[derive(Debug, Clone)]
pub struct AccentList<'a> {
    pub items: Vec<&'a str>,
    pub selected_index: usize,
    pub accent_color: Color,
    pub dim_color: Color,
    pub active_text_color: Color,
    pub bullet_char: &'a str,
    pub focused: bool,
}

impl<'a> AccentList<'a> {
    pub fn new(
        items: Vec<&'a str>,
        selected_index: usize,
        accent_color: Color,
        dim_color: Color,
        active_text_color: Color,
        bullet_char: &'a str,
        focused: bool,
    ) -> Self {
        Self {
            items,
            selected_index,
            accent_color,
            dim_color,
            active_text_color,
            bullet_char,
            focused,
        }
    }

    /// First-class constructor using the bundled `AccentColors` (recommended).
    pub fn new_with_colors(
        items: Vec<&'a str>,
        selected_index: usize,
        colors: &AccentColors,
        bullet_char: &'a str,
        focused: bool,
    ) -> Self {
        Self::new(items, selected_index, colors.accent, colors.dim, colors.text, bullet_char, focused)
    }
}

impl<'a> Widget for AccentList<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = Block::default();
        let inner_area = block.inner(area);
        if inner_area.height == 0 {
            return;
        }

        let height = inner_area.height as usize;
        let start_index = if self.items.len() <= height {
            0
        } else {
            if self.selected_index < height / 2 {
                0
            } else if self.selected_index + height / 2 >= self.items.len() {
                self.items.len() - height
            } else {
                self.selected_index - height / 2
            }
        };

        let mut lines = Vec::new();
        for idx in start_index..std::cmp::min(self.items.len(), start_index + height) {
            let item = &self.items[idx];
            if idx == self.selected_index {
                let bullet_style = if self.focused {
                    Style::default()
                        .fg(self.accent_color)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.dim_color)
                };
                let item_style = if self.focused {
                    Style::default()
                        .fg(self.active_text_color)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.active_text_color)
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" {} ", self.bullet_char),
                        bullet_style,
                    ),
                    Span::styled(*item, item_style),
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
