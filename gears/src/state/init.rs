use super::State;
use crate::ecs::components::Marker;
use crate::ecs::{self, components, World};
use crate::renderer::model;
use crate::renderer::{instance, light, resources};
use cgmath::prelude::*;
use std::sync::{Arc, Mutex};
use wgpu::hal::vulkan::Workarounds;
use wgpu::util::DeviceExt;

/// Initialie the player component.
///
/// # Returns
///
/// A boolean indicating whether the player was found.
pub(super) fn player(state: &mut State) -> bool {
    // * Look for a player first and retrieve it's camera
    let mut player_entity = state
        .world
        .get_entities_with_component::<components::misc::PlayerMarker>();

    if !player_entity.is_empty() {
        let player_entity = player_entity.pop().unwrap();
        state.player_entity = Some(player_entity);
        state.camera_owner_entity = Some(player_entity);
        //self.camera_type = ecs::components::misc::CameraType::Player;

        let view_controller = state
            .world
            .get_component::<components::controllers::ViewController>(player_entity)
            .unwrap_or_else(|| panic!("{}", components::misc::PlayerMarker::describe()));
        state.view_controller = Some(Arc::clone(&view_controller));

        let movement_controller = state
            .world
            .get_component::<components::controllers::MovementController>(player_entity)
            .unwrap_or_else(|| panic!("{}", components::misc::PlayerMarker::describe()));
        state.movement_controller = Some(Arc::clone(&movement_controller));

        return true;
    }

    false
}

/// Initialize the camera component.
pub(super) fn camera(state: &mut State) {
    let mut static_camera_entity = state
        .world
        .get_entities_with_component::<components::misc::CameraMarker>();

    if !static_camera_entity.is_empty() {
        let static_camera_entity = static_camera_entity.pop().unwrap();
        state.camera_owner_entity = Some(static_camera_entity);
        //self.camera_type = ecs::components::misc::CameraType::Static;

        let view_controller = state
            .world
            .get_component::<components::controllers::ViewController>(static_camera_entity)
            .unwrap_or_else(|| panic!("{}", components::misc::CameraMarker::describe()));
        state.view_controller = Some(Arc::clone(&view_controller));

        let movement_controller = state
            .world
            .get_component::<components::controllers::MovementController>(static_camera_entity);
        if let Some(movement_controller) = movement_controller {
            state.movement_controller = Some(Arc::clone(&movement_controller));
        }
        return;
    }

    panic!("No camera found in the ECS!");
}

/// Initialize the light components.
pub(super) fn lights(state: &mut State) {
    let light_entities = state
        .world
        .get_entities_with_component::<components::misc::LightMarker>();

    for entity in light_entities.iter() {
        let pos = state
            .world
            .get_component::<components::transforms::Pos3>(*entity)
            .unwrap_or_else(|| panic!("{}", components::misc::LightMarker::describe()));

        let light = state
            .world
            .get_component::<components::lights::Light>(*entity)
            .unwrap_or_else(|| panic!("{}", components::misc::LightMarker::describe()));

        let light_uniform = {
            let rlock_pos = pos.read().unwrap();
            let rlock_light = light.read().unwrap();

            match *rlock_light {
                components::lights::Light::Point { radius, intensity } => light::LightUniform {
                    position: [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z],
                    light_type: light::LightType::Point as u32,
                    color: [1.0, 1.0, 1.0],
                    radius,
                    direction: [0.0; 3],
                    intensity,
                },
                components::lights::Light::PointColoured {
                    radius,
                    color,
                    intensity,
                } => light::LightUniform {
                    position: [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z],
                    light_type: light::LightType::Point as u32,
                    color,
                    radius,
                    direction: [0.0; 3],
                    intensity,
                },
                components::lights::Light::Ambient { intensity } => light::LightUniform {
                    position: [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z],
                    light_type: light::LightType::Ambient as u32,
                    color: [1.0, 1.0, 1.0],
                    radius: 0.0,
                    direction: [0.0; 3],
                    intensity,
                },
                components::lights::Light::AmbientColoured { color, intensity } => {
                    light::LightUniform {
                        position: [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z],
                        light_type: light::LightType::Ambient as u32,
                        color,
                        radius: 0.0,
                        direction: [0.0; 3],
                        intensity,
                    }
                }
                components::lights::Light::Directional {
                    direction,
                    intensity,
                } => light::LightUniform {
                    position: [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z],
                    light_type: light::LightType::Directional as u32,
                    color: [1.0, 1.0, 1.0],
                    radius: 0.0,
                    direction,
                    intensity,
                },
                components::lights::Light::DirectionalColoured {
                    direction,
                    color,
                    intensity,
                } => light::LightUniform {
                    position: [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z],
                    light_type: light::LightType::Directional as u32,
                    color,
                    radius: 0.0,
                    direction,
                    intensity,
                },
            }
        };
        state.world.add_component(*entity, light_uniform);
    }

    if light_entities.len() > light::NUM_MAX_LIGHTS as usize {
        panic!(
            "The number of lights exceeds the maximum number of lights supported by the renderer!"
        );
    }

    state.light_entities = Some(light_entities);
}

/// Initialize the model components.
///
/// # Returns
///
/// A future which can be awaited.
pub(super) async fn models<'a>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    world: &Arc<ecs::World>,
) -> Vec<ecs::Entity> {
    let model_entities = world.get_entities_with_component::<components::misc::StaticModelMarker>();

    for entity in model_entities.iter() {
        let name = world
            .get_component::<components::misc::Name>(*entity)
            .unwrap_or_else(|| panic!("{}", components::misc::StaticModelMarker::describe()));
        let pos3 = world
            .get_component::<components::transforms::Pos3>(*entity)
            .unwrap_or_else(|| panic!("{}", components::misc::StaticModelMarker::describe()));
        let model_source = world
            .get_component::<components::models::ModelSource>(*entity)
            .unwrap_or_else(|| panic!("{}", components::misc::StaticModelMarker::describe()));

        let flip = world.get_component::<components::transforms::Flip>(*entity);

        let scale = world.get_component::<components::transforms::Scale>(*entity);

        let obj_model = {
            let rlock_model_source = model_source.read().unwrap();

            match *rlock_model_source {
                components::models::ModelSource::Obj(path) => {
                    resources::load_model_obj(path, device, queue, texture_bind_group_layout)
                        .await
                        .unwrap()
                }
                components::models::ModelSource::Gltf(path) => {
                    resources::load_model_gltf(path, device, queue, texture_bind_group_layout)
                        .await
                        .unwrap()
                }
            }
        };
        world.add_component(*entity, obj_model);

        // TODO rename instance to model::ModelUniform
        let mut instance = {
            let rlock_pos3 = pos3.read().unwrap();
            instance::Instance {
                position: rlock_pos3.pos,
                rotation: rlock_pos3.rot,
            }
        };

        if let Some(flip) = flip {
            let rlock_flip = flip.read().unwrap();

            match *rlock_flip {
                components::transforms::Flip::Horizontal => {
                    instance.rotation =
                        cgmath::Quaternion::from_angle_y(cgmath::Rad(std::f32::consts::PI));
                }
                components::transforms::Flip::Vertical => {
                    instance.rotation =
                        cgmath::Quaternion::from_angle_x(cgmath::Rad(std::f32::consts::PI));
                }
                components::transforms::Flip::Both => {
                    instance.rotation =
                        cgmath::Quaternion::from_angle_y(cgmath::Rad(std::f32::consts::PI));
                    instance.rotation =
                        cgmath::Quaternion::from_angle_x(cgmath::Rad(std::f32::consts::PI));
                }
            }
        }

        // TODO scale should update the rot (quaternion)??
        // if let Some(scale) = scale {
        //     let rlock_scale = scale.read().unwrap();

        //     match *rlock_scale {
        //         Scale::Uniform(s) => {
        //             instance.scale = cgmath::Vector3::new(s, s, s);
        //         }
        //         Scale::NonUniform { x, y, z } => {
        //             instance.scale = cgmath::Vector3::new(x, y, z);
        //         }
        //     }
        // }

        let instance_raw = instance.to_raw();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("{} Instance Buffer", name.read().unwrap().0).as_str()),
            contents: bytemuck::cast_slice(&[instance_raw]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        world.add_component(*entity, instance);
        world.add_component(*entity, instance_buffer);
    }

    model_entities
}

pub(super) async fn physics_models<'a>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    world: &Arc<ecs::World>,
) -> Vec<ecs::Entity> {
    let physics_entities = world.get_entities_with_component::<components::misc::RigidBodyMarker>();

    for entity in physics_entities.iter() {
        let name = world
            .get_component::<components::misc::Name>(*entity)
            .expect("No name provided for the Model!");

        let physics_body = world
            .get_component::<components::physics::RigidBody>(*entity)
            .unwrap_or_else(|| panic!("{}", components::misc::RigidBodyMarker::describe()));
        let model_source = world
            .get_component::<components::models::ModelSource>(*entity)
            .unwrap_or_else(|| panic!("{}", components::misc::RigidBodyMarker::describe()));
        let pos3 = world
            .get_component::<components::transforms::Pos3>(*entity)
            .unwrap_or_else(|| panic!("{}", components::misc::RigidBodyMarker::describe()));

        let flip = world.get_component::<components::transforms::Flip>(*entity);

        let scale = world.get_component::<components::transforms::Scale>(*entity);

        let obj_model = {
            let rlock_model_source = model_source.read().unwrap();

            match *rlock_model_source {
                components::models::ModelSource::Obj(path) => {
                    resources::load_model_obj(path, device, queue, texture_bind_group_layout)
                        .await
                        .unwrap()
                }
                components::models::ModelSource::Gltf(path) => {
                    resources::load_model_gltf(path, device, queue, texture_bind_group_layout)
                        .await
                        .unwrap()
                }
            }
        };
        world.add_component(*entity, obj_model);

        // TODO rename instance to model::ModelUniform
        let mut instance = {
            let rlock_physics_body = physics_body.read().unwrap();
            let rlock_pos3 = pos3.read().unwrap();
            instance::Instance {
                position: rlock_pos3.pos,
                rotation: rlock_pos3.rot,
            }
        };

        if let Some(flip) = flip {
            let rlock_flip = flip.read().unwrap();

            match *rlock_flip {
                components::transforms::Flip::Horizontal => {
                    instance.rotation =
                        cgmath::Quaternion::from_angle_y(cgmath::Rad(std::f32::consts::PI));
                }
                components::transforms::Flip::Vertical => {
                    instance.rotation =
                        cgmath::Quaternion::from_angle_x(cgmath::Rad(std::f32::consts::PI));
                }
                components::transforms::Flip::Both => {
                    instance.rotation =
                        cgmath::Quaternion::from_angle_y(cgmath::Rad(std::f32::consts::PI));
                    instance.rotation =
                        cgmath::Quaternion::from_angle_x(cgmath::Rad(std::f32::consts::PI));
                }
            }
        }

        // TODO scale should update the rot (quaternion)??
        // if let Some(scale) = scale {
        //     let rlock_scale = scale.read().unwrap();

        //     match *rlock_scale {
        //         Scale::Uniform(s) => {
        //             instance.scale = cgmath::Vector3::new(s, s, s);
        //         }
        //         Scale::NonUniform { x, y, z } => {
        //             instance.scale = cgmath::Vector3::new(x, y, z);
        //         }
        //     }
        // }

        let instance_raw = instance.to_raw();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("{} Instance Buffer", name.read().unwrap().0).as_str()),
            contents: bytemuck::cast_slice(&[instance_raw]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        world.add_component(*entity, instance);
        world.add_component(*entity, instance_buffer);

        // Create a wireframe collider from the RigidBody's data
        let wireframe = model::WireframeMesh::new(device, &physics_body.read().unwrap());
        world.add_component(*entity, wireframe);
    }

    physics_entities
}

pub(super) fn targets(state: &mut State) {
    let target_entities = state
        .world
        .get_entities_with_component::<components::misc::TargetMarker>();

    state.target_entities = Some(target_entities);
}
