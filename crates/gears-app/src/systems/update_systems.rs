use core::time;
use gears_ecs::{
    components::{self, lights::Light},
    World,
};
use gears_renderer::{light, state::State};
use rayon::str;
use std::future::Future;

use super::SystemAccessors;

/// Update the lights in the scene.
pub fn update_lights(sa: &SystemAccessors) -> Box<dyn Future<Output = ()> + Send + Unpin> {
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
