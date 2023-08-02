#[cfg(target_os = "macos")]
#[path = "macos/power_monitor.rs"]
mod monitor;
#[cfg(target_os = "windows")]
#[path = "windows/power_monitor.rs"]
mod monitor;

pub use monitor::*;
