mod errors;
mod update_systems;

use core::time;
use gears_ecs::World;
use gears_renderer::state::State;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, RwLock};

pub use errors::{SystemError, SystemResult};

/// Trait for async system functions that can capture variables
pub trait AsyncSystemFn: Send + Sync {
    fn call(
        &self,
        world: Arc<World>,
        dt: time::Duration,
    ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>>;
}

/// Trait for internal async system functions
pub(crate) trait InternalAsyncSystemFn: Send + Sync {
    fn call(
        &self,
        world: Arc<World>,
        state: Arc<Mutex<State>>,
        dt: time::Duration,
    ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>>;
}

/// Wrapper for closures that implement AsyncSystemFn
pub struct AsyncSystemClosure<F> {
    func: F,
}

impl<F> AsyncSystemClosure<F> {
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
    fn call(
        &self,
        world: Arc<World>,
        dt: time::Duration,
    ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>> {
        (self.func)(world, dt)
    }
}

/// Wrapper for internal system closures
pub(crate) struct InternalAsyncSystemClosure<F> {
    func: F,
}

impl<F> InternalAsyncSystemClosure<F> {
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F> InternalAsyncSystemFn for InternalAsyncSystemClosure<F>
where
    F: Fn(
            Arc<World>,
            Arc<Mutex<State>>,
            time::Duration,
        ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>>
        + Send
        + Sync,
{
    fn call(
        &self,
        world: Arc<World>,
        state: Arc<Mutex<State>>,
        dt: time::Duration,
    ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>> {
        (self.func)(world, state, dt)
    }
}

/// An async system with a name and function
pub struct AsyncSystem {
    name: &'static str,
    func: Box<dyn AsyncSystemFn>,
}

/// An internal async system with a name and function
pub(crate) struct InternalAsyncSystem {
    name: &'static str,
    func: Box<dyn InternalAsyncSystemFn>,
}

impl AsyncSystem {
    /// Create a new external async system
    pub fn new<F: AsyncSystemFn + 'static>(name: &'static str, func: F) -> Self {
        Self {
            name,
            func: Box::new(func),
        }
    }

    /// Get the name of the system
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Run the system
    pub async fn run(&self, world: Arc<World>, dt: time::Duration) -> SystemResult<()> {
        self.func.call(world, dt).await
    }
}

impl InternalAsyncSystem {
    /// Create a new internal async system
    pub(crate) fn new<F: InternalAsyncSystemFn + 'static>(name: &'static str, func: F) -> Self {
        Self {
            name,
            func: Box::new(func),
        }
    }

    /// Get the name of the system
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Run the system
    pub async fn run(
        &self,
        world: Arc<World>,
        state: Arc<Mutex<State>>,
        dt: time::Duration,
    ) -> SystemResult<()> {
        self.func.call(world, state, dt).await
    }
}

/// Helper function for creating external systems from closures
pub fn system<F>(name: &'static str, func: F) -> AsyncSystem
where
    F: Fn(Arc<World>, time::Duration) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>>
        + Send
        + Sync
        + 'static,
{
    AsyncSystem::new(name, AsyncSystemClosure::new(func))
}

/// Helper function for creating internal systems from closures
pub(crate) fn internal_system<F>(name: &'static str, func: F) -> InternalAsyncSystem
where
    F: Fn(
            Arc<World>,
            Arc<Mutex<State>>,
            time::Duration,
        ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>>
        + Send
        + Sync
        + 'static,
{
    InternalAsyncSystem::new(name, InternalAsyncSystemClosure::new(func))
}

/// Interface for collections of external systems
pub trait SystemCollection {
    fn add_system(&mut self, system: AsyncSystem);
    fn systems(&self) -> &[AsyncSystem];
}

/// Collection of internal systems used by the engine
pub(crate) struct InternalSystemCollection {
    pub async_systems: Vec<InternalAsyncSystem>,
}

impl Default for InternalSystemCollection {
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

impl InternalSystemCollection {
    pub fn systems(&self) -> &[InternalAsyncSystem] {
        &self.async_systems
    }
}

/// Collection of external systems provided by the user
#[derive(Default)]
pub struct ExternalSystemCollection {
    pub(crate) async_systems: Vec<AsyncSystem>,
}

impl SystemCollection for ExternalSystemCollection {
    fn add_system(&mut self, system: AsyncSystem) {
        self.async_systems.push(system);
    }

    fn systems(&self) -> &[AsyncSystem] {
        &self.async_systems
    }
}
