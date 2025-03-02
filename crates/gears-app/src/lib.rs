pub mod macros;
pub mod prelude;
pub mod systems;

use gears_core::config::{self, Config};
use gears_core::threadpool::ThreadPool;
use gears_core::Dt;
use gears_ecs::{Component, Entity, EntityBuilder, World};
use gears_renderer::state::State;
use log::{info, warn};
use rayon::vec;
use std::future::Future;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time;
use systems::SystemCollection;
use tokio::sync::broadcast;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowAttributes;

// This struct is used to manage the entire application.
/// The application can also be used to create entities, add components, windows etc. to itself.
pub struct GearsApp {
    config: Config,
    world: World,
    pub thread_pool: ThreadPool,
    egui_windows: Option<Vec<Box<dyn FnMut(&egui::Context)>>>,
    tx_dt: Option<tokio::sync::broadcast::Sender<Dt>>,
    rx_dt: Option<tokio::sync::broadcast::Receiver<Dt>>,
    is_running: Arc<AtomicBool>,
    internal_async_systems: systems::InternalSystemCollection,
    external_async_systems: systems::ExternalSystemCollection,
}

impl Default for GearsApp {
    fn default() -> Self {
        let (tx_dt, rx_dt) = broadcast::channel(64);

        GearsApp {
            config: config::Config::default(),
            world: World::default(),
            thread_pool: ThreadPool::new(8),
            egui_windows: None,
            tx_dt: Some(tx_dt),
            rx_dt: Some(rx_dt),
            is_running: Arc::new(AtomicBool::new(true)),
            internal_async_systems: systems::InternalSystemCollection::default(),
            external_async_systems: systems::ExternalSystemCollection::default(),
        }
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
            world: World::default(),
            egui_windows: None,
            tx_dt: Some(tx_dt),
            rx_dt: Some(rx_dt),
            is_running: Arc::new(AtomicBool::new(true)),
            internal_async_systems: systems::InternalSystemCollection::default(),
            external_async_systems: systems::ExternalSystemCollection::default(),
        }
    }

    /// Run the application and start the event loop.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        info!("Starting Gears...");
        let windows = self.egui_windows.take();

        // Run the event loop
        GearsApp::run_engine(self, windows).await
    }

    // /// Add a system to the world.
    // ///
    // /// # Arguments
    // ///
    // /// * `system` - The system to add.
    // pub fn add_system(&mut self, system: systems::System) {
    //     self.systems.push(system);
    // }

    pub fn add_async_system(&mut self, system: systems::AsyncSystem) {
        self.external_async_systems.add_system(system);
    }

    async fn run_systems(&self, sa: &systems::SystemAccessors<'_>) {
        log::debug!("Starting system execution cycle");

        let mut futures = vec![];

        match sa {
            systems::SystemAccessors::Internal { .. } => {
                let mut futures = vec![];
                futures.extend(self.internal_async_systems.systems().iter().map(|system| {
                    log::debug!("Preparing internal system: {}", system.name);
                    system.run(sa)
                }));
            }
            systems::SystemAccessors::External { .. } => {
                futures.extend(self.external_async_systems.systems().iter().map(|system| {
                    log::debug!("Preparing external system: {}", system.name);
                    system.run(sa)
                }));
            }
        }

        // Run all futures concurrently and wait for completion
        futures::future::join_all(futures).await;

        log::debug!("All systems completed");
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
        &self,
        egui_windows: Option<Vec<Box<dyn FnMut(&egui::Context)>>>,
    ) -> anyhow::Result<()> {
        // * Window creation
        let event_loop = EventLoop::new()?;
        let window_attributes = WindowAttributes::default()
            .with_title("Winit window")
            .with_transparent(true)
            .with_window_icon(None);

        let window = event_loop.create_window(window_attributes)?;
        let mut state = State::new(&window, &self.world).await;

        if let Some(windows) = egui_windows {
            state.add_windows(windows);
        }

        // Grab the cursor upon initialization
        state.grab_cursor();

        // Proper error handling for initialization
        if let Err(e) = state.init_components().await {
            log::error!("Failed to initialize components: {}", e);
            return Err(e);
        }

        let mut last_render_time = time::Instant::now();
        let mut dt: time::Duration = time::Duration::from_secs_f32(0_f32);
        let tx_dt = self.tx_dt.as_ref().unwrap();

        // * Event loop
        event_loop
            .run(move |event, ewlt| {
                match event {
                    Event::AboutToWait => {
                        // Only run systems during the AboutToWait event
                        // Start with the internal systems
                        let system_accessors = systems::SystemAccessors::Internal {
                            world: &self.world,
                            state: &state,
                            dt,
                        };
                        futures::executor::block_on(self.run_systems(&system_accessors));

                        // Then run the external systems
                        let system_accessors = systems::SystemAccessors::External {
                            world: &self.world,
                            dt,
                        };
                        futures::executor::block_on(self.run_systems(&system_accessors));

                        // Request a redraw
                        state.window().request_redraw();
                    }
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
                                dt = now - last_render_time;
                                last_render_time = now;

                                // If the state is paused, busy wait
                                if state.is_paused() {
                                    std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 fps
                                    return;
                                }

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
                    _ => {}
                }
            })
            .unwrap();

        Ok(())
    }
}

impl Drop for GearsApp {
    fn drop(&mut self) {
        self.is_running
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

impl EntityBuilder for GearsApp {
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

    fn build(&mut self) -> Entity {
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
        let mut app = GearsApp::default();
        let entity = new_entity!(app, TestComponent { value: 10 });

        let entities = app.world.storage_len();
        assert_eq!(entities, 1);

        let component = app.world.get_component::<TestComponent>(entity).unwrap();
        assert_eq!(component.read().unwrap().value, 10);
    }
}
