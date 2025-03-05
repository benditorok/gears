mod update_systems;

use core::time;
use gears_ecs::{
    components::{self, lights::Light},
    World,
};
use gears_renderer::{light, state::State};
use rayon::str;
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

// The core trait that both functions and closures will implement
pub trait AsyncSystemFn {
    fn call<'a>(
        &'a self,
        sa: &'a SystemAccessors<'a>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
}

// Store either a function pointer or a boxed closure
pub enum AsyncSystem {
    Fn {
        name: &'static str,
        run: fn(&SystemAccessors) -> Pin<Box<dyn Future<Output = ()> + Send>>,
    },
    Closure {
        name: &'static str,
        run: Box<dyn AsyncSystemFn + Send + Sync>,
    },
}

// Implement the trait for function pointers
impl AsyncSystemFn for fn(&SystemAccessors) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    fn call<'a>(
        &'a self,
        sa: &'a SystemAccessors<'a>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        self(sa)
    }
}

// Implement the trait for async closures
impl<F> AsyncSystemFn for F
where
    F: for<'a> Fn(&'a SystemAccessors<'a>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>,
{
    fn call<'a>(
        &'a self,
        sa: &'a SystemAccessors<'a>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        self(sa)
    }
}

impl AsyncSystem {
    // Simple constructor for regular async functions
    pub fn new_fn(
        name: &'static str,
        f: fn(&SystemAccessors) -> Pin<Box<dyn Future<Output = ()> + Send>>,
    ) -> Self {
        Self::Fn { name, run: f }
    }

    // Helper for async closures that handles the boxing
    pub fn new_closure<F>(name: &'static str, f: F) -> Self
    where
        F: AsyncSystemFn + Send + Sync + 'static,
    {
        Self::Closure {
            name,
            run: Box::new(f),
        }
    }

    pub async fn run<'a>(&'a self, sa: &'a SystemAccessors<'a>) {
        match self {
            Self::Fn { run, .. } => run(sa).await,
            Self::Closure { run, .. } => run.call(sa).await,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Fn { name, .. } => name,
            Self::Closure { name, .. } => name,
        }
    }
}

// Add this helper function
fn into_system_future<F>(f: F) -> Pin<Box<dyn Future<Output = ()> + Send>>
where
    F: Future<Output = ()> + Send + 'static,
{
    Box::pin(f) as Pin<Box<dyn Future<Output = ()> + Send>>
}

// Update the system helper function
pub fn system<F, Fut>(name: &'static str, f: F) -> AsyncSystem
where
    F: for<'a> Fn(&'a SystemAccessors<'a>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    AsyncSystem::new_closure(name, move |sa| {
        into_system_future(async move { f(sa).await })
    })
}

// Usage examples:
async fn test_system(sa: &SystemAccessors) {
    // system logic here
}

// Register systems like this:
fn register_systems(collection: &mut impl SystemCollection) {
    // Regular async function
    collection.add_system(AsyncSystem::new_fn("test", |sa| Box::pin(test_system(sa))));

    // Async closure
    collection.add_system(system("update", async move |sa| {
        // system logic here
    }));
}

pub trait SystemCollection {
    fn add_system(&mut self, system: AsyncSystem);
    fn systems(&self) -> &[AsyncSystem];
}

pub(crate) struct InternalSystemCollection {
    pub async_systems: Vec<AsyncSystem>,
}

// Update the default implementation to use the new enum
impl Default for InternalSystemCollection {
    fn default() -> Self {
        Self {
            async_systems: vec![
                // Regular async functions use from_fn
                AsyncSystem::new_fn("update_lights", |sa| {
                    Box::pin(update_systems::update_lights(sa))
                }),
                AsyncSystem::new_fn("update_models", |sa| {
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
