use crate::monitor::{PowerEventChannel, PowerState};
use std::thread;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
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
  pub fn start_listening(&self) {
    thread::spawn(move || {
      let runtime = tokio::runtime::Runtime::new().unwrap();
      runtime.block_on(async {
        let system_bus_result = zbus::Connection::system().await;
        if system_bus_result.is_err() {
          println!("D-Bus not available");
          return;
        }
        let system_bus = system_bus_result.unwrap();
        let manager_proxy = ManagerProxy::new(&system_bus).await.unwrap();

        let suspend_monitor_result = get_suspend_monitor(&manager_proxy).await;
        if suspend_monitor_result.is_err() {
          println!("Suspend state not available");
          return;
        }
        let (mut prepare_for_shutdown, mut prepare_for_sleep) = suspend_monitor_result.unwrap();

        let lock_monitor_result = get_lock_monitor(system_bus, manager_proxy).await;
        if lock_monitor_result.is_err() {
          println!("Screen lock state not available");
          return;
        }
        let (mut unlock, mut locked_hint) = lock_monitor_result.unwrap();

        let mut handles: Vec<JoinHandle<()>> = vec![];
        handles.push(tokio::spawn(async move {
          while let Some(signal) = prepare_for_shutdown.next().await {
            let args = signal.args().unwrap();
            dbg!(args);
          }
        }));
        handles.push(tokio::spawn(async move {
          while let Some(signal) = prepare_for_sleep.next().await {
            let args = signal.args().unwrap();
            dbg!(args);
          }
        }));
        handles.push(tokio::spawn(async move {
          while let Some(v) = locked_hint.next().await {
            let status = v.get().await.unwrap();
            if status {
              let sender = PowerEventChannel::sender();
              let _ = sender.send(PowerState::ScreenLocked);
            }
          }
        }));
        handles.push(tokio::spawn(async move {
          while unlock.next().await.is_some() {
            let sender = PowerEventChannel::sender();
            let _ = sender.send(PowerState::ScreenUnlocked);
          }
        }));

        for handle in handles {
          let _ = handle.await;
        }
      });
    });
  }
}

async fn get_suspend_monitor<'a>(
  manager_proxy: &ManagerProxy<'a>,
) -> Result<(PrepareForShutdownStream<'a>, PrepareForSleepStream<'a>)> {
  // not yet tested
  let prepare_for_shutdown = manager_proxy.receive_prepare_for_shutdown().await.unwrap();
  // not yet tested
  let prepare_for_sleep = manager_proxy.receive_prepare_for_sleep().await.unwrap();
  Ok((prepare_for_shutdown, prepare_for_sleep))
}

async fn get_lock_monitor(
  system_bus: zbus::Connection,
  manager_proxy: ManagerProxy<'_>,
) -> Result<(UnlockStream<'_>, zbus::PropertyStream<'_, bool>)> {
  let session_proxy = SessionProxy::new(&system_bus).await.unwrap();
  let session_id: String = session_proxy.id().await.unwrap();
  let session_obj_path: OwnedObjectPath = manager_proxy.get_session(&session_id).await.unwrap();

  let login_session_proxy = SessionProxy::builder(&system_bus)
    .path(session_obj_path)
    .unwrap()
    .build()
    .await
    .unwrap();
  let unlock = login_session_proxy.receive_unlock().await.unwrap();
  let locked_hint = login_session_proxy.receive_locked_hint_changed().await;

  Ok((unlock, locked_hint))
}
