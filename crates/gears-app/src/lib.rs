pub mod errors;
pub mod macros;
pub mod prelude;
pub mod systems;

use gears_core::config::{self, Config};
use gears_ecs::{Component, Entity, EntityBuilder, World};
use gears_gui::EguiWindowCallback;
use gears_renderer::state::State;
use log::{debug, info};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};
use std::time;
use systems::SystemCollection;
use winit::event::{DeviceEvent, Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowAttributes;

use crate::errors::EngineError;

// This struct is used to manage the entire application.
/// The application can also be used to create entities, add components, windows etc. to itself.
pub struct GearsApp {
    config: Config,
    world: Arc<World>,
    egui_windows: Option<Vec<EguiWindowCallback>>,
    is_running: Arc<AtomicBool>,
    internal_async_systems: systems::InternalSystemCollection,
    external_async_systems: systems::ExternalSystemCollection,
}

impl Default for GearsApp {
    fn default() -> Self {
        GearsApp {
            config: config::Config::default(),
            world: Arc::new(World::default()),
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
        Self {
            config,
            world: Arc::new(World::default()),
            egui_windows: None,
            is_running: Arc::new(AtomicBool::new(true)),
            internal_async_systems: systems::InternalSystemCollection::default(),
            external_async_systems: systems::ExternalSystemCollection::default(),
        }
    }

    /// Run the application and start the event loop.
    pub async fn run(&mut self) -> Result<(), EngineError> {
        info!("Starting Gears...");
        let windows = self.egui_windows.take();

        // Run the event loop
        GearsApp::run_engine(self, windows).await
    }

    pub fn add_async_system(&mut self, system: systems::AsyncSystem) {
        self.external_async_systems.add_system(system);
    }

    async fn run_external_systems(&self, world: Arc<World>, dt: time::Duration) {
        debug!("Starting external system execution cycle");

        let mut join_set = tokio::task::JoinSet::new();

        for system in self.external_async_systems.systems() {
            let system_name = system.name();
            let world = Arc::clone(&world);
            debug!("Spawning external system task: {}", system_name);

            join_set.spawn(async move {
                let result = system.run(world, dt).await;
                if let Err(err) = &result {
                    log::error!("External system '{}' failed: {}", system_name, err);
                }
                result
            });
        }

        // Wait for all tasks to complete
        while let Some(result) = join_set.join_next().await {
            if let Err(e) = result {
                log::error!("External system task panicked: {}", e);
            }
        }

        debug!("All external systems completed");
    }

    async fn run_internal_systems(
        &self,
        world: Arc<World>,
        state: Arc<Mutex<State>>,
        dt: time::Duration,
    ) {
        debug!("Starting internal system execution cycle");

        let mut join_set = tokio::task::JoinSet::new();

        for system in self.internal_async_systems.systems() {
            let system_name = system.name();
            let world = Arc::clone(&world);
            let state = Arc::clone(&state);
            debug!("Spawning internal system task: {}", system_name);

            join_set.spawn(async move {
                let result = system.run(world, state, dt).await;
                if let Err(err) = &result {
                    log::error!("Internal system '{}' failed: {}", system_name, err);
                }
                result
            });
        }

        // Wait for all tasks to complete
        while let Some(result) = join_set.join_next().await {
            if let Err(e) = result {
                log::error!("Internal system task panicked: {}", e);
            }
        }

        debug!("All internal systems completed");
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
    ) -> Result<(), EngineError> {
        // * Window creation
        let event_loop = EventLoop::new()?;
        let window_attributes = WindowAttributes::default()
            .with_title("Winit window")
            .with_transparent(true)
            .with_maximized(true)
            .with_active(true)
            .with_window_icon(None);

        let window = Arc::new(event_loop.create_window(window_attributes)?);
        let mut state = State::new(Arc::clone(&window), Arc::clone(&self.world)).await;

        if let Some(windows) = egui_windows {
            state.add_windows(windows);
        }

        // Proper error handling for initialization
        if let Err(e) = state.init_components().await {
            log::error!("Failed to initialize components: {}", e);
            return Err(EngineError::ComponentInitialization(e.to_string()));
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

                        // Clone Arc for systems and wrap state in Arc<Mutex>
                        let world = Arc::clone(&self.world);
                        let state_arc = Arc::new(Mutex::new(state));

                        // Run both system groups concurrently using Tokio runtime
                        tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current().block_on(async {
                                tokio::join!(
                                    self.run_external_systems(Arc::clone(&world), dt),
                                    self.run_internal_systems(Arc::clone(&world), Arc::clone(&state_arc), dt)
                                )
                            })
                        });

                        // Extract state back from Arc
                        let state = Arc::try_unwrap(state_arc).unwrap().into_inner().unwrap();
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
                                if let Err(e) = tokio::runtime::Handle::current().block_on(state.update(dt)) {
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
