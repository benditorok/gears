use futures::executor::block_on;
use gears::{
    core::app::{self, App},
    window,
};
use sandbox::run_sample_code;

fn main() {
    run_sample_code();

    // let mut app = app::GearsApp::default();
    // block_on(app.run());
}
