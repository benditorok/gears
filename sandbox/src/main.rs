use futures::executor::block_on;
use gears::{
    core::app::{self, App},
    window,
};
use sandbox::ecs_test;

fn main() {
    // ecs_test();

    let mut app = app::GearsApp::default();
    block_on(app.run());
}
