mod update_systems;

use core::time;
use gears_ecs::{
    components::{self, lights::Light},
    World,
};
use gears_renderer::{light, state::State};
use std::{future::Future, pin::Pin};

pub enum SystemAccessors<'a> {
    Internal {
        world: &'a World,
        state: &'a State<'a>,
        dt: time::Duration,
    },
    External {
        world: &'a World,
        dt: time::Duration,
    },
}

// Modified trait for async systems with proper lifetime handling
pub trait AsyncSystemFn: Send + Sync {
    fn run<'a>(
        &'a self,
        sa: &'a SystemAccessors<'a>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
}

// Helper struct to implement AsyncSystemFn for closures
pub struct AsyncFnPointer<F> {
    func: F,
}

// Implementation for async closures
impl<F, Fut> AsyncSystemFn for AsyncFnPointer<F>
where
    F: for<'r> Fn(&'r SystemAccessors<'r>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn run<'a>(
        &'a self,
        sa: &'a SystemAccessors<'a>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        // Create a future adapter that properly handles lifetimes
        Box::pin(async move {
            (self.func)(sa).await;
        })
    }
}

// A new wrapper specifically for async closures
pub struct AsyncClosureWrapper<F> {
    func: F,
}

impl<F> AsyncSystemFn for AsyncClosureWrapper<F>
where
    F: for<'r> FnOnce(&'r SystemAccessors<'r>) -> Pin<Box<dyn Future<Output = ()> + Send + 'r>>
        + Clone
        + Send
        + Sync
        + 'static,
{
    fn run<'a>(
        &'a self,
        sa: &'a SystemAccessors<'a>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        // Clone the closure to move it into the async block
        let func = self.func.clone();
        Box::pin(async move {
            func(sa).await;
        })
    }
}

// Wrapper for function pointers
pub struct AsyncFnSystem<F> {
    func: F,
}

// Implementation for direct function pointers
impl<F> AsyncSystemFn for AsyncFnSystem<F>
where
    F: for<'r> Fn(&'r SystemAccessors<'r>) -> Pin<Box<dyn Future<Output = ()> + Send + 'r>>
        + Send
        + Sync
        + 'static,
{
    fn run<'a>(
        &'a self,
        sa: &'a SystemAccessors<'a>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        (self.func)(sa)
    }
}

// Unified system struct that contains name and boxed function
pub struct AsyncSystem {
    name: &'static str,
    func: Box<dyn AsyncSystemFn + Send + Sync>,
}

impl AsyncSystem {
    pub fn new<F>(name: &'static str, func: F) -> Self
    where
        F: AsyncSystemFn + Send + Sync + 'static,
    {
        Self {
            name,
            func: Box::new(func),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub async fn run<'a>(&'a self, sa: &'a SystemAccessors<'a>) {
        self.func.run(sa).await
    }
}

// Helper function for direct function pointers
pub fn system_fn<F>(name: &'static str, func: F) -> AsyncSystem
where
    F: for<'r> Fn(&'r SystemAccessors<'r>) -> Pin<Box<dyn Future<Output = ()> + Send + 'r>>
        + Send
        + Sync
        + 'static,
{
    AsyncSystem::new(name, AsyncFnSystem { func })
}

// Helper function for async closures
pub fn system<F, Fut>(name: &'static str, func: F) -> AsyncSystem
where
    F: for<'r> Fn(&'r SystemAccessors<'r>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    AsyncSystem::new(name, AsyncFnPointer { func })
}

// Helper function specifically for async closures
pub fn async_system<F>(name: &'static str, func: F) -> AsyncSystem
where
    F: for<'r> FnOnce(&'r SystemAccessors<'r>) -> Pin<Box<dyn Future<Output = ()> + Send + 'r>>
        + Clone
        + Send
        + Sync
        + 'static,
{
    AsyncSystem::new(name, AsyncClosureWrapper { func })
}

pub trait SystemCollection {
    fn add_system(&mut self, system: AsyncSystem);
    fn systems(&self) -> &[AsyncSystem];
}

pub(crate) struct InternalSystemCollection {
    pub async_systems: Vec<AsyncSystem>,
}

impl Default for InternalSystemCollection {
    fn default() -> Self {
        Self {
            async_systems: vec![
                // Use separate wrappers for update_lights and update_models
                system_fn("update_lights", |sa| {
                    Box::pin(update_systems::update_lights(sa))
                }),
                system_fn("update_models", |sa| {
                    Box::pin(update_systems::update_models(sa))
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

pub struct ExternalSystemCollection {
    pub async_systems: Vec<AsyncSystem>,
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
