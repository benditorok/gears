use anyhow::Error;
use winit::{event_loop, platform::pump_events};

pub trait Window {
    fn new() -> Self
    where
        Self: Sized;
    fn loop_events(&mut self);
}

pub trait Application {
    fn new(window_context: Box<dyn Window>) -> Self;
    async fn run(&mut self);
}

pub trait Gui {
    fn new() -> Self;
    fn show(&mut self);
    fn hide(&mut self);
}

pub trait Renderer {}
