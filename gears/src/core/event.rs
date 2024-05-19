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

pub enum Event {
    WindowEvent(WindowEvent),
    DeviceEvent(DeviceEvent),
    CloseRequest,
}

pub struct EventQueue {
    events: Arc<Mutex<VecDeque<Event>>>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn push(&mut self, event: Event) {
        let mut events = self.events.lock().unwrap();
        events.push_back(event);
    }

    fn pop(&mut self) -> Option<Event> {
        let mut events = self.events.lock().unwrap();
        events.pop_front()
    }
}
