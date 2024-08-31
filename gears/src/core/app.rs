use super::{event::EventQueue, threadpool::ThreadPool};
use crate::window::{self, winit};
use crate::window::{Window, WindowType};
use env_logger::Env;
use log::info;

pub trait App {
    fn new(window_context_type: WindowType, threads: usize) -> Self;
    async fn run(&mut self);
}

pub struct GearsApp {
    window_context: Option<Box<dyn Window>>,
    thread_pool: ThreadPool,
    event_queue: EventQueue,
}

impl App for GearsApp {
    fn new(window_context_type: WindowType, threads: usize) -> Self {
        // Create window context
        let window_context: Option<Box<dyn Window>> = match window_context_type {
            WindowType::Winit => {
                let ctx = Box::new(window::winit::GearsWinitWindow::new());
                Some(ctx)
            }
            WindowType::Headless => None,
        };

        let mut threads = threads;
        if threads < 4 {
            threads = 4;
        }

        Self {
            window_context,
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

        // Start window
        if let Some(window_context) = self.window_context.as_mut() {
            window_context.start();
        }
    }
}
