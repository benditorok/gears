use gears::core::application::{self, Application};
use gears::window::window;

fn main() {
    let mut app = application::GearsApplication::new(window::WindowType::Winit, 8);
    pollster::block_on(app.run());
}
