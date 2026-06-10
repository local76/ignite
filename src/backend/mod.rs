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
