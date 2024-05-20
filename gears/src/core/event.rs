use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

pub enum WindowEvent {
    Resize(u32, u32), // width, height
    Update,
    Redraw,
}

pub enum DeviceEvent {
    MouseMotion,
    MouseWheel,
    KeyboardInput,
}

pub enum GearsEvent {
    WindowEvent(WindowEvent),
    DeviceEvent(DeviceEvent),
    CustomEvent,
    CloseRequest,
}

pub struct EventQueue {
    events: Arc<Mutex<VecDeque<GearsEvent>>>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn push(&mut self, event: GearsEvent) {
        let mut events = self.events.lock().unwrap();
        events.push_back(event);
    }

    fn pop(&mut self) -> Option<GearsEvent> {
        let mut events = self.events.lock().unwrap();
        events.pop_front()
    }
}
