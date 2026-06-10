#![allow(deprecated)]
pub use library::clipboard::copy_text_to_clipboard;
pub use library::window::{
    query_cursor_pos, get_window_rect, set_window_pos, relaunch_in_conhost_if_needed,
};
pub use library::sys_info::{
    get_console_window_dpi, get_system_screen_resolution, query_bios_info,
    query_dark_mode, query_os_version, query_power_status, query_shell_and_terminal,
    get_dwm_accent_color, GlyphMap,
};

pub use library::{hide_console_at_startup, show_console_window};
