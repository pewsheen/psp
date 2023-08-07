use crate::monitor::{PowerEventChannel, PowerState};

#[allow(dead_code)]
pub struct PowerMonitor {
  system_bus: zbus::blocking::Connection,
  session_bus: zbus::blocking::Connection,
}

impl PowerMonitor {
  pub fn new() -> Self {
    let system_bus = zbus::blocking::Connection::system().unwrap();
    let session_bus = zbus::blocking::Connection::session().unwrap();

    Self {
      system_bus,
      session_bus,
    }
  }
}
