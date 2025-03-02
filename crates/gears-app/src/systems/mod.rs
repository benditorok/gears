mod update_systems;

use core::time;
use gears_ecs::{
    components::{self, lights::Light},
    World,
};
use gears_renderer::{light, state::State};
use rayon::str;
use std::future::Future;

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

pub struct AsyncSystem {
    pub name: &'static str,
    pub run: Box<
        dyn Fn(&SystemAccessors) -> Box<dyn Future<Output = ()> + Send + Unpin>
            + Send
            + Sync
            + 'static,
    >,
}

impl AsyncSystem {
    pub fn new(
        name: &'static str,
        run: impl Fn(&SystemAccessors) -> Box<dyn Future<Output = ()> + Send + Unpin>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        Self {
            name,
            run: Box::new(run),
        }
    }

    pub fn run(&self, sa: &SystemAccessors) -> Box<dyn Future<Output = ()> + Send + Unpin> {
        (self.run)(sa)
    }
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
                AsyncSystem::new("update_lights", update_systems::update_lights),
                AsyncSystem::new("update_models", update_systems::update_models),
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
