use std::{
    collections::VecDeque,
    fmt::Debug,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub enum WindowEvent {
    Resize(u32, u32), // width, height
    Update,
    Redraw,
}

#[derive(Debug)]
pub enum DeviceEvent {
    MouseMotion,
    MouseWheel,
    KeyboardInput,
}

#[derive(Debug)]
pub enum GearsEvent {
    WindowEvent(WindowEvent),
    DeviceEvent(DeviceEvent),
    CustomEvent,
    UserEvent,
    CloseRequest,
}

pub struct EventQueue {
    events: Arc<Mutex<VecDeque<GearsEvent>>>,
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn add_event(&mut self, event: GearsEvent) {
        let mut events = &mut self.events.lock().unwrap();
        events.push_back(event);
    }

    pub fn remove_event(&mut self) -> Option<GearsEvent> {
        let mut events = &mut self.events.lock().unwrap();
        events.pop_front()
    }
}
