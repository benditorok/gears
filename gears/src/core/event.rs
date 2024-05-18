pub enum WindowEvent {
    Resize,
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
}
