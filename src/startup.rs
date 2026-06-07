#[cfg(windows)]
mod startup_win;
#[cfg(windows)]
pub use startup_win::*;

#[cfg(not(windows))]
mod startup_mock;
#[cfg(not(windows))]
pub use startup_mock::*;
