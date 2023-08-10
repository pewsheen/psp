use crate::monitor::{PowerEventChannel, PowerState};
use std::thread;
use tokio::task::JoinHandle;
use zbus::{dbus_proxy, zvariant::OwnedObjectPath, Result};

#[dbus_proxy(
  default_service = "org.freedesktop.login1",
  default_path = "/org/freedesktop/login1",
  interface = "org.freedesktop.login1.Manager"
)]
trait Manager {
  /// PrepareForShutdown signal
  #[dbus_proxy(signal)]
  fn prepare_for_shutdown(&self, start: bool) -> zbus::Result<()>;
  /// PrepareForSleep signal
  #[dbus_proxy(signal)]
  fn prepare_for_sleep(&self, start: bool) -> zbus::Result<()>;
  /// GetSession method
  fn get_session(&self, session_id: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[dbus_proxy(
  interface = "org.freedesktop.login1.Session",
  default_service = "org.freedesktop.login1",
  default_path = "/org/freedesktop/login1/session/auto"
)]
trait Session {
  #[dbus_proxy(signal)]
  fn unlock(&self) -> zbus::Result<()>;

  #[dbus_proxy(property)]
  fn locked_hint(&self) -> Result<bool>;
  #[dbus_proxy(property)]
  fn id(&self) -> zbus::Result<String>;
}

#[allow(dead_code)]
pub struct PowerMonitor {}

impl PowerMonitor {
  pub fn new() -> Self {
    Self {}
  }
}

impl PowerMonitor {
  pub fn start_listening(&self) -> std::result::Result<(), &'static str> {
    if !is_unity() {
      return Err("Desktop Environment dosen't support Unity");
    }

    let system_bus_result = zbus::blocking::Connection::system();
    if system_bus_result.is_err() {
      return Err("D-Bus not available");
    }

    let system_bus = system_bus_result.unwrap();
    let manager_proxy = ManagerProxyBlocking::new(&system_bus).unwrap();

    let suspend_monitor_result = get_suspend_monitor(&manager_proxy);
    if suspend_monitor_result.is_err() {
      return Err("Suspend state not available");
    }
    let (mut prepare_for_shutdown, mut prepare_for_sleep) = suspend_monitor_result.unwrap();

    let lock_monitor_result = get_lock_monitor(system_bus, manager_proxy);
    if lock_monitor_result.is_err() {
      return Err("Screen lock state not available");
    }
    let (mut unlock, mut locked_hint) = lock_monitor_result.unwrap();

    thread::spawn(move || {
      let runtime = tokio::runtime::Runtime::new().unwrap();
      runtime.block_on(async {
        let mut handles: Vec<JoinHandle<()>> = vec![];
        handles.push(tokio::spawn(async move {
          while let Some(signal) = prepare_for_shutdown.next() {
            let args = signal.args().unwrap();
            dbg!(args);
          }
        }));
        handles.push(tokio::spawn(async move {
          while let Some(signal) = prepare_for_sleep.next() {
            let args = signal.args().unwrap();
            dbg!(args);
          }
        }));
        handles.push(tokio::spawn(async move {
          while let Some(v) = locked_hint.next() {
            let status = v.get().unwrap();
            if status {
              let sender = PowerEventChannel::sender();
              let _ = sender.send(PowerState::ScreenLocked);
            }
          }
        }));
        handles.push(tokio::spawn(async move {
          while unlock.next().is_some() {
            let sender = PowerEventChannel::sender();
            let _ = sender.send(PowerState::ScreenUnlocked);
          }
        }));

        for handle in handles {
          let _ = handle.await;
        }
      });
    });

    Ok(())
  }
}

fn is_unity() -> bool {
  std::env::var("XDG_CURRENT_DESKTOP")
    .map(|d| {
      let d = d.to_lowercase();
      d.contains("unity") || d.contains("gnome")
    })
    .unwrap_or(false)
}

fn get_suspend_monitor<'a>(
  manager_proxy: &ManagerProxyBlocking<'a>,
) -> Result<(PrepareForShutdownIterator<'a>, PrepareForSleepIterator<'a>)> {
  // not yet tested
  let prepare_for_shutdown = manager_proxy.receive_prepare_for_shutdown().unwrap();
  // not yet tested
  let prepare_for_sleep = manager_proxy.receive_prepare_for_sleep().unwrap();
  Ok((prepare_for_shutdown, prepare_for_sleep))
}

fn get_lock_monitor(
  system_bus: zbus::blocking::Connection,
  manager_proxy: ManagerProxyBlocking<'_>,
) -> Result<(
  UnlockIterator<'_>,
  zbus::blocking::PropertyIterator<'_, bool>,
)> {
  let session_proxy = SessionProxyBlocking::new(&system_bus).unwrap();
  let session_id: String = session_proxy.id().unwrap();
  let session_obj_path: OwnedObjectPath = manager_proxy.get_session(&session_id).unwrap();

  let login_session_proxy = SessionProxyBlocking::builder(&system_bus)
    .path(session_obj_path)
    .unwrap()
    .build()
    .unwrap();
  let unlock = login_session_proxy.receive_unlock().unwrap();
  let locked_hint = login_session_proxy.receive_locked_hint_changed();

  Ok((unlock, locked_hint))
}
