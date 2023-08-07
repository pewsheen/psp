#[cfg(target_os = "macos")]
#[path = "macos/power_monitor.rs"]
mod monitor;
#[cfg(target_os = "windows")]
#[path = "windows/power_monitor.rs"]
mod monitor;

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
#[path = "linux/power_monitor.rs"]
mod monitor;

pub use monitor::*;
