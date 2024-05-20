use log::debug;
use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalSize, Size},
    event::{self, DeviceEvent, DeviceId, Event, KeyEvent, WindowEvent},
    event_loop::{self, ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{self, WindowAttributes, WindowId},
};

pub trait Window: Send {
    fn new() -> Self
    where
        Self: Sized;
    fn start(&mut self);
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn on_update(&mut self);
    fn set_event_callback(&mut self);
    fn set_vsync(&mut self, vsync: bool);
    fn is_vsync(&self) -> bool;
}

struct MyUserEvent {}

impl MyUserEvent {}

#[derive(Default)]
struct WinitWindowState {
    window: Option<winit::window::Window>,
    counter: i32,
}

impl ApplicationHandler for WinitWindowState {
    // This is a common indicator that you can create a window.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attr = WindowAttributes::default()
            .with_inner_size(Size::Physical(PhysicalSize::new(800, 600)))
            .with_resizable(true)
            .with_title("gears winit 0.30.0");

        self.window = Some(event_loop.create_window(attr).unwrap());

        debug!("window created");
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // `unwrap` is fine, the window will always be available when
        // receiving a window event.
        let window = self.window.as_ref().unwrap();
        // Handle window event.
    }
    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        // Handle window event.
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
            self.counter += 1;
            debug!("draw counter: {}", self.counter);
        }
    }
}

impl ApplicationHandler<MyUserEvent> for WinitWindowState {
    fn user_event(&mut self, event_loop: &ActiveEventLoop, user_event: MyUserEvent) {
        // Handle user event.
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Your application got resumed.
        self.window = Some(
            event_loop
                .create_window(WindowAttributes::default())
                .unwrap(),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // `unwrap` is fine, the window will always be available when
        // receiving a window event.
        let window = self.window.as_ref().unwrap();
        // Handle window event.
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        // Handle device event.
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
            self.counter += 1;
        }
    }
}

pub struct GearsWinitWindow {
    window_state: WinitWindowState,
    event_loop: Option<winit::event_loop::EventLoop<()>>,
}

unsafe impl Send for GearsWinitWindow {}

impl Window for GearsWinitWindow {
    fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        /*
            let window = event_loop
            .create_window(WindowAttributes::default())
            .unwrap();
        let state = WinitWindowState {
            window: Some(window),
            counter: 0,
        };
        */

        let state = WinitWindowState::default();

        Self {
            window_state: state,
            event_loop: Some(event_loop),
        }
    }

    #[allow(unused)]
    fn start(&mut self) {
        if let Some(event_loop) = self.event_loop.take() {
            event_loop.run_app(&mut self.window_state);
        }
    }

    fn get_width(&self) -> u32 {
        todo!()
    }

    fn get_height(&self) -> u32 {
        todo!()
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
