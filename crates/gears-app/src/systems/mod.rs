mod error;
mod update_systems;

use core::time;
use gears_ecs::World;
use gears_renderer::state::State;
use std::future::Future;
use std::pin::Pin;

pub use error::{SystemError, SystemResult};

/// System accessors allow external systems to access different parts of the engine
pub struct SystemAccessors<'a> {
    pub world: &'a World,
    pub dt: time::Duration,
}

/// Internal system accessors allow internal systems to access engine state
pub(crate) struct InternalSystemAccessors<'a> {
    pub world: &'a World,
    pub state: &'a State<'a>,
    pub dt: time::Duration,
}

/// A simple async system function type
pub type AsyncSystemFn = for<'a> fn(
    &'a SystemAccessors<'a>,
) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send + 'a>>;

/// An internal async system function type
pub(crate) type InternalAsyncSystemFn =
    for<'a> fn(
        &'a InternalSystemAccessors<'a>,
    ) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send + 'a>>;

/// An async system with a name and function
pub struct AsyncSystem {
    name: &'static str,
    func: AsyncSystemFn,
}

/// An internal async system with a name and function
pub(crate) struct InternalAsyncSystem {
    name: &'static str,
    func: InternalAsyncSystemFn,
}

impl AsyncSystem {
    /// Create a new external async system
    pub fn new(name: &'static str, func: AsyncSystemFn) -> Self {
        Self { name, func }
    }

    /// Get the name of the system
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Run the system
    pub async fn run<'a>(&'a self, sa: &'a SystemAccessors<'a>) -> SystemResult<()> {
        (self.func)(sa).await
    }
}

impl InternalAsyncSystem {
    /// Create a new internal async system
    pub(crate) fn new(name: &'static str, func: InternalAsyncSystemFn) -> Self {
        Self { name, func }
    }

    /// Get the name of the system
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Run the system
    pub async fn run<'a>(&'a self, sa: &'a InternalSystemAccessors<'a>) -> SystemResult<()> {
        (self.func)(sa).await
    }
}

/// Helper function for creating external systems
pub fn system(name: &'static str, func: AsyncSystemFn) -> AsyncSystem {
    AsyncSystem::new(name, func)
}

/// Helper function for creating internal systems
pub(crate) fn internal_system(
    name: &'static str,
    func: InternalAsyncSystemFn,
) -> InternalAsyncSystem {
    InternalAsyncSystem::new(name, func)
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
                internal_system(
                    "update_physics_system",
                    update_systems::update_physics_system,
                ),
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
