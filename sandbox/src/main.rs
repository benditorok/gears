use gears::core::application;
use gears::gears_test;

fn main() {
    gears::gears_test();

    let app = application::Application::new();
    app.run();
}
