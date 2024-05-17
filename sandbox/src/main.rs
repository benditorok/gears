use gears::core::prelude::*;
use gears::core::{application, window};

fn main() {
    let window_context = Box::new(window::GearsWinitWindow::new());
    let mut app = application::GearsApp::new(window_context);

    pollster::block_on(app.run());
}
