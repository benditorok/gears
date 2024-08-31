use super::config::{self, Config, LogConfig, LogLevel};
use super::{event::EventQueue, threadpool::ThreadPool};
use crate::renderer::state;
use crate::window::{self, Window, WindowType};
use env_logger::Env;
use log::{info, Log};

pub trait App {
    fn new(config: Config) -> Self;
    async fn run(&mut self);
}

pub struct GearsApp {
    config: Config,
    thread_pool: ThreadPool,
    event_queue: EventQueue,
}

impl Default for GearsApp {
    fn default() -> Self {
        let config = Config {
            log: LogConfig {
                level: LogLevel::Info,
            },
            threadpool_size: 8,
        };

        GearsApp::new(config)
    }
}

impl App for GearsApp {
    fn new(config: config::Config) -> Self {
        assert!(config.threadpool_size > 1);

        Self {
            event_queue: EventQueue::new(),
            thread_pool: ThreadPool::new(config.threadpool_size),
            config,
        }
    }

    async fn run(&mut self) {
        // Initialize logger
        let mut env_builder = env_logger::Builder::new();
        // Set the minimum log level from the config.
        env_builder.filter_level(match self.config.log.level {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        });
        env_builder.init();

        info!("Starting Gears...");

        // Run the event loop
        state::run().await;
    }
}
