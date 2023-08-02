use psp::monitor::PowerMonitor;
use tao::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
};

fn main() {
  let event_loop = EventLoop::new();

  let window = WindowBuilder::new()
    .with_title("Window")
    .build(&event_loop)
    .unwrap();

  let monitor = PowerMonitor::new();
  let power_event_channel = monitor.event_receiver();

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => *control_flow = ControlFlow::Exit,
      Event::MainEventsCleared => {
        window.request_redraw();
      }
      _ => (),
    }

    if let Ok(event) = power_event_channel.try_recv() {
      println!("{event:?}");
    }
  })
}
