pub mod identity;
pub mod sys_info;
pub mod shell_terminal;
pub mod monitors;
pub mod registry;
pub mod config;

pub use shell_terminal::query_shell_and_terminal;
pub use sys_info::{
    query_dark_mode, query_power_status, query_os_version,
    PowerStatus, GlyphMap,
};

#[cfg(target_os = "windows")]
mod startup {
    pub mod win32;
    pub mod backup_win;
}

#[cfg(target_os = "windows")]
pub use startup::win32::*;
#[cfg(target_os = "windows")]
pub use startup::backup_win::*;

#[cfg(not(target_os = "windows"))]
mod startup {
    pub mod stub;
}

#[cfg(not(target_os = "windows"))]
pub use startup::stub::*;


pub mod sysinfo_shim;
