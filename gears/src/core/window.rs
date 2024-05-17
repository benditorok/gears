use std::sync::Arc;

use super::prelude::Window;
use winit::{event, event_loop::EventLoop, window};

pub struct GearsWinitWindow {
    window: Arc<winit::window::Window>,
    pub event_loop: Option<winit::event_loop::EventLoop<()>>,
}

impl Window for GearsWinitWindow {
    fn new() -> Self {
        let event_loop = EventLoop::new().expect("Failed to create winit event loop.");
        let title = env!("CARGO_PKG_NAME");
        let window = window::WindowBuilder::new()
            .with_title(title)
            .build(&event_loop)
            .expect("Failed to create winit window.");

        Self {
            window: Arc::new(window),
            event_loop: Some(event_loop),
        }
    }

    fn loop_events(&mut self) {
        if let Some(event_loop) = Option::take(&mut self.event_loop) {
            event_loop.run(move |event, ewlt| match event {
                event::Event::NewEvents(_) => todo!(),
                event::Event::WindowEvent { window_id, event } => todo!(),
                event::Event::DeviceEvent { device_id, event } => todo!(),
                event::Event::UserEvent(_) => todo!(),
                event::Event::Suspended => todo!(),
                event::Event::Resumed => todo!(),
                event::Event::AboutToWait => todo!(),
                event::Event::LoopExiting => todo!(),
                event::Event::MemoryWarning => todo!(),
            });
        }
    }
}
