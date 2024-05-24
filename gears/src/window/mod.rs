pub mod winit;

pub trait Window {
    fn new() -> Self
    where
        Self: Sized;
    fn start(&mut self);
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn on_update(&mut self);
    fn send_user_event(&mut self);
    fn set_vsync(&mut self, vsync: bool);
    fn is_vsync(&self) -> bool;
}

pub enum WindowType {
    Headless,
    Winit,
}

pub enum WindowContext {
    None,
    Wgpu,
}
