use gears::core::{
    application::{self, Application},
    window::{self, Window},
};

fn main() {
    let window_context = Box::new(window::GearsWinitWindow::new());
    let mut app = application::GearsApplication::new(window_context, 8);

    pollster::block_on(app.run());
}
