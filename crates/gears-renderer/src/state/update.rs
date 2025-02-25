use super::{instance, light, model, State};
use crate::BufferComponent;
use cgmath::VectorSpace;
use gears_ecs::{
    components::{self, misc::Marker, physics::AABBCollisionBox},
    Component,
};
use log::warn;
use std::time::{self, Instant};

// /// Update the lights in the scene.
// pub(super) fn lights(state: &mut State) {
//     if let Some(light_entities) = &state.light_entities {
//         let mut light_uniforms: Vec<light::LightUniform> = Vec::new();

//         for entity in light_entities {
//             let pos = state
//                 .world
//                 .get_component::<components::transforms::Pos3>(*entity)
//                 .unwrap();
//             let light_uniform = state
//                 .world
//                 .get_component::<light::LightUniform>(*entity)
//                 .unwrap();
//             let light = state
//                 .world
//                 .get_component::<components::lights::Light>(*entity)
//                 .unwrap();

//             {
//                 // TODO update the colors
//                 let rlock_pos = pos.read().unwrap();
//                 let mut wlock_light_uniform = light_uniform.write().unwrap();

//                 wlock_light_uniform.position = [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z];
//             }

//             let rlock_light_uniform = light_uniform.read().unwrap();

//             light_uniforms.push(*rlock_light_uniform);
//         }

//         let num_lights = light_uniforms.len() as u32;

//         let light_data = light::LightData {
//             lights: {
//                 let mut array = [light::LightUniform::default(); light::NUM_MAX_LIGHTS as usize];
//                 for (i, light) in light_uniforms.iter().enumerate() {
//                     array[i] = *light;
//                 }
//                 array
//             },
//             num_lights,
//             _padding: [0; 3],
//         };

//         state
//             .queue
//             .write_buffer(&state.light_buffer, 0, bytemuck::cast_slice(&[light_data]));
//     }
// }

/// Update the models in the scene.
pub(super) fn models(state: &mut State) {
    if let Some(model_entities) = &state.static_model_entities {
        for entity in model_entities {
            let name = state
                .world
                .get_component::<components::misc::Name>(*entity)
                .unwrap_or_else(|| panic!("{}", components::misc::StaticModelMarker::describe()));
            let pos3 = state
                .world
                .get_component::<components::transforms::Pos3>(*entity)
                .unwrap_or_else(|| panic!("{}", components::misc::StaticModelMarker::describe()));
            let instance = state
                .world
                .get_component::<instance::Instance>(*entity)
                .unwrap_or_else(|| panic!("{}", components::misc::StaticModelMarker::describe()));
            let buffer = state
                .world
                .get_component::<BufferComponent>(*entity)
                .unwrap();
            let model = state.world.get_component::<model::Model>(*entity).unwrap();
            let animation_queue = state
                .world
                .get_component::<components::misc::AnimationQueue>(*entity);

            // ! Animations testing
            if let Some(animation_queue) = animation_queue {
                // * This will run if an animation is queued
                if let Some(selected_animation) = animation_queue.write().unwrap().pop() {
                    let rlock_model = model.read().unwrap();

                    let current_time = state.time.elapsed().as_secs_f32();
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
                        state.time = Instant::now();
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
        }
    }
}

pub(super) fn physics_system(state: &mut State, dt: time::Duration) {
    let dt = dt.as_secs_f32();
    let mut physics_bodies = Vec::new();

    if let Some(player) = state.player_entity {
        physics_bodies.push((
            player,
            state
                .world
                .get_component::<components::physics::RigidBody<AABBCollisionBox>>(player)
                .unwrap(),
            state
                .world
                .get_component::<components::transforms::Pos3>(player)
                .unwrap(),
        ));
    }

    if let Some(physics_entities) = &state.physics_entities {
        for entity in physics_entities {
            // TODO add an animation queue for physics entities as well

            let physics_body = state
                .world
                .get_component::<components::physics::RigidBody<AABBCollisionBox>>(*entity)
                .unwrap();

            let instance = state
                .world
                .get_component::<instance::Instance>(*entity)
                .unwrap();
            let buffer = state
                .world
                .get_component::<BufferComponent>(*entity)
                .unwrap();
            let pos3 = state
                .world
                .get_component::<components::transforms::Pos3>(*entity)
                .unwrap();

            {
                let mut wlock_instance = instance.write().unwrap();
                let rlock_pos3 = pos3.read().unwrap();

                wlock_instance.position = rlock_pos3.pos;
                wlock_instance.rotation = rlock_pos3.rot
            }

            let instance_raw = instance.read().unwrap().to_raw();
            state.queue.write_buffer(
                &buffer.write().unwrap(),
                0,
                bytemuck::cast_slice(&[instance_raw]),
            );

            physics_bodies.push((*entity, physics_body, pos3));
        }
    }

    // Update positions and velocities based on acceleration
    for (entity, physics_body, pos3) in &physics_bodies {
        let mut wlock_physics_body = physics_body.write().unwrap();
        let mut wlock_pos3 = pos3.write().unwrap();

        wlock_physics_body.update_pos(&mut wlock_pos3, dt);
    }

    // Check for collisions and resolve them
    for i in 0..physics_bodies.len() {
        for j in (i + 1)..physics_bodies.len() {
            let (_entity_a, physics_body_a, pos3_a) = &physics_bodies[i];
            let (_entity_b, physics_body_b, pos3_b) = &physics_bodies[j];

            let mut wlock_physics_body_a = physics_body_a.write().unwrap();
            let mut wlock_physics_body_b = physics_body_b.write().unwrap();
            let mut wlock_pos3_a = pos3_a.write().unwrap();
            let mut wlock_pos3_b = pos3_b.write().unwrap();

            components::physics::RigidBody::check_and_resolve_collision(
                &mut wlock_physics_body_a,
                &mut wlock_pos3_a,
                &mut wlock_physics_body_b,
                &mut wlock_pos3_b,
            );
        }
    }
}
