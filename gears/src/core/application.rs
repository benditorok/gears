use super::{threadpool::ThreadPool, window::Window};
use env_logger::Env;
use log::info;
use std::thread::{self, JoinHandle};

pub trait Application {
    fn new(window_context: Box<dyn Window>, threads: usize) -> Self;
    async fn run(&mut self);
}

pub struct GearsApplication {
    window: Option<Box<dyn Window>>,
    thread_pool: ThreadPool,
}

impl Application for GearsApplication {
    fn new(window_context: Box<dyn Window>, threads: usize) -> Self {
        Self {
            window: Some(window_context),
            thread_pool: ThreadPool::new(threads),
        }
    }

    async fn run(&mut self) {
        let env = Env::default()
            .filter_or("MY_LOG_LEVEL", "trace")
            .write_style_or("MY_LOG_STYLE", "always");
        env_logger::init_from_env(env);

        info!("Starting Gears...");

        if let Some(window) = &mut self.window {
            window.handle_events();
        }
    }
}
