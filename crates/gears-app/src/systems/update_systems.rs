use super::{SystemAccessors, SystemError, SystemResult};
use cgmath::VectorSpace;
use gears_ecs::components::physics::AABBCollisionBox;
use gears_ecs::components::{self, lights::Light};
use gears_renderer::{instance, light, model, BufferComponent};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::time::Instant;

/// Update the lights in the scene.
pub(super) async fn update_lights<'a>(sa: &'a SystemAccessors<'a>) -> SystemResult<()> {
    // Early return if not using internal variant
    let (world, state) = match sa {
        SystemAccessors::Internal {
            world,
            state,
            dt: _,
        } => (world, state),
        _ => return Ok(()),
    };

    let light_entities = world.get_entities_with_component::<Light>();

    // Collect light uniforms in parallel
    let light_uniforms: Vec<light::LightUniform> = light_entities
        .par_iter()
        .map(|&entity| {
            let pos = world
                .get_component::<components::transforms::Pos3>(entity)
                .ok_or_else(|| {
                    SystemError::MissingComponent(format!(
                        "Pos3 component missing for light entity {:?}",
                        entity
                    ))
                })?;
            let light_uniform = world
                .get_component::<light::LightUniform>(entity)
                .ok_or_else(|| {
                    SystemError::MissingComponent(format!(
                        "LightUniform component missing for entity {:?}",
                        entity
                    ))
                })?;

            // Update the light
            {
                let rlock_pos = pos.read().map_err(|e| {
                    SystemError::ComponentAccess(format!("Failed to read Pos3: {}", e))
                })?;
                let mut wlock_light_uniform = light_uniform.write().map_err(|e| {
                    SystemError::ComponentAccess(format!("Failed to write LightUniform: {}", e))
                })?;
                wlock_light_uniform.position = [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z];
            }

            let light_value = light_uniform.read().map_err(|e| {
                SystemError::ComponentAccess(format!("Failed to read LightUniform: {}", e))
            })?;
            Ok(*light_value)
        })
        .collect::<Result<Vec<light::LightUniform>, SystemError>>()?;

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

    Ok(())
}

/// Update the models in the scene.
pub(super) async fn update_models<'a>(sa: &'a SystemAccessors<'a>) -> SystemResult<()> {
    // Early return if not using internal variant
    let (world, state, dt) = match sa {
        SystemAccessors::Internal { world, state, dt } => (world, state, dt),
        _ => return Ok(()),
    };

    let model_entities = world.get_entities_with_component::<components::misc::StaticModelMarker>();

    let results: Vec<SystemResult<()>> = model_entities
        .par_iter()
        .map(|&entity| {
            let name = world
                .get_component::<components::misc::Name>(entity)
                .ok_or_else(|| {
                    SystemError::MissingComponent(format!(
                        "Name component missing for entity {:?} with StaticModelMarker",
                        entity
                    ))
                })?;
            let pos3 = world
                .get_component::<components::transforms::Pos3>(entity)
                .ok_or_else(|| {
                    SystemError::MissingComponent(format!(
                        "Pos3 component missing for entity {:?} with StaticModelMarker",
                        entity
                    ))
                })?;
            let instance = world
                .get_component::<instance::Instance>(entity)
                .ok_or_else(|| {
                    SystemError::MissingComponent(format!(
                        "Instance component missing for entity {:?} with StaticModelMarker",
                        entity
                    ))
                })?;
            let buffer = world
                .get_component::<BufferComponent>(entity)
                .ok_or_else(|| {
                    SystemError::MissingComponent(format!(
                        "BufferComponent missing for entity {:?} with StaticModelMarker",
                        entity
                    ))
                })?;
            let model = world.get_component::<model::Model>(entity).ok_or_else(|| {
                SystemError::MissingComponent(format!(
                    "Model component missing for entity {:?} with StaticModelMarker",
                    entity
                ))
            })?;
            let animation_queue = world.get_component::<components::misc::AnimationQueue>(entity);

            // ! Animations testing
            if let Some(animation_queue) = animation_queue {
                // * This will run if an animation is queued
                let mut wlock_animation_queue = animation_queue.write().map_err(|e| {
                    SystemError::ComponentAccess(format!("Failed to write AnimationQueue: {}", e))
                })?;

                if let Some(selected_animation) = wlock_animation_queue.pop() {
                    let rlock_model = model.read().map_err(|e| {
                        SystemError::ComponentAccess(format!("Failed to read Model: {}", e))
                    })?;

                    // Get the current time of the animation
                    let current_time = wlock_animation_queue.time.elapsed().as_secs_f32();

                    let animation = &rlock_model
                        .get_animation(selected_animation)
                        .map_err(SystemError::Animation)?;
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
                        wlock_animation_queue.time = Instant::now();
                        current_keyframe_index = 0;
                    }

                    let next_keyframe_index = current_keyframe_index + 1;
                    if next_keyframe_index >= animation.timestamps.len() {
                        return Err(SystemError::Animation(
                            "Invalid animation keyframe index".to_string(),
                        ));
                    }
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
                                let mut wlock_instance = instance.write().map_err(|e| {
                                    SystemError::ComponentAccess(format!(
                                        "Failed to write Instance: {}",
                                        e
                                    ))
                                })?;
                                wlock_instance.position = interpolated;
                            } else {
                                return Err(SystemError::Animation(
                                    "Translation frames do not have exactly 3 elements".to_string(),
                                ));
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
                                let mut wlock_instance = instance.write().map_err(|e| {
                                    SystemError::ComponentAccess(format!(
                                        "Failed to write Instance: {}",
                                        e
                                    ))
                                })?;
                                wlock_instance.rotation = interpolated;
                            } else {
                                return Err(SystemError::Animation(
                                    "Rotation quaternions do not have exactly 4 elements"
                                        .to_string(),
                                ));
                            }
                        }
                        model::Keyframes::Scale(_) => {
                            // Handle scale interpolation if necessary
                        }
                        model::Keyframes::Other => {
                            return Err(SystemError::Animation(
                                "Other animation types are not supported yet".to_string(),
                            ));
                        }
                    }
                } else {
                    // If the AnimationQueue is empty
                    // ! Do not remove, causes deadlock if the lock is held for more
                    {
                        let mut wlock_instance = instance.write().map_err(|e| {
                            SystemError::ComponentAccess(format!("Failed to write Instance: {}", e))
                        })?;
                        let rlock_pos3 = pos3.read().map_err(|e| {
                            SystemError::ComponentAccess(format!("Failed to read Pos3: {}", e))
                        })?;

                        wlock_instance.position = rlock_pos3.pos;
                        wlock_instance.rotation = rlock_pos3.rot;
                    }
                }
            } else {
                // If there is no AnimationQueue
                // ! Do not remove, causes deadlock if the lock is held for more
                {
                    let mut wlock_instance = instance.write().map_err(|e| {
                        SystemError::ComponentAccess(format!("Failed to write Instance: {}", e))
                    })?;
                    let rlock_pos3 = pos3.read().map_err(|e| {
                        SystemError::ComponentAccess(format!("Failed to read Pos3: {}", e))
                    })?;

                    wlock_instance.position = rlock_pos3.pos;
                    wlock_instance.rotation = rlock_pos3.rot;
                }
            }

            let instance_raw = instance
                .read()
                .map_err(|e| {
                    SystemError::ComponentAccess(format!("Failed to read Instance: {}", e))
                })?
                .to_raw();

            let buffer_guard = buffer.write().map_err(|e| {
                SystemError::ComponentAccess(format!("Failed to write Buffer: {}", e))
            })?;

            state
                .queue
                .write_buffer(&buffer_guard.0, 0, bytemuck::cast_slice(&[instance_raw]));

            Ok(())
        })
        .collect();

    // Check for any errors
    for result in results {
        result?;
    }

    Ok(())
}

pub(super) async fn physics_system<'a>(sa: &'a SystemAccessors<'a>) -> SystemResult<()> {
    // Early return if not using internal variant
    let (world, state, dt) = match sa {
        SystemAccessors::Internal { world, state, dt } => (world, state, dt),
        _ => return Ok(()),
    };

    let dt = dt.as_secs_f32();
    let mut physics_bodies = Vec::new();

    // Get all entities with RigidBody component
    let physics_entities =
        world.get_entities_with_component::<components::physics::RigidBody<AABBCollisionBox>>();

    for &entity in physics_entities.iter() {
        let physics_body = world
            .get_component::<components::physics::RigidBody<AABBCollisionBox>>(entity)
            .ok_or_else(|| {
                SystemError::MissingComponent(format!(
                    "RigidBody component missing for entity {:?}",
                    entity
                ))
            })?;
        let pos3 = world
            .get_component::<components::transforms::Pos3>(entity)
            .ok_or_else(|| {
                SystemError::MissingComponent(format!(
                    "Pos3 component missing for entity {:?} with RigidBody",
                    entity
                ))
            })?;

        // Update positions and velocities based on acceleration
        {
            let mut wlock_physics_body = physics_body.write().map_err(|e| {
                SystemError::ComponentAccess(format!("Failed to write RigidBody: {}", e))
            })?;
            let mut wlock_pos3 = pos3.write().map_err(|e| {
                SystemError::ComponentAccess(format!("Failed to write Pos3: {}", e))
            })?;
            wlock_physics_body.update_pos(&mut wlock_pos3, dt);
        }

        physics_bodies.push((entity, physics_body, pos3));
    }

    // Check for collisions and resolve them
    for i in 0..physics_bodies.len() {
        for j in (i + 1)..physics_bodies.len() {
            let (_entity_a, physics_body_a, pos3_a) = &physics_bodies[i];
            let (_entity_b, physics_body_b, pos3_b) = &physics_bodies[j];

            let mut wlock_physics_body_a = physics_body_a.write().map_err(|e| {
                SystemError::ComponentAccess(format!("Failed to write RigidBody A: {}", e))
            })?;
            let mut wlock_physics_body_b = physics_body_b.write().map_err(|e| {
                SystemError::ComponentAccess(format!("Failed to write RigidBody B: {}", e))
            })?;
            let mut wlock_pos3_a = pos3_a.write().map_err(|e| {
                SystemError::ComponentAccess(format!("Failed to write Pos3 A: {}", e))
            })?;
            let mut wlock_pos3_b = pos3_b.write().map_err(|e| {
                SystemError::ComponentAccess(format!("Failed to write Pos3 B: {}", e))
            })?;

            components::physics::RigidBody::check_and_resolve_collision(
                &mut wlock_physics_body_a,
                &mut wlock_pos3_a,
                &mut wlock_physics_body_b,
                &mut wlock_pos3_b,
            );
        }
    }

    // Update instance data
    let results: Vec<SystemResult<()>> = physics_entities
        .par_iter()
        .map(|&entity| {
            if let (Some(instance), Some(buffer), Some(pos3)) = (
                world.get_component::<instance::Instance>(entity),
                world.get_component::<BufferComponent>(entity),
                world.get_component::<components::transforms::Pos3>(entity),
            ) {
                let mut wlock_instance = instance.write().map_err(|e| {
                    SystemError::ComponentAccess(format!("Failed to write Instance: {}", e))
                })?;
                let rlock_pos3 = pos3.read().map_err(|e| {
                    SystemError::ComponentAccess(format!("Failed to read Pos3: {}", e))
                })?;

                wlock_instance.position = rlock_pos3.pos;
                wlock_instance.rotation = rlock_pos3.rot;

                let instance_raw = wlock_instance.to_raw();
                let buffer_guard = buffer.write().map_err(|e| {
                    SystemError::ComponentAccess(format!("Failed to write Buffer: {}", e))
                })?;
                state
                    .queue
                    .write_buffer(&buffer_guard.0, 0, bytemuck::cast_slice(&[instance_raw]));
            }
            Ok(())
        })
        .collect();

    // Check for any errors
    for result in results {
        result?;
    }

    Ok(())
}
