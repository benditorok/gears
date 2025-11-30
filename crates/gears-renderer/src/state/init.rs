use super::State;
use super::instance;
use super::model;
use crate::BufferComponent;
use crate::resources::{load_model_gltf, load_model_obj};
use cgmath::prelude::*;
use gears_ecs::components::misc::Marker;
use gears_ecs::components::physics::AABBCollisionBox;
use gears_ecs::{
    World,
    components::{self},
};
use log::info;
use std::sync::Arc;
use wgpu::util::DeviceExt;

/// Initializes the player component and camera controllers.
///
/// # Arguments
///
/// * `state` - The mutable reference to the current state.
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
        state.set_view_controller(Some(Arc::clone(&view_controller)));

        let movement_controller = state
            .world
            .get_component::<components::controllers::MovementController>(player_entity)
            .unwrap_or_else(|| panic!("{}", components::misc::PlayerMarker::describe()));
        state.set_movement_controller(Some(Arc::clone(&movement_controller)));

        return true;
    }

    false
}

/// Initializes the static camera component if no player is found.
///
/// # Arguments
///
/// * `state` - The mutable reference to the current state.
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
        state.set_view_controller(Some(Arc::clone(&view_controller)));

        let movement_controller = state
            .world
            .get_component::<components::controllers::MovementController>(static_camera_entity);
        if let Some(movement_controller) = movement_controller {
            state.set_movement_controller(Some(Arc::clone(&movement_controller)));
        }
        return;
    }

    panic!("No camera found in the ECS!");
}

/// Initializes static model components from the ECS world.
///
/// # Arguments
///
/// * `device` - The wgpu device for GPU resource creation.
/// * `queue` - The wgpu queue for buffer uploads.
/// * `texture_bind_group_layout` - The bind group layout for textures.
/// * `world` - The ECS world containing the entities and components.
pub(super) async fn models(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    world: &Arc<World>,
) {
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
            let model_source_copy = {
                let rlock_model_source = model_source.read().unwrap();
                info!("Loading model: {:?}", rlock_model_source);
                *rlock_model_source
            };

            match model_source_copy {
                components::models::ModelSource::Obj(path) => {
                    load_model_obj(path, device, queue, texture_bind_group_layout)
                        .await
                        .unwrap()
                }
                components::models::ModelSource::Gltf(path) => {
                    load_model_gltf(path, device, queue, texture_bind_group_layout)
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
                scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
            }
        };

        if let Some(flip) = flip {
            let rlock_flip = flip.read().unwrap();

            let flip_rotation = match *rlock_flip {
                components::transforms::Flip::Horizontal => {
                    cgmath::Quaternion::from_angle_y(cgmath::Rad(std::f32::consts::PI))
                }
                components::transforms::Flip::Vertical => {
                    cgmath::Quaternion::from_angle_x(cgmath::Rad(std::f32::consts::PI))
                }
                components::transforms::Flip::Both => {
                    cgmath::Quaternion::from_angle_y(cgmath::Rad(std::f32::consts::PI))
                        * cgmath::Quaternion::from_angle_x(cgmath::Rad(std::f32::consts::PI))
                }
            };
            instance.rotation = instance.rotation * flip_rotation;
        }

        if let Some(scale) = scale {
            let rlock_scale = scale.read().unwrap();

            instance.scale = match *rlock_scale {
                components::transforms::Scale::Uniform(s) => cgmath::Vector3::new(s, s, s),
                components::transforms::Scale::NonUniform { x, y, z } => {
                    cgmath::Vector3::new(x, y, z)
                }
            };
        }

        let instance_raw = instance.to_raw();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("{} Instance Buffer", name.read().unwrap().0).as_str()),
            contents: bytemuck::cast_slice(&[instance_raw]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        world.add_component(*entity, instance);
        world.add_component(*entity, BufferComponent(instance_buffer));
    }
}

/// Initializes physics-enabled model components with collision boxes.
///
/// # Arguments
///
/// * `device` - The wgpu device for GPU resource creation.
/// * `queue` - The wgpu queue for buffer uploads.
/// * `texture_bind_group_layout` - The bind group layout for textures.
/// * `world` - The ECS world containing the entities and components.
pub(super) async fn physics_models(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    world: &Arc<World>,
) {
    let physics_entities = world.get_entities_with_component::<components::misc::RigidBodyMarker>();

    for entity in physics_entities.iter() {
        let name = world
            .get_component::<components::misc::Name>(*entity)
            .expect("No name provided for the Model!");

        let physics_body = world
            .get_component::<components::physics::RigidBody<AABBCollisionBox>>(*entity)
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
            let model_source_copy = {
                let rlock_model_source = model_source.read().unwrap();
                *rlock_model_source
            };

            match model_source_copy {
                components::models::ModelSource::Obj(path) => {
                    load_model_obj(path, device, queue, texture_bind_group_layout)
                        .await
                        .unwrap()
                }
                components::models::ModelSource::Gltf(path) => {
                    load_model_gltf(path, device, queue, texture_bind_group_layout)
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
                scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
            }
        };

        if let Some(flip) = flip {
            let rlock_flip = flip.read().unwrap();

            let flip_rotation = match *rlock_flip {
                components::transforms::Flip::Horizontal => {
                    cgmath::Quaternion::from_angle_y(cgmath::Rad(std::f32::consts::PI))
                }
                components::transforms::Flip::Vertical => {
                    cgmath::Quaternion::from_angle_x(cgmath::Rad(std::f32::consts::PI))
                }
                components::transforms::Flip::Both => {
                    cgmath::Quaternion::from_angle_y(cgmath::Rad(std::f32::consts::PI))
                        * cgmath::Quaternion::from_angle_x(cgmath::Rad(std::f32::consts::PI))
                }
            };
            instance.rotation = instance.rotation * flip_rotation;
        }

        if let Some(scale) = scale {
            let rlock_scale = scale.read().unwrap();

            instance.scale = match *rlock_scale {
                components::transforms::Scale::Uniform(s) => cgmath::Vector3::new(s, s, s),
                components::transforms::Scale::NonUniform { x, y, z } => {
                    cgmath::Vector3::new(x, y, z)
                }
            };
        }

        let instance_raw = instance.to_raw();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("{} Instance Buffer", name.read().unwrap().0).as_str()),
            contents: bytemuck::cast_slice(&[instance_raw]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        world.add_component(*entity, instance);
        world.add_component(*entity, BufferComponent(instance_buffer));

        // Create a wireframe collider from the RigidBody's data
        let wireframe = model::WireframeMesh::new(device, &physics_body.read().unwrap());
        world.add_component(*entity, wireframe);
    }
}

/// Initializes target entities for gameplay systems.
///
/// # Arguments
///
/// * `state` - The mutable reference to the current state.
pub(super) fn targets(state: &mut State) {
    let target_entities = state
        .world
        .get_entities_with_component::<components::misc::TargetMarker>();

    state.target_entities = Some(target_entities);
}
