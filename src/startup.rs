#[cfg(windows)]
pub mod startup_win;
#[cfg(windows)]
pub use startup_win::*;

#[cfg(windows)]
pub mod backup_win;
#[cfg(windows)]
pub use backup_win::*;

#[cfg(not(windows))]
pub mod startup_mock;
#[cfg(not(windows))]
pub use startup_mock::*;
