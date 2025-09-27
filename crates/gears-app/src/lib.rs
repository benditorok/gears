pub mod errors;
pub mod macros;
pub mod prelude;
pub mod systems;

use crate::errors::EngineError;
use gears_core::config::{self, Config};
use gears_ecs::{Component, Entity, EntityBuilder, World};
use gears_gui::EguiWindowCallback;
use gears_renderer::errors::RendererError;
use gears_renderer::state::State;
use log::{debug, info};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use std::time;
use systems::SystemCollection;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowAttributes};

// This struct is used to manage the entire application.
/// The application can also be used to create entities, add components, windows etc. to itself.
pub struct GearsApp {
    config: Config,
    event_loop: Option<EventLoop<()>>,
    world: Arc<World>,
    window: Arc<Window>,
    state: Arc<RwLock<State>>,
    egui_windows: Option<Vec<EguiWindowCallback>>,
    is_running: Arc<AtomicBool>,
    internal_async_systems: systems::InternalSystemCollection,
    external_async_systems: systems::ExternalSystemCollection,
}

impl Default for GearsApp {
    fn default() -> Self {
        // Window creation
        let config = Config::default();
        let event_loop = EventLoop::new().expect("Window EventLoop creation failed");
        let window_attributes = WindowAttributes::default()
            .with_title(config.window_title)
            .with_transparent(true)
            .with_maximized(true)
            .with_active(true)
            .with_window_icon(None);

        let world = Arc::new(World::default());
        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Window creation failed"),
        );

        let state = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                Arc::new(RwLock::new(
                    State::new(Arc::clone(&window), Arc::clone(&world)).await,
                ))
            })
        });

        Self {
            config: config::Config::default(),
            event_loop: Some(event_loop),
            world,
            window,
            state,
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
        // Window creation
        let event_loop = EventLoop::new().expect("Window EventLoop creation failed");
        let window_attributes = WindowAttributes::default()
            .with_title(config.window_title)
            .with_transparent(true)
            .with_maximized(true)
            .with_active(true)
            .with_window_icon(None);

        let world = Arc::new(World::default());
        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Window creation failed"),
        );

        let state = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                Arc::new(RwLock::new(
                    State::new(Arc::clone(&window), Arc::clone(&world)).await,
                ))
            })
        });

        Self {
            config,
            event_loop: Some(event_loop),
            world,
            window,
            state,
            egui_windows: None,
            is_running: Arc::new(AtomicBool::new(true)),
            internal_async_systems: systems::InternalSystemCollection::default(),
            external_async_systems: systems::ExternalSystemCollection::default(),
        }
    }

    /// Run the application and start the event loop.
    pub async fn run(&mut self) -> Result<(), EngineError> {
        info!("Starting gears engine");

        // Run the event loop
        GearsApp::run_engine(self).await
    }

    pub fn add_async_system(&mut self, system: systems::AsyncSystem) {
        self.external_async_systems.add_system(system);
    }

    async fn run_external_systems(&self, world: Arc<World>, dt: time::Duration) {
        debug!("Starting external system execution cycle");

        let system_count = self.external_async_systems.async_systems.len();
        if system_count == 0 {
            debug!("No external systems to run");
            return;
        }

        let mut join_set = tokio::task::JoinSet::new();

        // Use indices to avoid borrowing issues
        for i in 0..system_count {
            let system = &self.external_async_systems.async_systems[i];
            let system_name = system.name();
            let world_clone = Arc::clone(&world);
            debug!("Spawning external system task: {}", system_name);

            // Call the system function directly to get the future
            let system_func = &system.func;
            let future = system_func.call(world_clone, dt);

            join_set.spawn(async move {
                let result = future.await;
                if let Err(err) = &result {
                    log::error!("External system '{}' failed: {}", system_name, err);
                }
                (system_name, result)
            });
        }

        // Wait for all tasks to complete concurrently
        let mut completed_count = 0;
        let total_systems = join_set.len();

        while let Some(task_result) = join_set.join_next().await {
            completed_count += 1;
            match task_result {
                Ok((system_name, system_result)) => {
                    if let Err(e) = system_result {
                        log::error!(
                            "External system '{}' completed with error: {}",
                            system_name,
                            e
                        );
                    } else {
                        debug!(
                            "External system '{}' completed successfully ({}/{})",
                            system_name, completed_count, total_systems
                        );
                    }
                }
                Err(e) => {
                    log::error!("External system task panicked: {}", e);
                }
            }
        }

        debug!("All {} external systems completed", total_systems);
    }

    async fn run_internal_systems(
        &self,
        world: Arc<World>,
        state: Arc<RwLock<State>>,
        dt: time::Duration,
    ) {
        debug!("Starting internal system execution cycle");

        let system_count = self.internal_async_systems.async_systems.len();
        if system_count == 0 {
            debug!("No internal systems to run");
            return;
        }

        let mut join_set = tokio::task::JoinSet::new();

        // Use indices to avoid borrowing issues
        for i in 0..system_count {
            let system = &self.internal_async_systems.async_systems[i];
            let system_name = system.name();
            let world_clone = Arc::clone(&world);
            let state_clone = Arc::clone(&state);
            debug!("Spawning internal system task: {}", system_name);

            // Call the system function directly to get the future
            let system_func = &system.func;
            let future = system_func.call(world_clone, state_clone, dt);

            join_set.spawn(async move {
                let result = future.await;
                if let Err(err) = &result {
                    log::error!("Internal system '{}' failed: {}", system_name, err);
                }
                (system_name, result)
            });
        }

        // Wait for all tasks to complete concurrently
        let mut completed_count = 0;
        let total_systems = join_set.len();

        while let Some(task_result) = join_set.join_next().await {
            completed_count += 1;
            match task_result {
                Ok((system_name, system_result)) => {
                    if let Err(e) = system_result {
                        log::error!(
                            "Internal system '{}' completed with error: {}",
                            system_name,
                            e
                        );
                    } else {
                        debug!(
                            "Internal system '{}' completed successfully ({}/{})",
                            system_name, completed_count, total_systems
                        );
                    }
                }
                Err(e) => {
                    log::error!("Internal system task panicked: {}", e);
                }
            }
        }

        debug!("All {} internal systems completed", total_systems);
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
    async fn run_engine(&mut self) -> Result<(), EngineError> {
        if let Some(windows) = self.egui_windows.take() {
            self.state.write().unwrap().add_windows(windows);
        }

        // Proper error handling for initialization
        if let Err(e) = self.state.write().unwrap().init_components().await {
            log::error!("Failed to initialize components: {}", e);
            return Err(EngineError::ComponentInitialization(e.to_string()));
        }

        let mut last_render_time = time::Instant::now();
        let mut dt: time::Duration = time::Duration::from_secs_f32(0_f32);

        // Track the previous pause state to detect transitions
        let mut was_paused = false;

        // Get the event loop
        let event_loop = self
            .event_loop
            .take()
            .expect("EventLoop was not initialized");

        // * Event loop
        event_loop
            .run(move |event, ewlt| {
                match event {
                    Event::AboutToWait => {
                        let is_paused = self.state.read().unwrap().is_paused();

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

                        // Run both system groups concurrently using Tokio runtime
                        tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current().block_on(async {
                                // System collections should be run one after another to ensure proper ordering
                                // as to avoid any potential issues with data consistency and synchronization.
                                self.run_external_systems(Arc::clone(&self.world), dt).await;
                                self.run_internal_systems(Arc::clone(&self.world), Arc::clone(&self.state), dt).await;
                            })
                        });

                        self.state.read().unwrap().window().request_redraw();
                    }
                    Event::DeviceEvent {
                        event: DeviceEvent::MouseMotion { delta },
                        ..
                    } => {
                        // Ignore mouse events if the app is paused
                        if self.state.read().unwrap().is_paused() {
                            return;
                        }

                        // TODO bench for performance??
                        if let Some(view_controller) = &self.state.read().unwrap().view_controller {
                            let mut wlock_view_controller = view_controller.write().unwrap();
                            wlock_view_controller.process_mouse(delta.0, delta.1);
                        }
                    }
                    Event::WindowEvent {
                        ref event,
                        window_id,
                        // TODO the state.input should handle device events as well as window events
                    } if {
                        window_id == self.state.read().unwrap().window().id() && !self.state.write().unwrap().input(event)
                    } => {
                        match event {
                            WindowEvent::CloseRequested => ewlt.exit(),
                            WindowEvent::Resized(physical_size) => {
                                self.state.write().unwrap().resize(*physical_size);
                            }
                            // WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => {
                            //     *inner_size_writer = state.size.to_logical::<f64>(*scale_factor);
                            // }
                            WindowEvent::RedrawRequested => {
                                // Skip update and render when paused
                                if self.state.read().unwrap().is_paused() {
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
                                tokio::task::block_in_place(|| {
                                    tokio::runtime::Handle::current().block_on(async {
                                        if let Err(e) = self.state.write().unwrap().update(dt).await {
                                            log::error!("Update failed: {}", e);
                                            ewlt.exit();
                                            return;
                                        }
                                    })
                                });

                                // Handle render errors
                                match self.state.write().unwrap().render() {
                                    Ok(_) => {}
                                    // Reconfigure the surface if it's lost or outdated
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => {
                                        let size = self.state.read().unwrap().size;
                                        self.state.write().unwrap().resize(size);
                                    }
                                    // The system is out of memory and must exit
                                    Err(e @ wgpu::SurfaceError::OutOfMemory) => {
                                        log::error!("Critical render error: {}", e);
                                        ewlt.exit()
                                    }
                                    // Ignore timeout errors
                                    Err(wgpu::SurfaceError::Timeout) => {
                                        log::warn!("Surface timeout")
                                    }
                                    Err(wgpu::SurfaceError::Other) => {
                                        log::error!("Acquiring a texture failed with a generic error. Check error callbacks for more information.");
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
