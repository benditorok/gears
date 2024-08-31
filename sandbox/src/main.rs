use futures::executor::block_on;
use gears::{
    core::app::{self, App},
    window,
};

fn main() {
    let mut app = app::GearsApp::default();
    block_on(app.run());
}
