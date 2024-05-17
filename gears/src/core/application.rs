use std::{thread, time};

pub struct Application {}

impl Application {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self) {
        while true {
            println!("Running application...");
            thread::sleep(time::Duration::from_secs(1));
        }
    }
}
