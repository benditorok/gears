use crate::core::{
    window::{self},
};

use super::{
    event::EventQueue,
    threadpool::ThreadPool,
    window::{Window, WindowType},
};
use env_logger::Env;
use log::{info};


pub trait Application {
    fn new(window_context_type: WindowType, threads: usize) -> Self;
    async fn run(&mut self);
}

pub struct GearsApplication {
    window_context: Option<Box<dyn Window>>,
    window_context_type: WindowType,
    thread_pool: ThreadPool,
    event_queue: EventQueue,
}

impl Application for GearsApplication {
    fn new(window_context_type: WindowType, threads: usize) -> Self {
        Self {
            window_context: None,
            window_context_type,
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

        match self.window_context_type {
            WindowType::Winit => {
                let window_context = Box::new(window::GearsWinitWindow::new());
                self.window_context = Some(window_context);
            }
            WindowType::None => (),
        }

        if let Some(window_context) = self.window_context.as_mut() {
            window_context.start();
        }
    }
}
