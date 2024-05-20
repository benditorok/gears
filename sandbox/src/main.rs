use gears::core::{
    application::{self, Application},
    window::{self, Window, WindowFactory},
};

fn main() {
    let mut app = application::GearsApplication::new(window::WindowContextType::Winit, 8);

    pollster::block_on(app.run());
}
