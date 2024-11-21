use super::config::{self, Config};
use super::Dt;
use super::{event::EventQueue, threadpool::ThreadPool};
use crate::ecs::traits::Component;
use crate::{ecs, renderer};
use log::{info, warn};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

pub trait App {
    fn new(config: Config) -> Self;
    #[allow(async_fn_in_trait)]
    async fn run(&mut self) -> anyhow::Result<()>;
    fn get_dt_channel(&self) -> Option<broadcast::Receiver<Dt>>;
    fn get_ecs(&self) -> Arc<Mutex<ecs::Manager>>;
    #[allow(async_fn_in_trait)]
    async fn update_loop<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: Fn(Arc<Mutex<ecs::Manager>>, Dt) + Send + Sync + 'static;
    fn add_window(&mut self, window: Box<dyn FnMut(&egui::Context)>);
    // TODO add a create job fn to access the thread pool
}

/// This struct is used to manage the entire application.
/// The application can also be used to create entities, add components, windows etc. to itself.
pub struct GearsApp {
    config: Config,
    ecs: Arc<Mutex<ecs::Manager>>,
    pub thread_pool: ThreadPool,
    event_queue: EventQueue,
    egui_windows: Option<Vec<Box<dyn FnMut(&egui::Context)>>>,
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
    /// Initialize the application.
    /// This will create a new instance of the application with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the application.
    ///
    /// # Returns
    ///
    /// A new instance of the application.
    fn new(config: config::Config) -> Self {
        assert!(config.threadpool_size >= 1);

        let (tx_dt, rx_dt) = broadcast::channel(64);

        Self {
            event_queue: EventQueue::new(),
            thread_pool: ThreadPool::new(config.threadpool_size),
            config,
            ecs: Arc::new(Mutex::new(ecs::Manager::default())),
            egui_windows: None,
            tx_dt: Some(tx_dt),
            rx_dt: Some(rx_dt),
            is_running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Run the application and start the event loop.
    async fn run(&mut self) -> anyhow::Result<()> {
        info!("Starting Gears...");

        let tx = self.tx_dt.take().unwrap();

        // Run the event loop
        renderer::run(Arc::clone(&self.ecs), tx, self.egui_windows.take()).await
    }

    /// Get the delta time channel.
    /// This is used to communicate the delta time between the main thread and the renderer thread.
    fn get_dt_channel(&self) -> Option<broadcast::Receiver<Dt>> {
        self.tx_dt.as_ref().map(|tx| tx.subscribe())
    }

    /// Get a mutable reference to the ecs manager.
    /// This can be used to access the ecs manager from outside the application.
    ///
    /// # Returns
    ///
    /// A mutable reference to the ecs manager.
    fn get_ecs(&self) -> Arc<Mutex<ecs::Manager>> {
        Arc::clone(&self.ecs)
    }

    /// This will create a new async task that will run the given update function on each update.
    /// The function will be passed the ecs manager and the delta time.ű
    /// **The update loop will run until the application is stopped.**
    ///
    /// # Arguments
    ///
    /// * `f` - The function to run on each update.
    async fn update_loop<F>(&self, f: F) -> anyhow::Result<()>
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
                        warn!("Failed to receive: {:?}", e);
                    }
                }
            }

            info!("Update loop stopped...");
        });

        Ok(())
    }

    /// Add a custom window to the app.
    ///
    /// # Arguments
    ///
    /// * `window` - A function that will be called to render the window.
    fn add_window(&mut self, window: Box<dyn FnMut(&egui::Context)>) {
        if let Some(windows) = &mut self.egui_windows {
            windows.push(window);
        } else {
            self.egui_windows = Some(vec![window]);
        }
    }
}

impl GearsApp {
    /// Create a new update job.
    /// This will create a new async task that will run the given update function on each update.
    #[warn(unstable_features)]
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

    fn add_component(&mut self, component: impl Component) -> &mut Self {
        {
            let ecs = self.ecs.lock().unwrap();

            let entity = if let Some(e) = ecs.get_last() {
                e
            } else {
                ecs.create_entity()
            };

            ecs.add_component_to_entity(entity, component);
        }

        self
    }

    fn build(&mut self) -> ecs::Entity {
        let ecs = self.ecs.lock().unwrap();

        if let Some(e) = ecs.get_last() {
            e
        } else {
            ecs.create_entity()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::new_entity;

    use super::*;
    use ecs::traits::EntityBuilder;

    #[derive(Debug, PartialEq)]
    struct TestComponent {
        value: i32,
    }

    impl Component for TestComponent {}

    #[test]
    fn test_entity_builder() {
        let mut app = GearsApp::default();

        let entity = app
            .new_entity()
            .add_component(TestComponent { value: 10 })
            .build();

        let ecs = app.ecs.lock().unwrap();

        let entities = ecs.entity_count();
        assert_eq!(entities, 1);

        let component = ecs
            .get_component_from_entity::<TestComponent>(entity)
            .unwrap();
        assert_eq!(component.read().unwrap().value, 10);
    }

    #[test]
    fn test_new_entity_macro() {
        let mut app = crate::core::app::GearsApp::default();
        let entity = new_entity!(app, TestComponent { value: 10 });

        let ecs = app.ecs.lock().unwrap();

        let entities = ecs.entity_count();
        assert_eq!(entities, 1);

        let component = ecs
            .get_component_from_entity::<TestComponent>(entity)
            .unwrap();
        assert_eq!(component.read().unwrap().value, 10);
    }
}
