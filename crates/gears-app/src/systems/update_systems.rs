use super::SystemAccessors;
use cgmath::VectorSpace;
use gears_ecs::components::{self, lights::Light, misc::Marker};
use gears_renderer::{instance, light, model, BufferComponent};
use log::warn;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

/// Update the lights in the scene.
pub(super) async fn update_lights<'a>(sa: &'a SystemAccessors<'a>) {
    // Early return if not using internal variant
    let (world, state) = match sa {
        SystemAccessors::Internal {
            world,
            state,
            dt: _,
        } => (world, state),
        _ => return,
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
}

/// Update the models in the scene.
pub(super) async fn update_models<'a>(sa: &'a SystemAccessors<'a>) {
    // Early return if not using internal variant
    let (world, state, dt) = match sa {
        SystemAccessors::Internal { world, state, dt } => (world, state, dt),
        _ => return,
    };

    let model_entities = world.get_entities_with_component::<components::misc::StaticModelMarker>();

    let instances = model_entities.par_iter().map(|&entity| {
        let name = world
            .get_component::<components::misc::Name>(entity)
            .unwrap_or_else(|| panic!("{}", components::misc::StaticModelMarker::describe()));
        let pos3 = world
            .get_component::<components::transforms::Pos3>(entity)
            .unwrap_or_else(|| panic!("{}", components::misc::StaticModelMarker::describe()));
        let instance = world
            .get_component::<instance::Instance>(entity)
            .unwrap_or_else(|| panic!("{}", components::misc::StaticModelMarker::describe()));
        let buffer = world.get_component::<BufferComponent>(entity).unwrap();
        let model = world.get_component::<model::Model>(entity).unwrap();
        let animation_queue = world.get_component::<components::misc::AnimationQueue>(entity);

        // ! Animations testing
        if let Some(animation_queue) = animation_queue {
            // * This will run if an animation is queued
            if let Some(selected_animation) = animation_queue.write().unwrap().pop() {
                let rlock_model = model.read().unwrap();

                // Get the current time of the animation
                let mut wlock_animation_queue = animation_queue.write().unwrap();
                let current_time = wlock_animation_queue.time.elapsed().as_secs_f32();

                let animation = &rlock_model.get_animation(selected_animation).unwrap();
                let mut current_keyframe_index = 0;

                // Find the two keyframes surrounding the current_time
                for (i, timestamp) in animation.timestamps.iter().enumerate() {
                    if *timestamp > current_time {
                        current_keyframe_index = i - 1;
                        break;
                    }
                    current_keyframe_index = i;
                }

                // Loop the animation
                if current_keyframe_index >= animation.timestamps.len() - 1 {
                    wlock_animation_queue.time = Instant::now(); // TODO this should be stored per component
                    current_keyframe_index = 0;
                }

                let next_keyframe_index = current_keyframe_index + 1;
                let t0 = animation.timestamps[current_keyframe_index];
                let t1 = animation.timestamps[next_keyframe_index];
                let factor = (current_time - t0) / (t1 - t0);

                // TODO animations should also take positions into consideration while playing
                let current_animation = &animation.keyframes;
                match current_animation {
                    model::Keyframes::Translation(frames) => {
                        let start_frame = &frames[current_keyframe_index];
                        let end_frame = &frames[next_keyframe_index];

                        // Ensure frames have exactly 3 elements
                        if start_frame.len() == 3 && end_frame.len() == 3 {
                            let start = cgmath::Vector3::new(
                                start_frame[0],
                                start_frame[1],
                                start_frame[2],
                            );
                            let end =
                                cgmath::Vector3::new(end_frame[0], end_frame[1], end_frame[2]);
                            let interpolated = start.lerp(end, factor);
                            let mut wlock_instance = instance.write().unwrap();
                            wlock_instance.position = interpolated;
                        } else {
                            warn!("Translation frames do not have exactly 3 elements.");
                        }
                    }
                    model::Keyframes::Rotation(quats) => {
                        let start_quat = &quats[current_keyframe_index];
                        let end_quat = &quats[next_keyframe_index];

                        // Ensure quaternions have exactly 4 elements
                        if start_quat.len() == 4 && end_quat.len() == 4 {
                            let start = cgmath::Quaternion::new(
                                start_quat[0],
                                start_quat[1],
                                start_quat[2],
                                start_quat[3],
                            );
                            let end = cgmath::Quaternion::new(
                                end_quat[0],
                                end_quat[1],
                                end_quat[2],
                                end_quat[3],
                            );
                            let interpolated = start.slerp(end, factor);
                            let mut wlock_instance = instance.write().unwrap();
                            wlock_instance.rotation = interpolated;
                        } else {
                            warn!("Rotation quaternions do not have exactly 4 elements.");
                        }
                    }
                    model::Keyframes::Scale(_) => {
                        // Handle scale interpolation if necessary
                    }
                    model::Keyframes::Other => {
                        warn!("Other animations are not supported yet!")
                    }
                }
            } else {
                // If the AnimationQueue is emtpy
                // ! Do not remove, causes deadlock if the lock is held for more
                {
                    let mut wlock_instance = instance.write().unwrap();
                    let rlock_pos3 = pos3.read().unwrap();

                    wlock_instance.position = rlock_pos3.pos;
                    wlock_instance.rotation = rlock_pos3.rot;
                }
            }
        } else {
            // If there is no AnimationQueue
            // ! Do not remove, causes deadlock if the lock is held for more
            {
                let mut wlock_instance = instance.write().unwrap();
                let rlock_pos3 = pos3.read().unwrap();

                wlock_instance.position = rlock_pos3.pos;
                wlock_instance.rotation = rlock_pos3.rot;
            }
        }

        let instance_raw = instance.read().unwrap().to_raw();

        state.queue.write_buffer(
            &buffer.write().unwrap(),
            0,
            bytemuck::cast_slice(&[instance_raw]),
        );
    });
}
