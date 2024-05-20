use crate::core::window::{self, WinitWindow};

use super::{
    event::EventQueue,
    threadpool::ThreadPool,
    window::{Window, WindowContextType},
};
use env_logger::Env;
use instant::Duration;
use log::{debug, info};
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub trait Application {
    fn new(window_context_type: WindowContextType, threads: usize) -> Self;
    async fn run(&mut self);
}

pub struct GearsApplication {
    window_context: Option<Arc<Mutex<Box<dyn Window>>>>,
    window_context_type: WindowContextType,
    thread_pool: ThreadPool,
    event_queue: EventQueue,
}

impl Application for GearsApplication {
    fn new(window_context_type: WindowContextType, threads: usize) -> Self {
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

        //let mut window_context: Arc<Mutex<Box<dyn Window>>>;

        // TODO: https://docs.rs/winit/latest/winit/event_loop/struct.EventLoop.html

        match self.window_context_type {
            WindowContextType::Winit => {
                let window_context = Box::new(window::WinitWindow::new());
                self.window_context = Some(Arc::new(Mutex::new(window_context)));

                if let Some(window_context) = &self.window_context {
                    let window_context_ptr = Arc::clone(&window_context);

                    self.thread_pool.execute(move || {
                        window_context_ptr.lock().unwrap().start();
                    });
                }
            }
            WindowContextType::None => (),
        }

        let mut iter: u32 = 0;

        'main: loop {
            iter += 1;
            debug!("In the main loop, iter: {}", iter);

            thread::sleep(Duration::from_millis(1000));
        }
    }
}
