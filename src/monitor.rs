use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::OnceLock;

use crate::platform_impl;

#[derive(Debug, Clone, Copy)]
pub enum PowerState {
  Unknown,
  Suspend,
  Resume,
  ScreenLocked,
  ScreenUnlocked,
}

static STATE_CHANNEL: OnceLock<(Sender<PowerState>, Receiver<PowerState>)> =
  OnceLock::<(Sender<PowerState>, Receiver<PowerState>)>::new();

pub struct PowerEventChannel {}

impl PowerEventChannel {
  pub fn receiver() -> Receiver<PowerState> {
    let (_, rx) = STATE_CHANNEL.get_or_init(unbounded);
    rx.clone()
  }

  pub fn sender() -> Sender<PowerState> {
    let (tx, _) = STATE_CHANNEL.get_or_init(unbounded);
    tx.clone()
  }
}

pub struct PowerMonitor {}

impl PowerMonitor {
  pub fn new() -> Self {
    platform_impl::PowerMonitor::new();
    Self {}
  }

  pub fn event_receiver(&self) -> Receiver<PowerState> {
    PowerEventChannel::receiver()
  }
}

impl Default for PowerMonitor {
  fn default() -> Self {
    Self::new()
  }
}
