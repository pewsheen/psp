use psp::monitor::PowerMonitor;
use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
};

fn main() {
  let event_loop = EventLoop::new();
  let window = WindowBuilder::new().build(&event_loop).unwrap();

  let power_monitor = PowerMonitor::new();
  let power_event_channel = power_monitor.event_receiver();
  if let Err(msg) = power_monitor.start_listening() {
    println!("Failed to start listening to power events: {}", msg);
    return;
  }

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        window_id,
      } if window_id == window.id() => *control_flow = ControlFlow::Exit,
      _ => (),
    }

    if let Ok(event) = power_event_channel.try_recv() {
      println!("{event:?}");
    }
  });
}
