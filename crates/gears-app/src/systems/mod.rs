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
            async_systems: vec![AsyncSystem::new("update_lights", internal_update_lights)],
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

/// Update the lights in the scene.
pub fn internal_update_lights(sa: &SystemAccessors) -> Box<dyn Future<Output = ()> + Send + Unpin> {
    use rayon::prelude::*;

    // Early return if not using internal variant
    let (world, state) = match sa {
        SystemAccessors::Internal {
            world,
            state,
            dt: _,
        } => (world, state),
        _ => return Box::new(std::future::ready(())),
    };

    let light_entities = world.get_entities_with_component::<Light>();

    // Collect light uniforms in parallel
    let light_uniforms: Vec<light::LightUniform> = light_entities
        .par_iter()
        .map(|&entity| {
            let pos = world
                .get_component::<components::transforms::Pos3>(entity)
                .unwrap();
            let light_uniform = world.get_component::<light::LightUniform>(entity).unwrap();

            // Update the light
            {
                let rlock_pos = pos.read().unwrap();
                let mut wlock_light_uniform = light_uniform.write().unwrap();
                wlock_light_uniform.position = [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z];
            }

            let light_value = *light_uniform.read().unwrap();
            light_value
        })
        .collect();

    let num_lights = light_uniforms.len() as u32;
    let light_data = light::LightData {
        lights: {
            let mut array = [light::LightUniform::default(); light::NUM_MAX_LIGHTS as usize];
            array[..light_uniforms.len()].copy_from_slice(&light_uniforms);
            array
        },
        num_lights,
        _padding: [0; 3],
    };

    state
        .queue
        .write_buffer(&state.light_buffer, 0, bytemuck::cast_slice(&[light_data]));

    Box::new(std::future::ready(()))
}
