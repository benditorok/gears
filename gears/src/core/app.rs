use super::config::{self, Config, LogConfig, LogLevel};
use super::Dt;
use super::{event::EventQueue, threadpool::ThreadPool};
use crate::{ecs, renderer};
use log::info;
use std::env;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

pub trait App {
    fn new(config: Config) -> Self;
    fn map_ecs(&mut self, ecs: ecs::Manager) -> Arc<Mutex<ecs::Manager>>;
    #[allow(async_fn_in_trait)]
    async fn run(&mut self) -> anyhow::Result<()>;
    fn get_dt_channel(&self) -> Option<broadcast::Receiver<Dt>>;
    // TODO add a create job fn to access the thread pool
}

/// The main application.
pub struct GearsApp {
    config: Config,
    world: Arc<Mutex<ecs::Manager>>,
    pub thread_pool: ThreadPool,
    event_queue: EventQueue,
    tx_dt: Option<broadcast::Sender<Dt>>,
    rx_dt: Option<broadcast::Receiver<Dt>>,
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
    // Initialize the application.
    fn new(config: config::Config) -> Self {
        assert!(config.threadpool_size > 1);

        let (tx_dt, rx_dt) = broadcast::channel(64);

        Self {
            event_queue: EventQueue::new(),
            thread_pool: ThreadPool::new(config.threadpool_size),
            config,
            world: Arc::new(Mutex::new(ecs::Manager::default())),
            tx_dt: Some(tx_dt),
            rx_dt: Some(rx_dt),
        }
    }

    /// Map the world to the app.
    fn map_ecs(&mut self, world: ecs::Manager) -> Arc<Mutex<ecs::Manager>> {
        self.world = Arc::new(Mutex::new(world));
        Arc::clone(&self.world)
    }

    /// Run the application.
    async fn run(&mut self) -> anyhow::Result<()> {
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
        // Filter out specific log messages
        env_builder.filter_module("wgpu_core::device::resource", log::LevelFilter::Warn);
        env_builder.init();

        info!("Starting Gears...");

        let tx = self.tx_dt.take().unwrap();

        // Run the event loop
        renderer::run(Arc::clone(&self.world), tx).await
    }

    /// Get the delta time channel.
    /// This is used to communicate the delta time between the main thread and the renderer thread.
    fn get_dt_channel(&self) -> Option<broadcast::Receiver<Dt>> {
        self.tx_dt.as_ref().map(|tx| tx.subscribe())
    }
}
