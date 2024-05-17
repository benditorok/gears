use std::sync::Arc;

use super::prelude::Window;
use winit::{
    event::{self, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window,
};

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

    #[allow(unused)]
    fn handle_events(&mut self) {
        let window = Arc::clone(&self.window);

        if let Some(event_loop) = Option::take(&mut self.event_loop) {
            event_loop.run(move |event, ewlt| match event {
                Event::DeviceEvent { .. } => (),
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    logical_key: Key::Named(NamedKey::Escape),
                                    ..
                                },
                            ..
                        } => ewlt.exit(),
                        WindowEvent::Resized(physical_size) => {}
                        WindowEvent::RedrawRequested => {}
                        _ => {}
                    };
                }
                _ => {}
            });
        }
    }
}
