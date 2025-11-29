// TODO: remove unnecessary wrappers

pub mod errors;
mod update_systems;

use core::time;
use errors::{SystemError, SystemResult};
use gears_ecs::World;
use gears_renderer::state::State;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

/// Trait for async system functions that can capture variables.
pub trait AsyncSystemFn: Send + Sync {
    /// Calls the async system function with the given world and delta time.
    ///
    /// # Returns
    ///
    /// A future that resolves to a [`SystemResult`] indicating the success or failure of the system.
    fn call(
        &self,
        world: Arc<World>,
        dt: time::Duration,
    ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>>;
}

/// Trait for internal async system functions.
pub(crate) trait InternalAsyncSystemFn: Send + Sync {
    /// Calls the internal async system function with the given world, state, and delta time.
    ///
    /// # Returns
    ///
    /// A future that resolves to a [`SystemResult`] indicating the success or failure of the system.
    fn call(
        &self,
        world: Arc<World>,
        state: Arc<RwLock<State>>,
        dt: time::Duration,
    ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>>;
}

/// Wrapper for closures that implement [`AsyncSystemFn`].
pub struct AsyncSystemClosure<F> {
    func: F,
}

impl<F> AsyncSystemClosure<F> {
    /// Create a new async system from a closure.
    ///
    /// # Returns
    ///
    /// A new [`AsyncSystemClosure`] instance.
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F> AsyncSystemFn for AsyncSystemClosure<F>
where
    F: Fn(Arc<World>, time::Duration) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>>
        + Send
        + Sync,
{
    /// Calls the async system function with the given world and delta time.
    ///
    /// # Returns
    ///
    /// A future that resolves to a [`SystemResult`] indicating the success or failure of the system.
    fn call(
        &self,
        world: Arc<World>,
        dt: time::Duration,
    ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>> {
        (self.func)(world, dt)
    }
}

/// Wrapper for internal system closures.
pub(crate) struct InternalAsyncSystemClosure<F> {
    func: F,
}

impl<F> InternalAsyncSystemClosure<F> {
    /// Creates a new internal async system from a closure.
    ///
    /// # Arguments
    ///
    /// * `func` - The closure to be wrapped.
    ///
    /// # Returns
    ///
    /// A new [`InternalAsyncSystemClosure`] instance.
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F> InternalAsyncSystemFn for InternalAsyncSystemClosure<F>
where
    F: Fn(
            Arc<World>,
            Arc<RwLock<State>>,
            time::Duration,
        ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>>
        + Send
        + Sync,
{
    /// Calls the internal async system function with the given world, state, and delta time.
    ///
    /// # Returns
    ///
    /// A future that resolves to a [`SystemResult`] indicating the success or failure of the system.
    fn call(
        &self,
        world: Arc<World>,
        state: Arc<RwLock<State>>,
        dt: time::Duration,
    ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>> {
        (self.func)(world, state, dt)
    }
}

/// An async system with a name and function.
pub struct AsyncSystem {
    name: &'static str,
    pub(crate) func: Box<dyn AsyncSystemFn>,
}

impl AsyncSystem {
    /// Create a new external async system.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the system.
    /// * `func` - The function to run when the system is called.
    ///
    /// # Returns
    ///
    /// A new [`AsyncSystem`] instance.
    pub fn new<F: AsyncSystemFn + 'static>(name: &'static str, func: F) -> Self {
        Self {
            name,
            func: Box::new(func),
        }
    }

    /// Get the name of the system
    ///
    /// # Returns
    ///
    /// The name of the system.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Run the system.
    ///
    /// # Arguments
    ///
    /// * `world` - The world to run the system in.
    /// * `dt` - The duration of the last frame.
    ///
    /// # Returns
    ///
    /// A [`SystemResult`] indicating the success or failure of the system call.
    pub async fn run(&self, world: Arc<World>, dt: time::Duration) -> SystemResult<()> {
        self.func.call(world, dt).await
    }
}

/// An internal async system with a name and function.
pub(crate) struct InternalAsyncSystem {
    name: &'static str,
    pub(crate) func: Box<dyn InternalAsyncSystemFn>,
}

impl InternalAsyncSystem {
    /// Create a new internal async system.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the system.
    /// * `func` - The function to run when the system is called.
    ///
    /// # Returns
    ///
    /// A new [`InternalAsyncSystem`] instance.
    pub(crate) fn new<F: InternalAsyncSystemFn + 'static>(name: &'static str, func: F) -> Self {
        Self {
            name,
            func: Box::new(func),
        }
    }

    /// Get the name of the system.
    ///
    /// # Returns
    ///
    /// The name of the system.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Run the system.
    ///
    /// # Arguments
    ///
    /// * `world` - The world to run the system in.
    /// * `state` - The state to run the system in.
    /// * `dt` - The duration of the last frame.
    ///
    /// # Returns
    ///
    /// A [`SystemResult`] indicating the success or failure of the system call.
    #[allow(unused)]
    pub async fn run(
        &self,
        world: Arc<World>,
        state: Arc<RwLock<State>>,
        dt: time::Duration,
    ) -> SystemResult<()> {
        self.func.call(world, state, dt).await
    }
}

/// Helper function for creating external systems from closures.
///
/// # Arguments
///
/// * `name` - The name of the system.
/// * `func` - The closure to run the system.
///
/// # Returns
///
/// An [`AsyncSystem`] that can be added to a [`SystemCollection`].
pub fn system<F>(name: &'static str, func: F) -> AsyncSystem
where
    F: Fn(Arc<World>, time::Duration) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>>
        + Send
        + Sync
        + 'static,
{
    AsyncSystem::new(name, AsyncSystemClosure::new(func))
}

/// Helper function for creating internal systems from closures.
///
/// # Arguments
///
/// * `name` - The name of the system.
/// * `func` - The closure to run the system.
///
/// # Returns
///
/// An [`InternalAsyncSystem`] that can be added to an [`InternalSystemCollection`].
pub(crate) fn internal_system<F>(name: &'static str, func: F) -> InternalAsyncSystem
where
    F: Fn(
            Arc<World>,
            Arc<RwLock<State>>,
            time::Duration,
        ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>>
        + Send
        + Sync
        + 'static,
{
    InternalAsyncSystem::new(name, InternalAsyncSystemClosure::new(func))
}

/// Trait for collections of external systems.
pub trait SystemCollection {
    /// Adds a system to the collection.
    fn add_system(&mut self, system: AsyncSystem);
    /// Returns a slice of all systems in the collection.
    fn systems(&self) -> &[AsyncSystem];
}

/// Collection of internal systems used by the engine
pub(crate) struct InternalSystemCollection {
    /// Collection of internal systems used by the engine.
    pub async_systems: Vec<InternalAsyncSystem>,
}

impl Default for InternalSystemCollection {
    /// Creates the default instance of the internal system collection.
    ///
    /// # Returns
    ///
    /// A new instance of [`InternalSystemCollection`] with the default internal systems.
    fn default() -> Self {
        Self {
            async_systems: vec![
                internal_system("update_lights", update_systems::update_lights),
                internal_system("update_models", update_systems::update_models),
                internal_system("update_physics", update_systems::update_physics),
            ],
        }
    }
}

/// Collection of external systems provided by the user.
#[derive(Default)]
pub struct ExternalSystemCollection {
    /// Collection of external systems provided by the user.
    pub(crate) async_systems: Vec<AsyncSystem>,
}

impl SystemCollection for ExternalSystemCollection {
    /// Adds a system to the collection.
    fn add_system(&mut self, system: AsyncSystem) {
        self.async_systems.push(system);
    }

    /// Returns a slice of all systems in the collection.
    fn systems(&self) -> &[AsyncSystem] {
        &self.async_systems
    }
}
