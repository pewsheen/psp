use crate::monitor::{PowerEventChannel, PowerState};
use windows::{
  core::Result,
  s,
  Win32::{
    Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM},
    System::{
      LibraryLoader::GetModuleHandleA,
      Power::RegisterPowerSettingNotification,
      RemoteDesktop::{WTSRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION},
      SystemServices::GUID_POWERSCHEME_PERSONALITY,
    },
    UI::WindowsAndMessaging::{
      CreateWindowExA, DefWindowProcA, LoadCursorW, RegisterClassA, CS_HREDRAW, CS_VREDRAW,
      CW_USEDEFAULT, IDC_ARROW, PBT_APMRESUMESUSPEND, PBT_APMSUSPEND, WINDOW_EX_STYLE,
      WM_POWERBROADCAST, WM_WTSSESSION_CHANGE, WNDCLASSA, WS_OVERLAPPEDWINDOW, WTS_SESSION_LOCK,
      WTS_SESSION_UNLOCK,
    },
  },
};

#[allow(dead_code)]
pub struct PowerMonitor {
  hwnd: HWND,
}

impl PowerMonitor {
  pub fn new() -> Self {
    unsafe {
      let hwnd = create_power_events_listener().unwrap();
      Self { hwnd }
    }
  }

  pub fn start_listening(&self) {
    unsafe {
      let _ =
        RegisterPowerSettingNotification(HANDLE(self.hwnd.0), &GUID_POWERSCHEME_PERSONALITY, 0);
      WTSRegisterSessionNotification(self.hwnd, NOTIFY_FOR_THIS_SESSION);
    }
  }
}

impl Default for PowerMonitor {
  fn default() -> Self {
    Self::new()
  }
}

unsafe fn create_power_events_listener() -> Result<HWND> {
  let instance = GetModuleHandleA(None)?;
  debug_assert!(instance.0 != 0);
  let window_class = s!("__psp_event_listener");

  let wc = WNDCLASSA {
    hCursor: LoadCursorW(None, IDC_ARROW)?,
    hInstance: instance,
    lpszClassName: window_class,
    style: CS_HREDRAW | CS_VREDRAW,
    lpfnWndProc: Some(wndproc),
    ..Default::default()
  };

  let atom = RegisterClassA(&wc);
  debug_assert!(atom != 0);

  let hwnd = CreateWindowExA(
    WINDOW_EX_STYLE::default(),
    window_class,
    s!("__psp_dummy_window"),
    WS_OVERLAPPEDWINDOW,
    CW_USEDEFAULT,
    CW_USEDEFAULT,
    CW_USEDEFAULT,
    CW_USEDEFAULT,
    None,
    None,
    instance,
    None,
  );

  // TODO: check hwnd

  Ok(hwnd)
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  unsafe {
    match message {
      WM_POWERBROADCAST => match wparam.0 as u32 {
        PBT_APMRESUMESUSPEND => {
          let sender = PowerEventChannel::sender();
          let _ = sender.send(PowerState::Resume);
        }
        PBT_APMSUSPEND => {
          let sender = PowerEventChannel::sender();
          let _ = sender.send(PowerState::Suspend);
        }
        _ => {}
      },
      WM_WTSSESSION_CHANGE => match wparam.0 as u32 {
        WTS_SESSION_LOCK => {
          let sender = PowerEventChannel::sender();
          let _ = sender.send(PowerState::ScreenLocked);
        }
        WTS_SESSION_UNLOCK => {
          let sender = PowerEventChannel::sender();
          let _ = sender.send(PowerState::ScreenUnlocked);
        }
        _ => {}
      },
      _ => {}
    }
    DefWindowProcA(window, message, wparam, lparam)
  }
}
