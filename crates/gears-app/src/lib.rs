pub mod macros;
pub mod prelude;
pub mod systems;

use gears_core::config::{self, Config};
use gears_core::threadpool::ThreadPool;
use gears_ecs::{Component, Entity, EntityBuilder, World};
use gears_gui::EguiWindowCallback;
use gears_renderer::state::State;
use log::{debug, info};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time;
use systems::SystemCollection;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowAttributes;

// This struct is used to manage the entire application.
/// The application can also be used to create entities, add components, windows etc. to itself.
pub struct GearsApp {
    config: Config,
    world: World,
    pub thread_pool: ThreadPool,
    egui_windows: Option<Vec<EguiWindowCallback>>,
    is_running: Arc<AtomicBool>,
    internal_async_systems: systems::InternalSystemCollection,
    external_async_systems: systems::ExternalSystemCollection,
}

impl Default for GearsApp {
    fn default() -> Self {
        GearsApp {
            config: config::Config::default(),
            world: World::default(),
            thread_pool: ThreadPool::new(8),
            egui_windows: None,
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

        Self {
            thread_pool: ThreadPool::new(config.threadpool_size),
            config,
            world: World::default(),
            egui_windows: None,
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

    pub fn add_async_system(&mut self, system: systems::AsyncSystem) {
        self.external_async_systems.add_system(system);
    }

    async fn run_systems(&self, sa: &systems::SystemAccessors<'_>) {
        debug!("Starting system execution cycle");

        match sa {
            systems::SystemAccessors::Internal { .. } => {
                let futures = self.internal_async_systems.systems().iter().map(|system| {
                    debug!("Preparing internal system: {}", system.name());
                    async move {
                        let result = system.run(sa).await;
                        if let Err(err) = &result {
                            log::error!("Internal system '{}' failed: {}", system.name(), err);
                        }
                        result
                    }
                });

                // Run all futures concurrently and wait for completion
                futures::future::join_all(futures).await;
            }
            systems::SystemAccessors::External { .. } => {
                let futures = self.external_async_systems.systems().iter().map(|system| {
                    debug!("Preparing external system: {}", system.name());
                    async move {
                        let result = system.run(sa).await;
                        if let Err(err) = &result {
                            log::error!("External system '{}' failed: {}", system.name(), err);
                        }
                        result
                    }
                });

                // Run all futures concurrently and wait for completion
                futures::future::join_all(futures).await;
            }
        }

        debug!("All systems completed");
    }

    /// Add a custom window to the app.
    ///
    /// # Arguments
    ///
    /// * `window` - A function that will be called to render the window.
    pub fn add_window(&mut self, window: EguiWindowCallback) {
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
        egui_windows: Option<Vec<EguiWindowCallback>>,
    ) -> anyhow::Result<()> {
        // * Window creation
        let event_loop = EventLoop::new()?;
        let window_attributes = WindowAttributes::default()
            .with_title("Winit window")
            .with_transparent(true)
            .with_maximized(true)
            .with_active(true)
            .with_window_icon(None);

        let window = event_loop.create_window(window_attributes)?;
        let mut state = State::new(&window, &self.world).await;

        if let Some(windows) = egui_windows {
            state.add_windows(windows);
        }

        // Proper error handling for initialization
        if let Err(e) = state.init_components().await {
            log::error!("Failed to initialize components: {}", e);
            return Err(e);
        }

        let mut last_render_time = time::Instant::now();
        let mut dt: time::Duration = time::Duration::from_secs_f32(0_f32);

        // Track the previous pause state to detect transitions
        let mut was_paused = false;

        // * Event loop
        event_loop
            .run(move |event, ewlt| {
                match event {
                    Event::AboutToWait => {
                        let is_paused = state.is_paused();

                        // Detect pause state transitions
                        if is_paused != was_paused {
                            // State changed - reset timing to prevent large deltas
                            last_render_time = time::Instant::now();
                            dt = time::Duration::from_secs_f32(0.0);
                            was_paused = is_paused;

                            if is_paused {
                                log::debug!("Game paused - resetting delta time");
                            } else {
                                log::debug!("Game unpaused - resetting delta time");
                            }
                        }

                        // Handle the paused state
                        if is_paused {
                            // While paused, maintain a small constant dt for UI updates
                            std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 fps
                            dt = time::Duration::from_secs_f32(0.0);
                            return;
                        }

                        // Run systems in a more efficient way
                        // Create a future that runs both internal and external systems concurrently
                        let external_sa = systems::SystemAccessors::External {
                            world: &self.world,
                            dt,
                        };

                        let internal_sa = systems::SystemAccessors::Internal {
                            world: &self.world,
                            state: &state,
                            dt,
                        };

                        // Run both system groups concurrently instead of sequentially
                        futures::executor::block_on(async {
                            futures::join!(
                                self.run_systems(&external_sa),
                                self.run_systems(&internal_sa)
                            )
                        });

                        state.window().request_redraw();
                    }
                    // todo HANDLE this on a separate thread
                    Event::DeviceEvent {
                        event: DeviceEvent::MouseMotion { delta },
                        ..
                    } => {
                        // Ignore mouse events if the app is paused ??
                        if state.is_paused() {
                            return;
                        }

                        // TODO bench for performance??
                        if let Some(view_controller) = &state.view_controller {
                            let mut wlock_view_controller = view_controller.write().unwrap();
                            wlock_view_controller.process_mouse(delta.0, delta.1);
                        }
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
                                // Skip update and render when paused
                                if state.is_paused() {
                                    return;
                                }

                                let now = time::Instant::now();

                                // Limit the maximum delta time to prevent large jumps
                                // This helps if the game was paused or if there was a lag spike
                                let elapsed = now - last_render_time;
                                dt = if elapsed > time::Duration::from_millis(100) {
                                    // Cap at 100ms (10 fps) to prevent large movements
                                    time::Duration::from_millis(100)
                                } else {
                                    elapsed
                                };

                                last_render_time = now;

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
