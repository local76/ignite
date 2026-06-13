use ratatui::Frame;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Clear, Wrap};
use crate::app::{App, ThemeColors};
use crate::ui::text_format::centered_rect;

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
