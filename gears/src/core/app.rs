use super::config::{self, Config, LogConfig, LogLevel};
use super::Dt;
use super::{event::EventQueue, threadpool::ThreadPool};
use crate::ecs::Entity;
use crate::{ecs, renderer};
use log::info;
use std::env;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
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
    ecs: Arc<Mutex<ecs::Manager>>,
    pub thread_pool: ThreadPool,
    event_queue: EventQueue,
    tx_dt: Option<broadcast::Sender<Dt>>,
    rx_dt: Option<broadcast::Receiver<Dt>>,
    is_running: Arc<AtomicBool>,
}

impl Default for GearsApp {
    fn default() -> Self {
        GearsApp::new(Config::default())
    }
}

impl App for GearsApp {
    // Initialize the application.
    fn new(config: config::Config) -> Self {
        assert!(config.threadpool_size >= 1);

        let (tx_dt, rx_dt) = broadcast::channel(64);

        Self {
            event_queue: EventQueue::new(),
            thread_pool: ThreadPool::new(config.threadpool_size),
            config,
            ecs: Arc::new(Mutex::new(ecs::Manager::default())),
            tx_dt: Some(tx_dt),
            rx_dt: Some(rx_dt),
            is_running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Map the world to the app.
    fn map_ecs(&mut self, world: ecs::Manager) -> Arc<Mutex<ecs::Manager>> {
        self.ecs = Arc::new(Mutex::new(world));
        Arc::clone(&self.ecs)
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
        renderer::run(Arc::clone(&self.ecs), tx).await
    }

    /// Get the delta time channel.
    /// This is used to communicate the delta time between the main thread and the renderer thread.
    fn get_dt_channel(&self) -> Option<broadcast::Receiver<Dt>> {
        self.tx_dt.as_ref().map(|tx| tx.subscribe())
    }
}

impl GearsApp {
    /// Create a new update job.
    pub async fn update_loop_async<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: Fn(Arc<Mutex<ecs::Manager>>, Dt) -> Pin<Box<dyn Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
    {
        let mut rx_dt = self
            .get_dt_channel()
            .ok_or_else(|| anyhow::anyhow!("No dt channel exists"))?;

        let ecs = Arc::clone(&self.ecs);
        let is_running = Arc::clone(&self.is_running);

        tokio::spawn(async move {
            while is_running.load(std::sync::atomic::Ordering::Relaxed) {
                match rx_dt.recv().await {
                    Ok(dt) => {
                        f(Arc::clone(&ecs), dt).await;
                    }
                    Err(e) => {
                        eprintln!("Failed to receive: {:?}", e);
                    }
                }
            }

            info!("Update loop stopped...");
        });

        Ok(())
    }

    /// Create a new update job.
    pub async fn update_loop<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: Fn(Arc<Mutex<ecs::Manager>>, Dt) + Send + Sync + 'static,
    {
        let mut rx_dt = self
            .get_dt_channel()
            .ok_or_else(|| anyhow::anyhow!("No dt channel exists"))?;

        let ecs = Arc::clone(&self.ecs);
        let is_running = Arc::clone(&self.is_running);

        tokio::spawn(async move {
            while is_running.load(std::sync::atomic::Ordering::Relaxed) {
                match rx_dt.recv().await {
                    Ok(dt) => f(Arc::clone(&ecs), dt),
                    Err(e) => {
                        eprintln!("Failed to receive: {:?}", e);
                    }
                }
            }

            info!("Update loop stopped...");
        });

        Ok(())
    }
}

impl Drop for GearsApp {
    fn drop(&mut self) {
        self.is_running
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

impl ecs::traits::EntityBuilder for GearsApp {
    fn new_entity(&mut self) -> &mut Self {
        self.ecs.lock().unwrap().create_entity();

        self
    }

    fn add_component<T: 'static + Send + Sync>(&mut self, component: T) -> &mut Self {
        {
            let ecs = self.ecs.lock().unwrap();
            ecs.add_component_to_entity(ecs.get_last(), component);
        }

        self
    }

    fn build(&mut self) -> ecs::Entity {
        self.ecs.lock().unwrap().get_last()
    }
}
