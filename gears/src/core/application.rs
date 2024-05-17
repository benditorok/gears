use winit::window;

use super::prelude::{Application, Window};

pub struct GearsApp {
    window: Option<Box<dyn Window>>,
}

impl Application for GearsApp {
    fn new(window_context: Box<dyn Window>) -> Self {
        Self { window: None }
    }

    async fn run(&mut self) {
        if let Some(window) = &mut self.window {
            window.loop_events();
        }
    }
}

pub struct ApplicationBuilder {
    application: GearsApp,
}
