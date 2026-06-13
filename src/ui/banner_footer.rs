use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use crate::app::{App, ThemeColors};

pub fn draw_title_banner(f: &mut Frame, app: &mut App, area: Rect, theme: &ThemeColors) {
    let username = &app.username;
    let host_name = &app.host_name;
    let os_str_val = app.os_version.clone();

    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " Rust Startup Manager ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let ver_str = format!(" ignite v{} ", env!("CARGO_PKG_VERSION"));
    let user_host_str = format!("{}@{}", username, host_name);

    let button_y = area.y + 1;
    let inner_width = area.width.saturating_sub(2) as usize;
    let left_len = ver_str.len() + 3 + user_host_str.len() + 3 + os_str_val.len();
    let right_len = 6 + 3 + 6;

    let title_line = if inner_width > left_len + right_len {
        let padding_len = inner_width - (left_len + right_len);
        let padding_str = " ".repeat(padding_len);

        let help_offset = 1 + left_len + padding_len;
        let help_start_x = area.x + help_offset as u16;
        let help_end_x = help_start_x + 6;
        app.help_btn_bounds = Some((button_y, help_start_x, help_end_x));

        let quit_offset = help_offset + 6 + 3;
        let quit_start_x = area.x + quit_offset as u16;
        let quit_end_x = quit_start_x + 6;
        app.quit_btn_bounds = Some((button_y, quit_start_x, quit_end_x));

        Line::from(vec![
            Span::styled(ver_str, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(user_host_str, Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(os_str_val, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(padding_str, Style::default()),
            // Help button: " help"
            Span::styled(" ", Style::default().bg(Color::Rgb(250, 210, 50)).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled("h", Style::default().bg(Color::Rgb(250, 210, 50)).fg(Color::Black).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled("elp ", Style::default().bg(Color::Rgb(250, 210, 50)).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            // Quit button: " quit"
            Span::styled(" ", Style::default().bg(Color::Rgb(255, 85, 85)).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("q", Style::default().bg(Color::Rgb(255, 85, 85)).fg(Color::White).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled("uit ", Style::default().bg(Color::Rgb(255, 85, 85)).fg(Color::White).add_modifier(Modifier::BOLD)),
        ])
    } else {
        let help_offset = 1 + ver_str.len() + 3 + user_host_str.len() + 3 + os_str_val.len() + 3;
        let help_start_x = area.x + help_offset as u16;
        let help_end_x = help_start_x + 6;
        app.help_btn_bounds = Some((button_y, help_start_x, help_end_x));

        let quit_offset = help_offset + 6 + 3;
        let quit_start_x = area.x + quit_offset as u16;
        let quit_end_x = quit_start_x + 6;
        app.quit_btn_bounds = Some((button_y, quit_start_x, quit_end_x));

        Line::from(vec![
            Span::styled(ver_str, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(user_host_str, Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(os_str_val, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            // Help button
            Span::styled(" ", Style::default().bg(Color::Rgb(250, 210, 50)).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled("h", Style::default().bg(Color::Rgb(250, 210, 50)).fg(Color::Black).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled("elp ", Style::default().bg(Color::Rgb(250, 210, 50)).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            // Quit button
            Span::styled(" ", Style::default().bg(Color::Rgb(255, 85, 85)).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("q", Style::default().bg(Color::Rgb(255, 85, 85)).fg(Color::White).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled("uit ", Style::default().bg(Color::Rgb(255, 85, 85)).fg(Color::White).add_modifier(Modifier::BOLD)),
        ])
    };

    f.render_widget(Paragraph::new(title_line).block(title_block), area);
}

pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect, theme: &ThemeColors) {
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " Status ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let footer_inner = footer_block.inner(area);
    f.render_widget(footer_block, area);

    let is_default_msg = app.status_msg
        == "Use arrow keys to browse startup entries. Press Space to toggle, Delete to remove. (h for help)";
    let (text_color, status_text) = if is_default_msg {
        (theme.text_dim, app.status_msg.clone())
    } else {
        let lower = app.status_msg.to_lowercase();
        let color = if lower.contains("failed") || lower.contains("error") {
            Color::Rgb(255, 85, 85)
        } else {
            theme.accent
        };
        (color, app.status_msg.clone())
    };

    let footer_p = Paragraph::new(Line::from(vec![Span::styled(
        status_text,
        Style::default().fg(text_color).add_modifier(Modifier::BOLD),
    )]));
    f.render_widget(footer_p, footer_inner);
}
