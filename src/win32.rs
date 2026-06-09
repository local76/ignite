#![allow(deprecated)]
pub use library::clipboard::copy_text_to_clipboard;
pub use library::window::{
    center_console_window, query_cursor_pos, get_window_rect, set_window_pos,
    BorderlessConsole, ConsoleTitleGuard, SingleInstanceGuard, relaunch_in_conhost_if_needed,
};
pub use library::sys_info::{
    get_console_window_dpi, get_system_screen_resolution, query_bios_info,
    query_dark_mode, query_os_version, query_power_status, query_shell_and_terminal,
    get_dwm_accent_color, GlyphMap,
};

/// Hide the console window early at startup (common pattern for TUI apps).
/// Returns the hwnd if successful (for potential later restore).
#[cfg(windows)]
pub fn hide_console_at_startup() -> Option<*mut std::ffi::c_void> {
    unsafe extern "system" {
        fn GetConsoleWindow() -> *mut std::ffi::c_void;
        fn ShowWindow(hWnd: *mut std::ffi::c_void, nCmdShow: i32) -> i32;
    }
    unsafe {
        let h = GetConsoleWindow();
        if !h.is_null() {
            ShowWindow(h, 0); // SW_HIDE = 0
            Some(h)
        } else {
            None
        }
    }
}

/// Re-show the console window after TUI init (parity with helm/scout).
#[cfg(windows)]
pub fn show_console_window() {
    unsafe extern "system" {
        fn ShowWindow(hWnd: *mut std::ffi::c_void, nCmdShow: i32) -> i32;
        fn SetForegroundWindow(hWnd: *mut std::ffi::c_void) -> i32;
    }
    let hwnd = hide_console_at_startup().unwrap_or(std::ptr::null_mut());
    if !hwnd.is_null() {
        unsafe {
            ShowWindow(hwnd, 5); // SW_SHOW = 5
            SetForegroundWindow(hwnd);
        }
    }
}
