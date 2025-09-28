pub mod errors;
pub mod macros;
pub mod prelude;
pub mod systems;

use crate::errors::EngineError;
use gears_core::config::{self, Config};
use gears_ecs::{Component, Entity, EntityBuilder, World};
use gears_gui::EguiWindowCallback;

use gears_renderer::state::State;
use log::{debug, info};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use std::time;
use systems::SystemCollection;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

// This struct is used to manage the entire application.
/// The application can also be used to create entities, add components, windows etc. to itself.
pub struct GearsApp {
    config: Config,
    world: Arc<World>,
    window: Option<Arc<Window>>,
    state: Option<Arc<RwLock<State>>>,
    egui_windows: Option<Vec<EguiWindowCallback>>,
    is_running: Arc<AtomicBool>,
    internal_async_systems: systems::InternalSystemCollection,
    external_async_systems: systems::ExternalSystemCollection,
    last_render_time: time::Instant,
    dt: time::Duration,
    was_paused: bool,
}

impl Default for GearsApp {
    fn default() -> Self {
        Self {
            config: Config::default(),
            world: Arc::new(World::default()),
            window: None,
            state: None,
            egui_windows: None,
            is_running: Arc::new(AtomicBool::new(true)),
            internal_async_systems: systems::InternalSystemCollection::default(),
            external_async_systems: systems::ExternalSystemCollection::default(),
            last_render_time: time::Instant::now(),
            dt: time::Duration::from_secs_f32(0_f32),
            was_paused: false,
        }
    }
}

impl GearsApp {
    /// Initialize the application.
    /// This will create a new instance of the application with the given configuration.
    pub fn new(config: config::Config) -> Self {
        Self {
            config,
            world: Arc::new(World::default()),
            window: None,
            state: None,
            egui_windows: None,
            is_running: Arc::new(AtomicBool::new(true)),
            internal_async_systems: systems::InternalSystemCollection::default(),
            external_async_systems: systems::ExternalSystemCollection::default(),
            last_render_time: time::Instant::now(),
            dt: time::Duration::from_secs_f32(0_f32),
            was_paused: false,
        }
    }

    /// Run the application and start the event loop.
    pub fn run(&mut self) -> Result<(), EngineError> {
        info!("Starting gears engine");

        // ! The event loop must be created on the main thread
        let event_loop =
            EventLoop::new().map_err(|e| EngineError::ComponentInitialization(e.to_string()))?;
        event_loop
            .run_app(self)
            .map_err(|e| EngineError::ComponentInitialization(e.to_string()))
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
}

impl ApplicationHandler for GearsApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window if it doesn't exist
        if self.window.is_none() {
            let window_attributes = WindowAttributes::default()
                .with_title(self.config.window_title)
                .with_transparent(true)
                .with_maximized(true)
                .with_active(true)
                .with_window_icon(None);

            let window = Arc::new(
                event_loop
                    .create_window(window_attributes)
                    .expect("Window creation failed"),
            );

            // Initialize state
            let state = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    Arc::new(RwLock::new(
                        State::new(Arc::clone(&window), Arc::clone(&self.world)).await,
                    ))
                })
            });

            // Add egui windows if any
            if let Some(windows) = self.egui_windows.take() {
                state.write().unwrap().add_windows(windows);
            }

            // Initialize components
            if let Err(e) = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { state.write().unwrap().init_components().await })
            }) {
                log::error!("Failed to initialize components: {}", e);
                event_loop.exit();
                return;
            }

            self.window = Some(window);
            self.state = Some(state);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let (Some(state), Some(window)) = (&self.state, &self.window) {
            let is_paused = state.read().unwrap().is_paused();

            // Detect pause state transitions
            if is_paused != self.was_paused {
                // State changed - reset timing to prevent large deltas
                self.last_render_time = time::Instant::now();
                self.dt = time::Duration::from_secs_f32(0.0);
                self.was_paused = is_paused;

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
                self.dt = time::Duration::from_secs_f32(0.0);
                return;
            }

            // Run both system groups concurrently using Tokio runtime
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    // System collections should be run one after another to ensure proper ordering
                    // as to avoid any potential issues with data consistency and synchronization.
                    self.run_external_systems(Arc::clone(&self.world), self.dt)
                        .await;
                    self.run_internal_systems(Arc::clone(&self.world), Arc::clone(&state), self.dt)
                        .await;
                })
            });

            window.request_redraw();
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let state = match &self.state {
            Some(state) => state,
            None => return,
        };

        match event {
            DeviceEvent::MouseMotion { delta } => {
                // Ignore mouse events if the app is paused
                if state.read().unwrap().is_paused() {
                    return;
                }

                // TODO bench for performance??
                if let Some(view_controller) = &state.read().unwrap().view_controller() {
                    let mut wlock_view_controller = view_controller.write().unwrap();
                    wlock_view_controller.process_mouse(delta.0, delta.1);
                }
            }
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let (Some(state), Some(window)) = (&self.state, &self.window) {
            // Check if this is our window and if input should be handled
            if window_id != window.id() || state.write().unwrap().input(&event) {
                return;
            }

            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(physical_size) => {
                    state.write().unwrap().resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    // Skip update and render when paused
                    if state.read().unwrap().is_paused() {
                        return;
                    }

                    let now = time::Instant::now();

                    // Limit the maximum delta time to prevent large jumps
                    // This helps if the game was paused or if there was a lag spike
                    let elapsed = now - self.last_render_time;
                    self.dt = if elapsed > time::Duration::from_millis(100) {
                        // Cap at 100ms (10 fps) to prevent large movements
                        time::Duration::from_millis(100)
                    } else {
                        elapsed
                    };

                    self.last_render_time = now;

                    // Handle update errors
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            if let Err(e) = state.write().unwrap().update(self.dt).await {
                                log::error!("Update failed: {}", e);
                                event_loop.exit();
                                return;
                            }
                        })
                    });

                    // Handle render errors
                    match state.write().unwrap().render() {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            state.write().unwrap().resize_self();
                        }
                        // The system is out of memory and must exit
                        Err(e @ wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("Critical render error: {}", e);
                            event_loop.exit()
                        }
                        // Ignore timeout errors
                        Err(wgpu::SurfaceError::Timeout) => {
                            log::warn!("Surface timeout")
                        }
                        Err(wgpu::SurfaceError::Other) => {
                            log::error!(
                                "Acquiring a texture failed with a generic error. Check error callbacks for more information."
                            );
                        }
                    }
                }
                _ => {}
            }
        }
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
