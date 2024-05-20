use log::debug;
use std::sync::{Arc, Mutex};
use winit::{
    event::{self, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window,
};

pub trait Window: Send {
    fn new() -> Self
    where
        Self: Sized;
    fn handle_events(&mut self);
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn on_update(&mut self);
    fn set_event_callback(&mut self);
    fn set_vsync(&mut self, vsync: bool);
    fn is_vsync(&self) -> bool;
}

pub struct GearsWinitWindow {
    window: Arc<Mutex<winit::window::Window>>,
    event_loop: Option<winit::event_loop::EventLoop<()>>,
}

unsafe impl Send for GearsWinitWindow {}

impl Window for GearsWinitWindow {
    fn new() -> Self {
        let event_loop = EventLoop::new().expect("Failed to create winit event loop.");
        let title = env!("CARGO_PKG_NAME");
        let window = window::WindowBuilder::new()
            .with_title(title)
            .build(&event_loop)
            .expect("Failed to create winit window.");

        Self {
            window: Arc::new(Mutex::new(window)),
            event_loop: Some(event_loop),
        }
    }

    #[allow(unused)]
    fn handle_events(&mut self) {
        let window = Arc::clone(&self.window);
        let mut event_iter: u32 = 0;

        if let Some(event_loop) = Option::take(&mut self.event_loop) {
            event_loop.run(move |event, ewlt| {
                event_iter += 1;
                debug!("Event iter: {}", event_iter);
                let mut window = window.lock().unwrap();

                // TODO: Thread panick on the 7th iteration when calling when calling 'GetMessageW'
                // on line 327 of winit::event_loop.rs; when resize requested?
                match event {
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
                            } => {
                                // Send close event
                                // Wait for response
                                ewlt.exit()
                            }
                            WindowEvent::Resized(physical_size) => {}
                            WindowEvent::RedrawRequested => {}
                            _ => {}
                        };
                    }
                    _ => {}
                };
            });
        }
    }
    /*
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
                        } => {
                            // Send close event
                            // Wait for response
                            ewlt.exit()
                        }
                        WindowEvent::Resized(physical_size) => {}
                        WindowEvent::RedrawRequested => {}
                        _ => {}
                    };
                }
                _ => {}
            });
        }*/

    fn get_width(&self) -> u32 {
        self.window.lock().unwrap().inner_size().width
    }

    fn get_height(&self) -> u32 {
        self.window.lock().unwrap().inner_size().height
    }

    fn on_update(&mut self) {
        todo!()
    }

    fn set_event_callback(&mut self) {
        todo!()
    }

    fn set_vsync(&mut self, vsync: bool) {}

    fn is_vsync(&self) -> bool {
        todo!()
    }
}

pub struct WindowFactory {}

impl WindowFactory {
    pub fn new_winit_window() -> Box<GearsWinitWindow> {
        Box::new(GearsWinitWindow::new())
    }
}
