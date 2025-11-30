use super::{SystemError, SystemResult};
use core::time;
use gears_ecs::World;
use gears_ecs::components::physics::AABBCollisionBox;
use gears_ecs::components::transforms::Pos3;
use gears_ecs::components::{self, lights::Light};
use gears_ecs::query::{ComponentQuery, WorldQueryExt};
use gears_renderer::light::LightUniform;
use gears_renderer::state::State;
use gears_renderer::{BufferComponent, animation, instance, light, model};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

/// Updates all of the lights stored in the ECS instance.
///
/// # Arguments
///
/// * `world` - The ECS world instance.
/// * `state` - The renderer state instance.
/// * `dt` - The duration of the last frame.
///
/// # Returns
///
/// A future that resolves to a [`SystemResult`] indicating the success or failure of the system.
pub(super) fn update_lights(
    world: Arc<World>,
    state: Arc<RwLock<State>>,
    _dt: time::Duration,
) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>> {
    Box::pin(async move {
        let state = state.read().unwrap();
        let light_entities = world.get_entities_with_component::<Light>();

        if light_entities.is_empty() {
            return Ok(());
        }

        // Build query for all light entities
        let query = ComponentQuery::new()
            .read::<Pos3>(light_entities.clone())
            .read::<Light>(light_entities.clone());

        // Acquire resources (blocking)
        let resources = world.acquire_query(query).ok_or_else(|| {
            SystemError::ComponentAccess("Failed to acquire light components".to_string())
        })?;

        let mut light_uniforms = Vec::new();

        for &entity in &light_entities {
            if let (Some(pos3_component), Some(light_component)) = (
                resources.get::<Pos3>(entity),
                resources.get::<Light>(entity),
            ) {
                let pos3 = pos3_component.read().map_err(|e| {
                    SystemError::ComponentAccess(format!("Failed to read Pos3: {}", e))
                })?;
                let light = light_component.read().map_err(|e| {
                    SystemError::ComponentAccess(format!("Failed to read Light: {}", e))
                })?;

                let light_uniform = LightUniform::from_components(&light, &pos3);
                light_uniforms.push(light_uniform);
            } else {
                return Err(SystemError::MissingComponent(format!(
                    "Required components missing for light entity {:?}",
                    entity
                )));
            }
        }

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

        state.queue().write_buffer(
            state.base_pipeline().light_buffer(),
            0,
            bytemuck::cast_slice(&[light_data]),
        );

        Ok(())
    })
}

/// Updates all of the models stored in the ECS instance.
///
/// # Arguments
///
/// * `world` - The ECS world instance.
/// * `state` - The renderer state instance.
/// * `dt` - The duration of the last frame.
///
/// # Returns
///
/// A future that resolves to a [`SystemResult`] indicating the success or failure of the system.
pub(super) fn update_models(
    world: Arc<World>,
    state: Arc<RwLock<State>>,
    _dt: time::Duration,
) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>> {
    Box::pin(async move {
        let state = state.write().unwrap();
        let model_entities =
            world.get_entities_with_component::<components::misc::StaticModelMarker>();

        if model_entities.is_empty() {
            return Ok(());
        }

        // Process each entity individually to avoid lock conflicts
        _ = model_entities
            .par_iter()
            .map(|&entity| {
                // Build query for this specific model entity
                let query = ComponentQuery::new()
                    .read::<components::misc::Name>(vec![entity])
                    .read::<components::transforms::Pos3>(vec![entity])
                    .write::<instance::Instance>(vec![entity])
                    .read::<BufferComponent>(vec![entity])
                    .read::<model::Model>(vec![entity])
                    .write::<components::misc::AnimationQueue>(vec![entity]);

                if let Some(resources) = world.acquire_query(query) {
                    let _name =
                        resources
                            .get::<components::misc::Name>(entity)
                            .ok_or_else(|| {
                                SystemError::MissingComponent(format!(
                                    "Name component missing for entity {:?}",
                                    entity
                                ))
                            })?;

                    let pos3 = resources
                        .get::<components::transforms::Pos3>(entity)
                        .ok_or_else(|| {
                            SystemError::MissingComponent(format!(
                                "Pos3 component missing for entity {:?}",
                                entity
                            ))
                        })?;

                    let instance =
                        resources.get::<instance::Instance>(entity).ok_or_else(|| {
                            SystemError::MissingComponent(format!(
                                "Instance component missing for entity {:?}",
                                entity
                            ))
                        })?;

                    let buffer = resources.get::<BufferComponent>(entity).ok_or_else(|| {
                        SystemError::MissingComponent(format!(
                            "BufferComponent missing for entity {:?}",
                            entity
                        ))
                    })?;

                    let model = resources.get::<model::Model>(entity).ok_or_else(|| {
                        SystemError::MissingComponent(format!(
                            "Model component missing for entity {:?}",
                            entity
                        ))
                    })?;

                    let animation_queue = resources.get::<components::misc::AnimationQueue>(entity);

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
                            && let Some(next_animation) = wlock_animation_queue.play_next() {
                                log::info!("Starting animation: {}", next_animation);
                            }

                        // Process current animation if one is playing
                        if let Some(current_anim_name) = wlock_animation_queue.current_animation() {
                            let rlock_model = model.read().map_err(|e| {
                                SystemError::ComponentAccess(format!("Failed to read Model: {}", e))
                            })?;

                            // Find the animation in the model
                            if let Ok(animation_clip) = rlock_model.get_animation(current_anim_name)
                            {
                                // Get current animation time
                                let current_time =
                                    wlock_animation_queue.time.elapsed().as_secs_f32();

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
                                        if let Some(next_animation) =
                                            wlock_animation_queue.play_next()
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
                                    SystemError::ComponentAccess(format!(
                                        "Failed to read Pos3: {}",
                                        e
                                    ))
                                })?;

                                wlock_instance.position = rlock_pos3.pos;
                                wlock_instance.rotation = rlock_pos3.rot;
                            }
                        } else {
                            // No animation playing, use position component
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

                    // Update the buffer with instance data
                    let instance_raw = instance
                        .read()
                        .map_err(|e| {
                            SystemError::ComponentAccess(format!("Failed to read Instance: {}", e))
                        })?
                        .to_raw();

                    let buffer_guard = buffer.write().map_err(|e| {
                        SystemError::ComponentAccess(format!("Failed to write Buffer: {}", e))
                    })?;

                    state.queue().write_buffer(
                        &buffer_guard.0,
                        0,
                        bytemuck::cast_slice(&[instance_raw]),
                    );
                } else {
                    // Could not acquire resources, skip this entity this frame
                    log::debug!("Skipping model entity {:?} - resources locked", entity);
                }

                Ok(())
            })
            .collect::<Result<Vec<()>, SystemError>>()?;

        Ok(())
    })
}

/// Updates the physics objects stored in the ECS instance.
///
/// # Arguments
///
/// * `world` - The ECS world instance.
/// * `state` - The renderer state instance.
/// * `dt` - The duration of the last frame.
///
/// # Returns
///
/// A future that resolves to a [`SystemResult`] indicating the success or failure of the system.
pub(super) fn update_physics(
    world: Arc<World>,
    state: Arc<RwLock<State>>,
    dt: time::Duration,
) -> Pin<Box<dyn Future<Output = SystemResult<()>> + Send>> {
    Box::pin(async move {
        let dt_secs = dt.as_secs_f32();
        let state = state.read().unwrap();
        let physics_entities =
            world.get_entities_with_component::<components::physics::RigidBody<AABBCollisionBox>>();

        if physics_entities.is_empty() {
            return Ok(());
        }

        // First pass: Update positions based on velocities
        _ = physics_entities
            .par_iter()
            .map(|&entity| {
                let query = ComponentQuery::new()
                    .write::<components::physics::RigidBody<AABBCollisionBox>>(vec![entity])
                    .write::<components::transforms::Pos3>(vec![entity]);

                if let Some(resources) = world.acquire_query(query)
                    && let (Some(physics_body), Some(pos3)) = (
                        resources.get::<components::physics::RigidBody<AABBCollisionBox>>(entity),
                        resources.get::<components::transforms::Pos3>(entity),
                    ) {
                        let mut wlock_physics_body = physics_body.write().map_err(|e| {
                            SystemError::ComponentAccess(format!(
                                "Failed to write RigidBody: {}",
                                e
                            ))
                        })?;
                        let mut wlock_pos3 = pos3.write().map_err(|e| {
                            SystemError::ComponentAccess(format!("Failed to write Pos3: {}", e))
                        })?;

                        wlock_physics_body.update_pos(&mut wlock_pos3, dt_secs);
                    }

                Ok(())
            })
            .collect::<Result<Vec<()>, SystemError>>()?;

        // Second pass: Check for collisions and resolve them
        for i in 0..physics_entities.len() {
            for j in (i + 1)..physics_entities.len() {
                let entity_a = physics_entities[i];
                let entity_b = physics_entities[j];

                // Build query for both entities involved in potential collision
                let query = ComponentQuery::new()
                    .write::<components::physics::RigidBody<AABBCollisionBox>>(vec![
                        entity_a, entity_b,
                    ])
                    .write::<components::transforms::Pos3>(vec![entity_a, entity_b])
                    .read::<components::transforms::Scale>(vec![entity_a, entity_b]);

                if let Some(resources) = world.acquire_query(query)
                    && let (
                        Some(physics_body_a),
                        Some(pos3_a),
                        Some(physics_body_b),
                        Some(pos3_b),
                    ) = (
                        resources.get::<components::physics::RigidBody<AABBCollisionBox>>(entity_a),
                        resources.get::<components::transforms::Pos3>(entity_a),
                        resources.get::<components::physics::RigidBody<AABBCollisionBox>>(entity_b),
                        resources.get::<components::transforms::Pos3>(entity_b),
                    ) {
                        // Get optional scale components
                        let scale_a = resources.get::<components::transforms::Scale>(entity_a);
                        let scale_b = resources.get::<components::transforms::Scale>(entity_b);
                        let mut wlock_physics_body_a = physics_body_a.write().map_err(|e| {
                            SystemError::ComponentAccess(format!(
                                "Failed to write RigidBody A: {}",
                                e
                            ))
                        })?;
                        let mut wlock_physics_body_b = physics_body_b.write().map_err(|e| {
                            SystemError::ComponentAccess(format!(
                                "Failed to write RigidBody B: {}",
                                e
                            ))
                        })?;
                        let mut wlock_pos3_a = pos3_a.write().map_err(|e| {
                            SystemError::ComponentAccess(format!("Failed to write Pos3 A: {}", e))
                        })?;
                        let mut wlock_pos3_b = pos3_b.write().map_err(|e| {
                            SystemError::ComponentAccess(format!("Failed to write Pos3 B: {}", e))
                        })?;

                        // Convert Scale to Vector3 if present
                        let scale_vec_a = scale_a.and_then(|s| {
                            s.read().ok().map(|scale_guard| match *scale_guard {
                                components::transforms::Scale::Uniform(s) => {
                                    cgmath::Vector3::new(s, s, s)
                                }
                                components::transforms::Scale::NonUniform { x, y, z } => {
                                    cgmath::Vector3::new(x, y, z)
                                }
                            })
                        });
                        let scale_vec_b = scale_b.and_then(|s| {
                            s.read().ok().map(|scale_guard| match *scale_guard {
                                components::transforms::Scale::Uniform(s) => {
                                    cgmath::Vector3::new(s, s, s)
                                }
                                components::transforms::Scale::NonUniform { x, y, z } => {
                                    cgmath::Vector3::new(x, y, z)
                                }
                            })
                        });

                        components::physics::RigidBody::check_and_resolve_collision(
                            &mut wlock_physics_body_a,
                            &mut wlock_pos3_a,
                            scale_vec_a.as_ref(),
                            &mut wlock_physics_body_b,
                            &mut wlock_pos3_b,
                            scale_vec_b.as_ref(),
                        );
                    }
            }
        }

        // Third pass: Update instance data based on new positions
        _ = physics_entities
            .par_iter()
            .map(|&entity| {
                let query = ComponentQuery::new()
                    .write::<instance::Instance>(vec![entity])
                    .read::<BufferComponent>(vec![entity])
                    .read::<Pos3>(vec![entity]);

                if let Some(resources) = world.acquire_query(query)
                    && let (Some(instance), Some(buffer), Some(pos3)) = (
                        resources.get::<instance::Instance>(entity),
                        resources.get::<BufferComponent>(entity),
                        resources.get::<Pos3>(entity),
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
                        state.queue().write_buffer(
                            &buffer_guard.0,
                            0,
                            bytemuck::cast_slice(&[instance_raw]),
                        );
                    }

                Ok(())
            })
            .collect::<Result<Vec<()>, SystemError>>()?;

        Ok(())
    })
}
