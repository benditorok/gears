use core::time;
use gears_ecs::{
    components::{self, lights::Light},
    World,
};
use gears_renderer::{light, state::State};
use rayon::str;
use std::future::Future;

pub struct SystemAccessors<'a> {
    pub world: &'a World,
    pub state: &'a State<'a>,
    pub dt: time::Duration,
}

impl<'a> SystemAccessors<'a> {
    pub fn new(world: &'a World, state: &'a State<'a>, dt: time::Duration) -> SystemAccessors<'a> {
        SystemAccessors { world, state, dt }
    }
}

pub struct System {
    pub name: &'static str,
    pub run: Box<dyn Fn(&SystemAccessors) + Send + Sync + 'static>,
}

impl System {
    pub fn new(name: &'static str, run: impl Fn(&SystemAccessors) + Send + Sync + 'static) -> Self {
        Self {
            name,
            run: Box::new(run),
        }
    }

    pub fn run(&self, sa: &SystemAccessors) {
        (self.run)(sa);
    }
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

pub struct SystemCollection {
    pub systems: Vec<System>,
    pub async_systems: Vec<AsyncSystem>,
}

impl Default for SystemCollection {
    fn default() -> Self {
        Self {
            systems: vec![System::new("update_lights", update_lights)],
            async_systems: vec![],
        }
    }
}

/// Update the lights in the scene.
pub fn update_lights(sa: &SystemAccessors) {
    use rayon::prelude::*;

    let light_entities = sa.world.get_entities_with_component::<Light>();

    // Collect light uniforms in parallel
    let light_uniforms: Vec<light::LightUniform> = light_entities
        .par_iter()
        .map(|&entity| {
            let pos = sa
                .world
                .get_component::<components::transforms::Pos3>(entity)
                .unwrap();
            let light_uniform = sa
                .world
                .get_component::<light::LightUniform>(entity)
                .unwrap();

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

    sa.state.queue.write_buffer(
        &sa.state.light_buffer,
        0,
        bytemuck::cast_slice(&[light_data]),
    );
}
