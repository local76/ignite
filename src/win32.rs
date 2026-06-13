#![allow(dead_code)]

pub use crate::clipboard::copy_text_to_clipboard;
pub use crate::backend::sys_info::{
    query_dark_mode, query_os_version, query_power_status, GlyphMap,
};
pub use win32_relaunch::relaunch_in_conhost_if_needed;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// Retrieve the bounding rectangle of the console window.
pub fn get_window_rect() -> Option<RECT> {
    #[cfg(target_os = "windows")]
    unsafe {
        let hwnd = windows_sys::Win32::System::Console::GetConsoleWindow();
        if hwnd.is_null() {
            return None;
        }
        let mut rect = RECT::default();
        let ok = windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(
            hwnd,
            &mut rect as *mut RECT as *mut _,
        );
        if ok != 0 {
            Some(rect)
        } else {
            None
        }
    }
    #[cfg(not(target_os = "windows"))]
    None
}

/// Update the position of the console window.
pub fn set_window_pos(_x: i32, _y: i32) {
    #[cfg(target_os = "windows")]
    unsafe {
        let hwnd = windows_sys::Win32::System::Console::GetConsoleWindow();
        if !hwnd.is_null() {
            windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                _x,
                _y,
                0,
                0,
                windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOSIZE
                    | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                    | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
            );
        }
    }
}

/// Queries the mouse cursor position in screen coordinates.
pub fn query_cursor_pos() -> Option<(i32, i32)> {
    #[cfg(target_os = "windows")]
    unsafe {
        let mut pt = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
        if windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut pt) != 0 {
            Some((pt.x, pt.y))
        } else {
            None
        }
    }
    #[cfg(not(target_os = "windows"))]
    None
}

/// Make the console window visible and focus it.
pub fn show_console_window() {
    #[cfg(target_os = "windows")]
    unsafe {
        let hwnd = windows_sys::Win32::System::Console::GetConsoleWindow();
        if !hwnd.is_null() {
            windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                hwnd,
                windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW,
            );
            windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);
        }
    }
}

/// Returns the DWM accent color as a Ratatui style Color.
pub fn get_dwm_accent_color() -> ratatui::style::Color {
    let (r, g, b) = crate::backend::sys_info::query_accent_color();
    ratatui::style::Color::Rgb(r, g, b)
}

#[path = "win32_relaunch.rs"]
mod win32_relaunch;
