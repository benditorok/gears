use gears::{
    core::app::{self, App},
    window,
};

fn main() {
    let mut app = app::GearsApp::new(window::WindowType::Winit, 8);
    futures::executor::block_on(app.run());
}
