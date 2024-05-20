use gears::core::{
    application::{self, Application},
    window::{self, Window, WindowFactory},
};

fn main() {
    let window_context = WindowFactory::new_winit_window();
    let mut app = application::GearsApplication::new(window_context, 8);

    pollster::block_on(app.run());
}
