use winit::window;

use super::prelude::{Application, Window};

pub struct GearsApplication {
    window: Option<Box<dyn Window>>,
}

impl Application for GearsApplication {
    fn new(window_context: Box<dyn Window>) -> Self {
        Self {
            window: Some(window_context),
        }
    }

    async fn run(&mut self) {
        if let Some(window) = &mut self.window {
            window.handle_events();
        }
    }
}

pub struct ApplicationBuilder {
    application: GearsApplication,
}
