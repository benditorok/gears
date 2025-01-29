use super::config::{self, Config};
use super::threadpool::ThreadPool;
use super::Dt;
use crate::ecs::Component;
use crate::ecs::World;
use crate::{ecs, state::State};
use log::{info, warn};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time;
use tokio::sync::broadcast;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowAttributes;

// This struct is used to manage the entire application.
/// The application can also be used to create entities, add components, windows etc. to itself.
pub struct GearsApp {
    config: Config,
    world: Arc<ecs::World>,
    pub thread_pool: ThreadPool,
    egui_windows: Option<Vec<Box<dyn FnMut(&egui::Context)>>>,
    tx_dt: Option<tokio::sync::broadcast::Sender<Dt>>,
    rx_dt: Option<tokio::sync::broadcast::Receiver<Dt>>,
    is_running: Arc<AtomicBool>,
}

impl Default for GearsApp {
    fn default() -> Self {
        GearsApp::new(Config::default())
    }
}

impl GearsApp {
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
    pub fn new(config: config::Config) -> Self {
        assert!(config.threadpool_size >= 1);

        let (tx_dt, rx_dt) = broadcast::channel(64);

        Self {
            thread_pool: ThreadPool::new(config.threadpool_size),
            config,
            world: Arc::new(ecs::World::default()),
            egui_windows: None,
            tx_dt: Some(tx_dt),
            rx_dt: Some(rx_dt),
            is_running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Run the application and start the event loop.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        info!("Starting Gears...");

        let tx = self.tx_dt.take().unwrap();

        // Run the event loop
        GearsApp::run_engine(Arc::clone(&self.world), tx, self.egui_windows.take()).await
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
    pub fn get_ecs(&self) -> Arc<World> {
        Arc::clone(&self.world)
    }

    /// This will create a new async task that will run the given update function on each update.
    /// The function will be passed the ecs manager and the delta time.ű
    /// **The update loop will run until the application is stopped.**
    ///
    /// # Arguments
    ///
    /// * `f` - The function to run on each update.
    pub async fn update_loop<F>(&self, f: F) -> anyhow::Result<()>
    where
        F: Fn(Arc<ecs::World>, Dt) + Send + Sync + 'static,
    {
        let mut rx_dt = self
            .get_dt_channel()
            .ok_or_else(|| anyhow::anyhow!("No dt reciever channel exists"))?;

        // let mut rx_event = self
        //     .get_event_channel()
        //     .ok_or_else(|| anyhow::anyhow!("No event reciever channel exists"))?;

        let world = Arc::clone(&self.world);
        let is_running = Arc::clone(&self.is_running);

        tokio::spawn(async move {
            while is_running.load(std::sync::atomic::Ordering::Relaxed) {
                match rx_dt.recv().await {
                    Ok(dt) => f(Arc::clone(&world), dt),
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
    pub fn add_window(&mut self, window: Box<dyn FnMut(&egui::Context)>) {
        if let Some(windows) = &mut self.egui_windows {
            windows.push(window);
        } else {
            self.egui_windows = Some(vec![window]);
        }
    }

    /// The main event loop of the application
    ///
    /// # Returns
    ///
    /// A future which can be awaited.
    async fn run_engine(
        ecs: Arc<ecs::World>,
        tx_dt: broadcast::Sender<Dt>,
        egui_windows: Option<Vec<Box<dyn FnMut(&egui::Context)>>>,
    ) -> anyhow::Result<()> {
        // * Window creation
        let event_loop = EventLoop::new()?;
        let window_attributes = WindowAttributes::default()
            .with_title("Winit window")
            .with_transparent(true)
            .with_window_icon(None);

        let window = event_loop.create_window(window_attributes)?;
        let mut state = State::new(&window, ecs).await;

        // Proper error handling for initialization
        if let Err(e) = state.init_components().await {
            log::error!("Failed to initialize components: {}", e);
            return Err(e);
        }

        if let Some(egui_windows) = egui_windows {
            state.egui_windows = egui_windows;
        }

        let mut last_render_time = time::Instant::now();

        // * Event loop
        event_loop
            .run(move |event, ewlt| {
                // if let Event::DeviceEvent {
                //     event: DeviceEvent::MouseMotion{ delta, },
                //     .. // We're not using device_id currently
                // } = event {
                //     if state.mouse_pressed {
                //         state.camera_controller.process_mouse(delta.0, delta.1);
                //     }
                // }

                match event {
                    // todo HANDLE this on a separate thread
                    Event::DeviceEvent {
                        event: DeviceEvent::MouseMotion { delta },
                        ..
                    } => {
                        // Handle the mouse motion for the camera if the state is NOT in a paused state
                        //if !state.is_paused() {
                        // TODO bench for performance??
                        if let Some(view_controller) = &state.view_controller {
                            let mut wlock_view_controller = view_controller.write().unwrap();
                            wlock_view_controller.process_mouse(delta.0, delta.1);
                        }
                        //}
                    }
                    Event::WindowEvent {
                        ref event,
                        window_id,
                        // TODO the state.input should handle device events as well as window events
                    } if window_id == state.window().id() && !state.input(event) => {
                        match event {
                            WindowEvent::CloseRequested => ewlt.exit(),
                            WindowEvent::Resized(physical_size) => {
                                state.resize(*physical_size);
                            }
                            // WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => {
                            //     *inner_size_writer = state.size.to_logical::<f64>(*scale_factor);
                            // }
                            WindowEvent::RedrawRequested => {
                                /*
                                   TODO refactor
                                   defer state.update to a tokio::task like in the user facing update loop
                                   send dt -> recv dt -> update
                                   shrink the update channel so if it the update sags behind it will wait before entering a new
                                    draw call and sending the new render dt

                                */
                                let now = time::Instant::now();
                                let dt = now - last_render_time;
                                last_render_time = now;

                                // If the state is paused, busy wait
                                if state.is_paused() {
                                    std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 fps
                                    return;
                                }

                                // * Log FPS
                                // info!(
                                //     "FPS: {:.0}, frame time: {} ms",
                                //     1.0 / &dt.as_secs_f32(),
                                //     &dt.as_millis()
                                // );

                                // Send the delta time using the broadcast channel
                                if let Err(e) = tx_dt.send(dt) {
                                    log::warn!("Failed to send delta time: {:?}", e);
                                }

                                // Handle update errors
                                if let Err(e) = futures::executor::block_on(state.update(dt)) {
                                    log::error!("Update failed: {}", e);
                                    ewlt.exit();
                                    return;
                                }

                                // Handle render errors
                                match state.render() {
                                    Ok(_) => {}
                                    // Reconfigure the surface if it's lost or outdated
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => state.resize(state.size),
                                    // The system is out of memory and must exit
                                    Err(e @ wgpu::SurfaceError::OutOfMemory) => {
                                        log::error!("Critical render error: {}", e);
                                        ewlt.exit()
                                    }
                                    // Ignore timeout errors
                                    Err(wgpu::SurfaceError::Timeout) => {
                                        log::warn!("Surface timeout")
                                    }
                                }
                            }
                            _ => {}
                        };
                    }
                    Event::AboutToWait => {
                        // RedrawRequested will only trigger once unless manually requested.
                        state.window().request_redraw();
                    }
                    _ => {}
                }
            })
            .unwrap();

        Ok(())
    }

    /* /// Create a new update job.
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
    } */
}

impl Drop for GearsApp {
    fn drop(&mut self) {
        self.is_running
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

impl ecs::EntityBuilder for GearsApp {
    fn new_entity(&mut self) -> &mut Self {
        self.world.create_entity();

        self
    }

    fn add_component(&mut self, component: impl Component) -> &mut Self {
        {
            let entity = if let Some(e) = self.world.get_last() {
                e
            } else {
                self.world.create_entity()
            };

            self.world.add_component(entity, component);
        }

        self
    }

    fn build(&mut self) -> ecs::Entity {
        if let Some(e) = self.world.get_last() {
            e
        } else {
            self.world.create_entity()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::new_entity;
    use ecs::EntityBuilder;

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

        let entities = app.world.storage_len();
        assert_eq!(entities, 1);

        let component = app.world.get_component::<TestComponent>(entity).unwrap();
        assert_eq!(component.read().unwrap().value, 10);
    }

    #[test]
    fn test_new_entity_macro() {
        let mut app = crate::core::app::GearsApp::default();
        let entity = new_entity!(app, TestComponent { value: 10 });

        let entities = app.world.storage_len();
        assert_eq!(entities, 1);

        let component = app.world.get_component::<TestComponent>(entity).unwrap();
        assert_eq!(component.read().unwrap().value, 10);
    }
}
