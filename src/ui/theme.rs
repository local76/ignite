//! Theme coloring utility and factory for ratatui-based TUIs.

use ratatui::style::Color;

/// Theme color definitions for styling console panels and text.
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub border: Color,
    pub text_main: Color,
    pub text_dim: Color,
    pub accent: Color,
}

/// Factory function to retrieve light or dark theme presets.
pub fn get_theme(dark: bool, accent_color: Color) -> ThemeColors {
    if dark {
        ThemeColors {
            border: Color::Rgb(68, 68, 84),
            text_main: Color::Rgb(248, 248, 242),
            text_dim: Color::Rgb(136, 136, 153),
            accent: accent_color,
        }
    } else {
        ThemeColors {
            border: Color::Rgb(180, 180, 190),
            text_main: Color::Rgb(40, 42, 54),
            text_dim: Color::Rgb(100, 100, 115),
            accent: accent_color,
        }
    }
}
