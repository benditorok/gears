mod error;
mod update_systems;

use core::time;
use gears_ecs::{
    components::{self},
    World,
};
use gears_renderer::state::State;
use std::{future::Future, pin::Pin};

pub use error::{SystemError, SystemResult};

/// System accessors allow systems to access different parts of the engine
/// depending on whether they are internal or external systems
pub enum SystemAccessors<'a> {
    /// Internal systems have access to the world, state and delta time
    Internal {
        world: &'a World,
        state: &'a State<'a>,
        dt: time::Duration,
    },
    /// External systems only have access to the world and delta time
    External {
        world: &'a World,
        dt: time::Duration,
    },
}

/// Base trait for all async systems
pub trait AsyncSystemFn: Send + Sync {
    fn run<'a>(
        &'a self,
        sa: &'a SystemAccessors<'a>,
    ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send + 'a>>;
}

// Separate wrapper types for each implementation - without storing the future type

/// Wrapper for regular async closures - no need to store Fut
pub(crate) struct AsyncFnWrapper<F> {
    func: F,
}

/// Wrapper for functions that return boxed futures
pub(crate) struct AsyncBoxFnWrapper<F> {
    func: F,
}

// Implementation for regular async functions/closures
impl<F, Fut> AsyncSystemFn for AsyncFnWrapper<F>
where
    F: for<'r> Fn(&'r SystemAccessors<'r>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = SystemResult<()>> + Send + 'static,
{
    fn run<'a>(
        &'a self,
        sa: &'a SystemAccessors<'a>,
    ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send + 'a>> {
        Box::pin(async move { (self.func)(sa).await })
    }
}

// Implementation for boxed futures
impl<F> AsyncSystemFn for AsyncBoxFnWrapper<F>
where
    F: for<'r> Fn(
            &'r SystemAccessors<'r>,
        ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send + 'r>>
        + Send
        + Sync
        + 'static,
{
    fn run<'a>(
        &'a self,
        sa: &'a SystemAccessors<'a>,
    ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send + 'a>> {
        (self.func)(sa)
    }
}

/// An async system with a name and function
pub struct AsyncSystem {
    name: &'static str,
    func: Box<dyn AsyncSystemFn + Send + Sync>,
}

impl AsyncSystem {
    /// Create a new async system with the given name and function
    pub(crate) fn new<F>(name: &'static str, func: F) -> Self
    where
        F: AsyncSystemFn + Send + Sync + 'static,
    {
        Self {
            name,
            func: Box::new(func),
        }
    }

    /// Get the name of the system
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Run the system with the given system accessors
    pub async fn run<'a>(&'a self, sa: &'a SystemAccessors<'a>) -> SystemResult<()> {
        self.func.run(sa).await
    }
}

/// Helper function for creating systems from regular async functions or closures
pub fn system<F, Fut>(name: &'static str, func: F) -> AsyncSystem
where
    F: for<'r> Fn(&'r SystemAccessors<'r>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = SystemResult<()>> + Send + 'static,
{
    AsyncSystem::new(name, AsyncFnWrapper { func })
}

/// Helper function for creating systems from functions that return boxed futures
pub fn async_system<F>(name: &'static str, func: F) -> AsyncSystem
where
    F: for<'r> Fn(
            &'r SystemAccessors<'r>,
        ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send + 'r>>
        + Send
        + Sync
        + 'static,
{
    AsyncSystem::new(name, AsyncBoxFnWrapper { func })
}

/// Interface for collections of systems
pub trait SystemCollection {
    fn add_system(&mut self, system: AsyncSystem);
    fn systems(&self) -> &[AsyncSystem];
}

/// Collection of internal systems used by the engine
pub(crate) struct InternalSystemCollection {
    pub async_systems: Vec<AsyncSystem>,
}

impl Default for InternalSystemCollection {
    fn default() -> Self {
        Self {
            async_systems: vec![
                async_system("update_lights", |sa| {
                    Box::pin(update_systems::update_lights(sa))
                }),
                async_system("update_models", |sa| {
                    Box::pin(update_systems::update_models(sa))
                }),
                async_system("physics_system", |sa| {
                    Box::pin(update_systems::physics_system(sa))
                }),
            ],
        }
    }
}

impl SystemCollection for InternalSystemCollection {
    fn add_system(&mut self, system: AsyncSystem) {
        self.async_systems.push(system);
    }

    fn systems(&self) -> &[AsyncSystem] {
        &self.async_systems
    }
}

/// Collection of external systems provided by the user
pub struct ExternalSystemCollection {
    pub(crate) async_systems: Vec<AsyncSystem>,
}

impl Default for ExternalSystemCollection {
    fn default() -> Self {
        Self {
            async_systems: vec![],
        }
    }
}

impl SystemCollection for ExternalSystemCollection {
    fn add_system(&mut self, system: AsyncSystem) {
        self.async_systems.push(system);
    }

    fn systems(&self) -> &[AsyncSystem] {
        &self.async_systems
    }
}
