use super::{SystemError, SystemResult};

use core::time;
use gears_ecs::World;
use gears_ecs::components::physics::AABBCollisionBox;
use gears_ecs::components::{self, lights::Light};
use gears_renderer::light::LightUniform;
use gears_renderer::state::State;
use gears_renderer::{BufferComponent, animation, instance, light, model};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

/// Update the lights in the scene.
pub(super) fn update_lights(
    world: Arc<World>,
    state: Arc<RwLock<State>>,
    _dt: time::Duration,
) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>> {
    Box::pin(async move {
        let state = state.read().unwrap();
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

                let light = world.get_component::<Light>(entity).ok_or_else(|| {
                    SystemError::MissingComponent(format!(
                        "Light component missing for entity {:?}",
                        entity
                    ))
                })?;

                let pos = pos.read().unwrap();
                let light = light.read().unwrap();

                let light_uniform = LightUniform::from_components(&light, &pos);

                Ok(light_uniform)
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
    })
}

/// Update the models in the scene.
pub(super) fn update_models(
    world: Arc<World>,
    state: Arc<RwLock<State>>,
    _dt: time::Duration,
) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>> {
    Box::pin(async move {
        let state = state.write().unwrap();
        let model_entities =
            world.get_entities_with_component::<components::misc::StaticModelMarker>();

        let results: Vec<SystemResult<()>> = model_entities
            .par_iter()
            .map(|&entity| {
                let _name = world
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
                let animation_queue =
                    world.get_component::<components::misc::AnimationQueue>(entity);

                // New animation system handling
                if let Some(animation_queue) = animation_queue {
                    let mut wlock_animation_queue = animation_queue.write().map_err(|e| {
                        SystemError::ComponentAccess(format!(
                            "Failed to write AnimationQueue: {}",
                            e
                        ))
                    })?;

                    // Check if we need to start a new animation
                    if wlock_animation_queue.current_animation().is_none()
                        && wlock_animation_queue.has_queued_animations()
                    {
                        if let Some(next_animation) = wlock_animation_queue.play_next() {
                            log::info!("Starting animation: {}", next_animation);
                        }
                    }

                    // Process current animation if one is playing
                    if let Some(current_anim_name) = wlock_animation_queue.current_animation() {
                        let rlock_model = model.read().map_err(|e| {
                            SystemError::ComponentAccess(format!("Failed to read Model: {}", e))
                        })?;

                        // Find the animation in the model
                        if let Ok(animation_clip) = rlock_model.get_animation(current_anim_name) {
                            // Get current animation time
                            let current_time = wlock_animation_queue.time.elapsed().as_secs_f32();

                            // Log animation progress every second for visibility
                            if current_time as i32 != ((current_time - 0.016) as i32) {
                                log::info!(
                                    "Animation '{}' progress: {:.1}s / {:.1}s ({:.0}%)",
                                    current_anim_name,
                                    current_time,
                                    animation_clip.duration,
                                    (current_time / animation_clip.duration * 100.0).min(100.0)
                                );
                            }

                            // Check if animation is finished
                            if current_time >= animation_clip.duration {
                                wlock_animation_queue.is_current_finished = true;

                                if wlock_animation_queue.auto_transition
                                    && wlock_animation_queue.has_queued_animations()
                                {
                                    // Auto-play next animation
                                    if let Some(next_animation) = wlock_animation_queue.play_next()
                                    {
                                        log::info!(
                                            "Auto-transitioning to animation: {}",
                                            next_animation
                                        );
                                    }
                                } else {
                                    // Stop current animation
                                    wlock_animation_queue.stop_current();
                                }
                            } else {
                                // Sample the animation at current time
                                let animation_values = animation_clip.sample(current_time);

                                let mut wlock_instance = instance.write().map_err(|e| {
                                    SystemError::ComponentAccess(format!(
                                        "Failed to write Instance: {}",
                                        e
                                    ))
                                })?;

                                // Apply animation values to instance
                                for (target, value) in animation_values {
                                    match target {
                                        animation::AnimationTarget::Translation => {
                                            if let Some(translation) = value.as_vector3() {
                                                wlock_instance.position = translation;
                                            }
                                        }
                                        animation::AnimationTarget::Rotation => {
                                            if let Some(rotation) = value.as_quaternion() {
                                                wlock_instance.rotation = rotation;
                                            }
                                        }
                                        animation::AnimationTarget::Scale => {
                                            // Handle scale if needed in the future
                                        }
                                        animation::AnimationTarget::Custom(_) => {
                                            // Handle custom properties if needed
                                        }
                                    }
                                }
                            }
                        } else {
                            // Animation not found, fall back to position
                            log::warn!(
                                "Animation '{}' not found in model, falling back to Pos3",
                                current_anim_name
                            );

                            let mut wlock_instance = instance.write().map_err(|e| {
                                SystemError::ComponentAccess(format!(
                                    "Failed to write Instance: {}",
                                    e
                                ))
                            })?;
                            let rlock_pos3 = pos3.read().map_err(|e| {
                                SystemError::ComponentAccess(format!("Failed to read Pos3: {}", e))
                            })?;

                            wlock_instance.position = rlock_pos3.pos;
                            wlock_instance.rotation = rlock_pos3.rot;
                        }
                    } else {
                        // No animation playing, use position component
                        let mut wlock_instance = instance.write().map_err(|e| {
                            SystemError::ComponentAccess(format!("Failed to write Instance: {}", e))
                        })?;
                        let rlock_pos3 = pos3.read().map_err(|e| {
                            SystemError::ComponentAccess(format!("Failed to read Pos3: {}", e))
                        })?;

                        wlock_instance.position = rlock_pos3.pos;
                        wlock_instance.rotation = rlock_pos3.rot;
                    }
                } else {
                    // No AnimationQueue component, use position directly
                    let mut wlock_instance = instance.write().map_err(|e| {
                        SystemError::ComponentAccess(format!("Failed to write Instance: {}", e))
                    })?;
                    let rlock_pos3 = pos3.read().map_err(|e| {
                        SystemError::ComponentAccess(format!("Failed to read Pos3: {}", e))
                    })?;

                    wlock_instance.position = rlock_pos3.pos;
                    wlock_instance.rotation = rlock_pos3.rot;
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
    })
}

/// Update the physics system
pub(super) fn update_physics(
    world: Arc<World>,
    state: Arc<RwLock<State>>,
    dt: time::Duration,
) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>> {
    Box::pin(async move {
        let dt_secs = dt.as_secs_f32();
        let mut physics_bodies = Vec::new();

        // Get all entities with RigidBody component
        let state = state.read().unwrap();
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
                wlock_physics_body.update_pos(&mut wlock_pos3, dt_secs);
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
                    state.queue.write_buffer(
                        &buffer_guard.0,
                        0,
                        bytemuck::cast_slice(&[instance_raw]),
                    );
                }
                Ok(())
            })
            .collect();

        // Check for any errors
        for result in results {
            result?;
        }

        Ok(())
    })
}
