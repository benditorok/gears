use super::{event::EventQueue, threadpool::ThreadPool, window::Window};
use env_logger::Env;
use log::info;
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};
use winit::window;

pub trait Application {
    fn new(window_context: Box<dyn Window>, threads: usize) -> Self;
    async fn run(&mut self);
}

pub struct GearsApplication {
    window_context: Option<Box<dyn Window>>,
    thread_pool: ThreadPool,
    event_queue: EventQueue,
}

impl Application for GearsApplication {
    fn new(window_context: Box<dyn Window>, threads: usize) -> Self {
        Self {
            window_context: Some(window_context),
            thread_pool: ThreadPool::new(threads),
            event_queue: EventQueue::new(),
        }
    }

    async fn run(&mut self) {
        let env = Env::default()
            .filter_or("MY_LOG_LEVEL", "trace")
            .write_style_or("MY_LOG_STYLE", "always");
        env_logger::init_from_env(env);

        info!("Starting Gears...");

        if let Some(window_context) = &mut self.window_context.take() {
            window_context.handle_events();
        }
    }
}
