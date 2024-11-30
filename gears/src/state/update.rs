use super::State;
use crate::{ecs::components, renderer::light};

/// Update the lights in the scene.
///
/// # Returns
///
/// A future which can be awaited.
pub(crate) fn update_lights(state: &mut State) {
    if let Some(light_entities) = &state.light_entities {
        let mut light_uniforms: Vec<light::LightUniform> = Vec::new();

        for entity in light_entities {
            let ecs_lock = state.ecs.lock().unwrap();

            let pos = ecs_lock
                .get_component_from_entity::<components::transforms::Pos3>(*entity)
                .unwrap();
            let light_uniform = ecs_lock
                .get_component_from_entity::<light::LightUniform>(*entity)
                .unwrap();
            let light = ecs_lock
                .get_component_from_entity::<components::lights::Light>(*entity)
                .unwrap();

            {
                // TODO update the colors
                let rlock_pos = pos.read().unwrap();
                let mut wlock_light_uniform = light_uniform.write().unwrap();

                wlock_light_uniform.position = [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z];
            }

            let rlock_light_uniform = light_uniform.read().unwrap();

            light_uniforms.push(*rlock_light_uniform);
        }

        let num_lights = light_uniforms.len() as u32;

        let light_data = light::LightData {
            lights: {
                let mut array = [light::LightUniform::default(); light::NUM_MAX_LIGHTS as usize];
                for (i, light) in light_uniforms.iter().enumerate() {
                    array[i] = *light;
                }
                array
            },
            num_lights,
            _padding: [0; 3],
        };

        state
            .queue
            .write_buffer(&state.light_buffer, 0, bytemuck::cast_slice(&[light_data]));
    }
}
