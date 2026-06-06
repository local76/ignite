#![allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct MONITORINFO {
    pub cbSize: u32,
    pub rcMonitor: RECT,
    pub rcWork: RECT,
    pub dwFlags: u32,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct COORD {
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct SMALL_RECT {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct CONSOLE_SELECTION_INFO {
    pub dwFlags: u32,
    pub dwSelectionAnchor: COORD,
    pub srSelection: SMALL_RECT,
}

#[cfg(windows)]
#[link(name = "user32")]
unsafe extern "system" {
    fn GetSystemMetrics(n_index: i32) -> i32;
    fn GetWindowRect(h_wnd: *mut std::ffi::c_void, lp_rect: *mut RECT) -> i32;
    fn SetWindowLongPtrW(h_wnd: *mut std::ffi::c_void, n_index: i32, dw_new_long: isize) -> isize;
    fn GetWindowLongPtrW(h_wnd: *mut std::ffi::c_void, n_index: i32) -> isize;
    fn SetWindowPos(
        h_wnd: *mut std::ffi::c_void,
        h_wnd_insert_after: *mut std::ffi::c_void,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        u_flags: u32,
    ) -> i32;
    fn GetDpiForWindow(h_wnd: *mut std::ffi::c_void) -> u32;
    fn OpenClipboard(h_wnd_new_owner: *mut std::ffi::c_void) -> i32;
    fn EmptyClipboard() -> i32;
    fn SetClipboardData(u_format: u32, h_mem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn CloseClipboard() -> i32;
    fn MonitorFromWindow(h_wnd: *mut std::ffi::c_void, dw_flags: u32) -> *mut std::ffi::c_void;
    fn GetMonitorInfoW(h_monitor: *mut std::ffi::c_void, lp_mi: *mut MONITORINFO) -> i32;
    fn GetAsyncKeyState(v_key: i32) -> i16;
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetConsoleWindow() -> *mut std::ffi::c_void;
    fn CreateMutexW(
        lp_mutex_attributes: *const std::ffi::c_void,
        b_initial_owner: i32,
        lp_name: *const u16,
    ) -> *mut std::ffi::c_void;
    fn GetLastError() -> u32;
    fn CloseHandle(h_object: *mut std::ffi::c_void) -> i32;
    fn GetConsoleTitleW(lp_console_title: *mut u16, n_size: u32) -> u32;
    fn SetConsoleTitleW(lp_console_title: *const u16) -> i32;
    fn GetLogicalDriveStringsW(n_buffer_length: u32, lp_buffer: *mut u16) -> u32;
    fn GetDiskFreeSpaceExW(
        lp_directory_name: *const u16,
        lp_free_bytes_available_to_caller: *mut u64,
        lp_total_number_of_bytes: *mut u64,
        lp_total_number_of_free_bytes: *mut u64,
    ) -> i32;
    fn GlobalAlloc(u_flags: u32, dw_bytes: usize) -> *mut std::ffi::c_void;
    fn GlobalLock(h_mem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn GlobalUnlock(h_mem: *mut std::ffi::c_void) -> i32;
    fn GlobalFree(h_mem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn GetStdHandle(n_std_handle: u32) -> *mut std::ffi::c_void;
    fn ReadConsoleOutputCharacterW(
        h_console_output: *mut std::ffi::c_void,
        lp_character: *mut u16,
        n_length: u32,
        dw_read_coord: COORD,
        lp_number_of_chars_read: *mut u32,
    ) -> i32;
    fn GetConsoleSelectionInfo(lp_console_selection_info: *mut CONSOLE_SELECTION_INFO) -> i32;
    fn CreateFileW(
        lp_file_name: *const u16,
        dw_desired_access: u32,
        dw_share_mode: u32,
        lp_security_attributes: *const std::ffi::c_void,
        dw_creation_disposition: u32,
        dw_flags_and_attributes: u32,
        h_template_file: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;
}

#[cfg(windows)]
#[link(name = "dwmapi")]
unsafe extern "system" {
    fn DwmGetColorizationColor(pcr_color: *mut u32, pf_opaque_blend: *mut i32) -> i32;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
#[allow(non_snake_case)]
pub struct SERVICE_STATUS {
    pub dwServiceType: u32,
    pub dwCurrentState: u32,
    pub dwControlsAccepted: u32,
    pub dwWin32ExitCode: u32,
    pub dwServiceSpecificExitCode: u32,
    pub dwCheckPoint: u32,
    pub dwWaitHint: u32,
}

#[cfg(windows)]
#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegisterEventSourceW(
        lp_unc_server_name: *const u16,
        lp_source_name: *const u16,
    ) -> *mut std::ffi::c_void;

    fn ReportEventW(
        h_event_log: *mut std::ffi::c_void,
        w_type: u16,
        w_category: u16,
        dw_event_id: u32,
        lp_user_sid: *mut std::ffi::c_void,
        w_num_strings: u16,
        dw_data_size: u32,
        lp_strings: *const *const u16,
        lp_raw_data: *mut std::ffi::c_void,
    ) -> i32;

    fn DeregisterEventSource(h_event_log: *mut std::ffi::c_void) -> i32;

    fn OpenSCManagerW(
        lp_machine_name: *const u16,
        lp_database_name: *const u16,
        dw_desired_access: u32,
    ) -> *mut std::ffi::c_void;

    fn OpenServiceW(
        h_sc_manager: *mut std::ffi::c_void,
        lp_service_name: *const u16,
        dw_desired_access: u32,
    ) -> *mut std::ffi::c_void;

    fn QueryServiceStatus(
        h_service: *mut std::ffi::c_void,
        lp_service_status: *mut SERVICE_STATUS,
    ) -> i32;

    fn CloseServiceHandle(h_sc_object: *mut std::ffi::c_void) -> i32;
}

/// Bounding rect of the console window.
#[allow(dead_code)]
pub fn get_console_rect() -> Option<RECT> {
    #[cfg(windows)]
    {
        let hwnd = unsafe { GetConsoleWindow() };
        if hwnd.is_null() {
            return None;
        }
        let mut r = RECT::default();
        if unsafe { GetWindowRect(hwnd, &mut r) } != 0 {
            Some(r)
        } else {
            None
        }
    }
    #[cfg(not(windows))]
    None
}

/// Ensures only one instance of the TUI application is active at any time.
pub struct SingleInstanceGuard {
    #[allow(dead_code)]
    handle: *mut std::ffi::c_void,
}

impl SingleInstanceGuard {
    pub fn try_new() -> Result<Self, String> {
        #[cfg(windows)]
        {
            let name: Vec<u16> = "Local\\rsta_SingleInstanceMutex_2026\0"
                .encode_utf16()
                .collect();
            let handle = unsafe { CreateMutexW(std::ptr::null(), 1, name.as_ptr()) };
            if handle.is_null() {
                return Err("Failed to create single-instance mutex.".to_string());
            }

            let err = unsafe { GetLastError() };
            if err == 183 {
                // ERROR_ALREADY_EXISTS = 183
                unsafe { CloseHandle(handle) };
                return Err("Another instance of this application is already running.".to_string());
            }

            Ok(SingleInstanceGuard { handle })
        }
        #[cfg(not(windows))]
        {
            Ok(SingleInstanceGuard {
                handle: std::ptr::null_mut(),
            })
        }
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        #[cfg(windows)]
        if !self.handle.is_null() {
            unsafe { CloseHandle(self.handle) };
        }
    }
}

/// Strips standard console headers/borders and centers window dynamically (DPI-aware).
pub struct BorderlessConsole {
    hwnd: *mut std::ffi::c_void,
    original_style: isize,
    original_rect: RECT,
    active: bool,
}

impl BorderlessConsole {
    pub fn enable() -> Self {
        #[cfg(windows)]
        {
            let hwnd = unsafe { GetConsoleWindow() };
            if hwnd.is_null() {
                return BorderlessConsole {
                    hwnd: std::ptr::null_mut(),
                    original_style: 0,
                    original_rect: RECT::default(),
                    active: false,
                };
            }

            let original_style = unsafe { GetWindowLongPtrW(hwnd, -16) }; // GWL_STYLE = -16
            let mut original_rect = RECT::default();
            unsafe {
                GetWindowRect(hwnd, &mut original_rect);
            }

            // Strip border decorations: WS_CAPTION | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU
            let style_mask = 0x00C00000 | 0x00040000 | 0x00020000 | 0x00010000 | 0x00080000;
            let new_style = original_style & !(style_mask as isize);
            unsafe {
                SetWindowLongPtrW(hwnd, -16, new_style);
            }

            let dpi = unsafe { GetDpiForWindow(hwnd) };
            let scale = dpi as f32 / 96.0;
            let width = (780.0 * scale) as i32;
            let height = (520.0 * scale) as i32;

            let mut x = 100;
            let mut y = 100;
            let h_monitor = unsafe { MonitorFromWindow(hwnd, 2) }; // MONITOR_DEFAULTTONEAREST = 2
            if !h_monitor.is_null() {
                let mut mi = MONITORINFO::default();
                mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                if unsafe { GetMonitorInfoW(h_monitor, &mut mi) } != 0 {
                    let monitor_w = mi.rcWork.right - mi.rcWork.left;
                    let monitor_h = mi.rcWork.bottom - mi.rcWork.top;
                    x = mi.rcWork.left + (monitor_w - width) / 2;
                    y = mi.rcWork.top + (monitor_h - height) / 2;
                }
            }

            unsafe {
                SetWindowPos(
                    hwnd,
                    std::ptr::null_mut(),
                    x,
                    y,
                    width,
                    height,
                    0x0020 | 0x0004 | 0x0010, // SWP_FRAMECHANGED | SWP_NOZORDER | SWP_NOACTIVATE
                );
            }

            BorderlessConsole {
                hwnd,
                original_style,
                original_rect,
                active: true,
            }
        }
        #[cfg(not(windows))]
        {
            BorderlessConsole {
                hwnd: std::ptr::null_mut(),
                original_style: 0,
                original_rect: RECT::default(),
                active: false,
            }
        }
    }
}

impl Drop for BorderlessConsole {
    fn drop(&mut self) {
        #[cfg(windows)]
        if self.active && !self.hwnd.is_null() {
            unsafe {
                SetWindowLongPtrW(self.hwnd, -16, self.original_style);
                let width = self.original_rect.right - self.original_rect.left;
                let height = self.original_rect.bottom - self.original_rect.top;
                SetWindowPos(
                    self.hwnd,
                    std::ptr::null_mut(),
                    self.original_rect.left,
                    self.original_rect.top,
                    width,
                    height,
                    0x0020 | 0x0004 | 0x0010, // SWP_FRAMECHANGED | SWP_NOZORDER | SWP_NOACTIVATE
                );
            }
        }
    }
}

/// Center the console window on the primary display or active monitor.
pub fn center_console_window() {
    #[cfg(windows)]
    unsafe {
        let hwnd = GetConsoleWindow();
        if hwnd.is_null() {
            return;
        }

        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect) != 0 {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            let h_monitor = MonitorFromWindow(hwnd, 2); // MONITOR_DEFAULTTONEAREST = 2
            if !h_monitor.is_null() {
                let mut mi = MONITORINFO::default();
                mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                if GetMonitorInfoW(h_monitor, &mut mi) != 0 {
                    let monitor_w = mi.rcWork.right - mi.rcWork.left;
                    let monitor_h = mi.rcWork.bottom - mi.rcWork.top;

                    let x = mi.rcWork.left + (monitor_w - width) / 2;
                    let y = mi.rcWork.top + (monitor_h - height) / 2;

                    SetWindowPos(
                        hwnd,
                        std::ptr::null_mut(),
                        x,
                        y,
                        width,
                        height,
                        0x0001 | 0x0004 | 0x0010, // SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE
                    );
                }
            }
        }
    }
}

pub struct ConsoleTitleGuard {
    original_title: Option<Vec<u16>>,
}

impl ConsoleTitleGuard {
    pub fn new(new_title: &str) -> Self {
        #[cfg(windows)]
        {
            let mut buf = [0u16; 512];
            let len = unsafe { GetConsoleTitleW(buf.as_mut_ptr(), buf.len() as u32) };
            let original_title = if len > 0 {
                Some(buf[..len as usize].to_vec())
            } else {
                None
            };

            let title_w: Vec<u16> = new_title.encode_utf16().chain(std::iter::once(0)).collect();
            unsafe {
                SetConsoleTitleW(title_w.as_ptr());
            }

            ConsoleTitleGuard { original_title }
        }
        #[cfg(not(windows))]
        {
            ConsoleTitleGuard {
                original_title: None,
            }
        }
    }
}

impl Drop for ConsoleTitleGuard {
    fn drop(&mut self) {
        #[cfg(windows)]
        if let Some(ref title) = self.original_title {
            let mut title_null = title.clone();
            title_null.push(0);
            unsafe {
                SetConsoleTitleW(title_null.as_ptr());
            }
        }
    }
}

/// Retrieve dynamic Windows Accent Color using dwmapi.
pub fn get_dwm_accent_color() -> ratatui::style::Color {
    #[cfg(windows)]
    {
        let mut color: u32 = 0;
        let mut opaque: i32 = 0;
        let hr = unsafe { DwmGetColorizationColor(&mut color, &mut opaque) };
        if hr == 0 {
            // ARGB color (0xAARRGGBB) -> extract RGB
            let r = ((color >> 16) & 0xFF) as u8;
            let g = ((color >> 8) & 0xFF) as u8;
            let b = (color & 0xFF) as u8;
            return ratatui::style::Color::Rgb(r, g, b);
        }
    }
    ratatui::style::Color::Rgb(0, 245, 255)
}

/// Query system metrics for layout sizing
pub fn get_system_screen_resolution() -> (i32, i32) {
    #[cfg(windows)]
    {
        let screen_w = unsafe { GetSystemMetrics(0) }; // SM_CXSCREEN
        let screen_h = unsafe { GetSystemMetrics(1) }; // SM_CYSCREEN
        (screen_w, screen_h)
    }
    #[cfg(not(windows))]
    (1920, 1080)
}

/// Query native console window DPI.
pub fn get_console_window_dpi() -> u32 {
    #[cfg(windows)]
    {
        let hwnd = unsafe { GetConsoleWindow() };
        if !hwnd.is_null() {
            unsafe { GetDpiForWindow(hwnd) }
        } else {
            96
        }
    }
    #[cfg(not(windows))]
    96
}

/// Query the Windows OS version and build number.
pub fn query_os_version() -> String {
    #[cfg(windows)]
    {
        let product_name = crate::reg::read_string(
            crate::reg::HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
            "ProductName",
        )
        .unwrap_or_else(|| "Windows".to_string());
        let current_build = crate::reg::read_string(
            crate::reg::HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
            "CurrentBuild",
        )
        .unwrap_or_default();
        let display_version = crate::reg::read_string(
            crate::reg::HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
            "DisplayVersion",
        )
        .unwrap_or_default();

        let mut final_product = product_name;
        if final_product.starts_with("Windows 10") {
            if let Ok(build) = current_build.parse::<u32>() {
                if build >= 22000 {
                    final_product = final_product.replace("Windows 10", "Windows 11");
                }
            }
        }

        let mut parts = vec![final_product];
        if !display_version.is_empty() {
            parts.push(display_version);
        }
        if !current_build.is_empty() {
            parts.push(format!("(Build {})", current_build));
        }
        parts.join(" ")
    }
    #[cfg(not(windows))]
    {
        "Generic OS".to_string()
    }
}

/// Query dynamic dark mode status.
pub fn query_dark_mode() -> bool {
    crate::reg::read_u32(
        crate::reg::HKEY_CURRENT_USER,
        r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
        "AppsUseLightTheme",
    )
    .map(|v| v == 0)
    .unwrap_or(true)
}

#[derive(Debug, Clone, Default)]
pub struct PowerStatus {
    pub ac_online: bool,
    pub battery_percent: u8,
}

/// Query system battery life and charging source using GetSystemPowerStatus.
pub fn query_power_status() -> Option<PowerStatus> {
    #[cfg(windows)]
    {
        #[repr(C)]
        struct SYSTEM_POWER_STATUS {
            ac_line_status: u8,
            battery_flag: u8,
            battery_life_percent: u8,
            system_status_flag: u8,
            battery_life_time: u32,
            battery_full_life_time: u32,
        }

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetSystemPowerStatus(lp_system_power_status: *mut SYSTEM_POWER_STATUS) -> i32;
        }

        let mut status: SYSTEM_POWER_STATUS = unsafe { std::mem::zeroed() };
        if unsafe { GetSystemPowerStatus(&mut status) } != 0 {
            return Some(PowerStatus {
                ac_online: status.ac_line_status == 1,
                battery_percent: status.battery_life_percent,
            });
        }
    }
    None
}

#[derive(Debug, Clone, Default)]
pub struct SystemBiosInfo {
    pub manufacturer: String,
    pub product: String,
    pub model: String,
}

/// Query BIOS system details from registry.
pub fn query_bios_info() -> Option<SystemBiosInfo> {
    #[cfg(windows)]
    {
        use winreg::RegKey;
        use winreg::enums::HKEY_LOCAL_MACHINE;
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let path = r"HARDWARE\DESCRIPTION\System\BIOS";
        if let Ok(key) = hklm.open_subkey(path) {
            let manufacturer = key
                .get_value::<String, _>("SystemManufacturer")
                .unwrap_or_default();
            let product = key
                .get_value::<String, _>("SystemProductName")
                .unwrap_or_default();
            let model = key
                .get_value::<String, _>("BaseBoardProduct")
                .unwrap_or_default();
            return Some(SystemBiosInfo {
                manufacturer: manufacturer.trim().to_string(),
                product: product.trim().to_string(),
                model: model.trim().to_string(),
            });
        }
    }
    None
}

/// Query process hierarchy to detect active Shell and Terminal Emulator.
pub fn query_shell_and_terminal() -> (String, String) {
    let mut shell = "Unknown Shell".to_string();
    let mut terminal = "Unknown Terminal".to_string();

    #[cfg(windows)]
    {
        use sysinfo::System;
        let mut sys = System::new_all();
        sys.refresh_all();

        let mut current_pid = sysinfo::get_current_pid().ok();
        let mut depth = 0;

        while let Some(pid) = current_pid {
            if depth > 12 {
                break;
            }
            if let Some(process) = sys.process(pid) {
                let name = process.name().to_lowercase();
                if shell == "Unknown Shell" {
                    if name.contains("powershell") || name.contains("pwsh") {
                        shell = "PowerShell".to_string();
                    } else if name == "cmd.exe" || name == "cmd" {
                        shell = "CMD".to_string();
                    } else if name.contains("bash") || name.contains("sh") || name.contains("zsh") {
                        shell = name.replace(".exe", "");
                    }
                }

                if terminal == "Unknown Terminal" {
                    if name.contains("windowsterminal") || name == "openconsole.exe" {
                        terminal = "Windows Terminal".to_string();
                    } else if name.contains("code") {
                        terminal = "VS Code Terminal".to_string();
                    } else if name.contains("alacritty") {
                        terminal = "Alacritty".to_string();
                    } else if name.contains("wezterm") {
                        terminal = "WezTerm".to_string();
                    } else if name.contains("conhost") {
                        terminal = "Windows Console Host".to_string();
                    }
                }

                current_pid = process.parent();
                depth += 1;
            } else {
                break;
            }
        }
    }

    (shell, terminal)
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphMap {
    pub status_ok: &'static str,
    pub status_err: &'static str,
    pub info: &'static str,
    pub warning: &'static str,
    pub cpu: &'static str,
    pub gpu: &'static str,
    pub memory: &'static str,
    pub disk: &'static str,
    pub package: &'static str,
    pub battery: &'static str,
    pub shell: &'static str,
    pub terminal: &'static str,
    pub network: &'static str,
}

impl GlyphMap {
    pub fn load() -> Self {
        let (_, terminal) = query_shell_and_terminal();
        if terminal == "Windows Console Host" {
            Self {
                status_ok: "[OK]",
                status_err: "[ERR]",
                info: "[i]",
                warning: "[!]",
                cpu: "[CPU]",
                gpu: "[GPU]",
                memory: "[RAM]",
                disk: "[DISK]",
                package: "[PKG]",
                battery: "[BAT]",
                shell: "[SH]",
                terminal: "[TERM]",
                network: "[NET]",
            }
        } else {
            Self {
                status_ok: "✔️",
                status_err: "❌",
                info: "ℹ️",
                warning: "⚠️",
                cpu: "🧠",
                gpu: "🎮",
                memory: "📟",
                disk: "💾",
                package: "📦",
                battery: "🔋",
                shell: "🐚",
                terminal: "📟",
                network: "🌐",
            }
        }
    }
}

/// Trigger a native Windows Toast Notification using a PowerShell/WinRT shim.
pub fn show_toast_notification(title: &str, message: &str) {
    let script = format!(
        "[void] [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime]; \
		 [void] [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime]; \
		 $el = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); \
		 $el.GetElementsByTagName('text').Item(0).InnerText = '{}'; \
		 $el.GetElementsByTagName('text').Item(1).InnerText = '{}'; \
		 $notifier = [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('rsta'); \
		 $notifier.Show($el)",
        title.replace('\'', "''"),
        message.replace('\'', "''")
    );

    // Spawn powershell in the background to avoid blocking the main TUI thread.
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
        .spawn();
}

/// Write a record directly to the native Windows Event Log under Application.
pub fn log_windows_event(source_name: &str, event_type: u16, event_id: u32, message: &str) {
    #[cfg(windows)]
    unsafe {
        let source_w: Vec<u16> = source_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let handle = RegisterEventSourceW(std::ptr::null(), source_w.as_ptr());
        if !handle.is_null() {
            let message_w: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
            let strings: [*const u16; 1] = [message_w.as_ptr()];

            ReportEventW(
                handle,
                event_type,
                0, // category
                event_id,
                std::ptr::null_mut(), // user sid
                1,                    // num strings
                0,                    // data size
                strings.as_ptr(),
                std::ptr::null_mut(), // raw data
            );
            DeregisterEventSource(handle);
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiskDriveInfo {
    pub path: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
}

/// Query storage capacities and free space of all active logical drives in real-time.
pub fn query_disk_drives() -> Vec<DiskDriveInfo> {
    let mut drives = Vec::new();
    #[cfg(windows)]
    unsafe {
        let mut buffer = [0u16; 512];
        let len = GetLogicalDriveStringsW(buffer.len() as u32, buffer.as_mut_ptr());
        if len > 0 && len < buffer.len() as u32 {
            let mut start = 0;
            for idx in 0..len as usize {
                if buffer[idx] == 0 {
                    if idx > start {
                        let drive_w = &buffer[start..idx];
                        let mut path_null = drive_w.to_vec();
                        path_null.push(0);

                        let mut free_caller = 0u64;
                        let mut total = 0u64;
                        let mut free_total = 0u64;
                        let ok = GetDiskFreeSpaceExW(
                            path_null.as_ptr(),
                            &mut free_caller,
                            &mut total,
                            &mut free_total,
                        );
                        if ok != 0 {
                            if let Ok(path) = String::from_utf16(drive_w) {
                                drives.push(DiskDriveInfo {
                                    path: path.trim_end_matches('\0').to_string(),
                                    total_bytes: total,
                                    free_bytes: free_caller,
                                });
                            }
                        }
                    }
                    start = idx + 1;
                }
            }
        }
    }
    drives
}

/// Query current state (RUNNING, STOPPED, etc.) of a specific Windows Service from SCM.
pub fn query_windows_service_status(service_name: &str) -> String {
    #[cfg(windows)]
    unsafe {
        let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), 0x0001); // SC_MANAGER_CONNECT = 0x0001
        if !scm.is_null() {
            let service_name_w: Vec<u16> = service_name
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let svc = OpenServiceW(scm, service_name_w.as_ptr(), 0x0004); // SERVICE_QUERY_STATUS = 0x0004
            if !svc.is_null() {
                let mut status = SERVICE_STATUS::default();
                let ok = QueryServiceStatus(svc, &mut status);
                CloseServiceHandle(svc);
                CloseServiceHandle(scm);
                if ok != 0 {
                    return match status.dwCurrentState {
                        1 => "STOPPED".to_string(),
                        2 => "START_PENDING".to_string(),
                        3 => "STOP_PENDING".to_string(),
                        4 => "RUNNING".to_string(),
                        5 => "CONTINUE_PENDING".to_string(),
                        6 => "PAUSE_PENDING".to_string(),
                        7 => "PAUSED".to_string(),
                        _ => "UNKNOWN".to_string(),
                    };
                }
            } else {
                CloseServiceHandle(scm);
            }
        }
    }
    "NOT_FOUND".to_string()
}

/// Dynamic platform-independent helper to query the primary host IP address.
pub fn query_local_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

/// Set the system clipboard text using native Win32 APIs.
pub fn copy_text_to_clipboard(text: &str) -> std::io::Result<()> {
    #[cfg(windows)]
    unsafe {
        use std::ptr;
        if OpenClipboard(ptr::null_mut()) == 0 {
            return Err(std::io::Error::last_os_error());
        }
        if EmptyClipboard() == 0 {
            let _ = CloseClipboard();
            return Err(std::io::Error::last_os_error());
        }

        let text_w: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let len = text_w.len() * 2;
        let h_mem = GlobalAlloc(0x0002, len); // GMEM_MOVEABLE = 0x0002
        if h_mem.is_null() {
            let _ = CloseClipboard();
            return Err(std::io::Error::last_os_error());
        }

        let ptr = GlobalLock(h_mem);
        if ptr.is_null() {
            let _ = GlobalFree(h_mem);
            let _ = CloseClipboard();
            return Err(std::io::Error::last_os_error());
        }

        std::ptr::copy_nonoverlapping(text_w.as_ptr(), ptr as *mut u16, text_w.len());
        GlobalUnlock(h_mem);

        if SetClipboardData(13, h_mem).is_null() {
            // CF_UNICODETEXT = 13
            let _ = GlobalFree(h_mem);
            let _ = CloseClipboard();
            return Err(std::io::Error::last_os_error());
        }

        CloseClipboard();
    }
    Ok(())
}

