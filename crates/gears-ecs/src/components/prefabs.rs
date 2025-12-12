use super::{
    controllers::{MovementController, ViewController},
    models::ModelSource,
    physics::{AABBCollisionBox, RigidBody},
    transforms::Pos3,
};
use crate::{Entity, EntityBuilder, components::misc::PlayerMarker};

/// Trait for creating entities from prefabs.
pub trait Prefab: Sized {
    /// Creates an entity from the prefab and adds it to the world.
    ///
    /// # Arguments
    ///
    /// * `builder` - The entity builder to use for creating the entity.
    /// * `prefab` - The prefab instance to create the entity from.
    ///
    /// # Returns
    ///
    /// The created entity.
    fn from_prefab(builder: &mut impl EntityBuilder, prefab: Self) -> Entity;
}

/// Prefab for creating a player entity with common components.
pub struct PlayerPrefab {
    /// Position component.
    pub pos3: Option<Pos3>,
    /// Model source component.
    pub model_source: Option<ModelSource>,
    /// Movement controller component.
    pub movement_controller: Option<MovementController>,
    /// View controller component.
    pub view_controller: Option<ViewController>,
    /// Rigid body component.
    pub rigidbody: Option<RigidBody<AABBCollisionBox>>,
}

impl Default for PlayerPrefab {
    /// Creates a player prefab with common components initialized.
    ///
    /// # Returns
    ///
    /// The default [`PlayerPrefab`] instance.
    fn default() -> Self {
        let rigid_body = RigidBody {
            mass: 80.0, // Average human mass in kg
            velocity: cgmath::Vector3::new(0.0, 0.0, 0.0),
            acceleration: cgmath::Vector3::new(0.0, -10.0, 0.0),
            collision_box: super::physics::AABBCollisionBox {
                min: cgmath::Vector3::new(-0.5, -2.0, -0.5),
                max: cgmath::Vector3::new(0.5, 2.0, 0.5),
            },
            is_static: false,
            restitution: 0.0,
        };

        let view_controller = ViewController::new(0.8, 1.8);

        let pos3 = Pos3::new(cgmath::Vector3::new(0.0, 1.0, 0.0));

        Self {
            pos3: Some(pos3),
            model_source: Some(ModelSource::Obj("res/models/sphere/sphere.obj")),
            movement_controller: Some(MovementController::default()),
            view_controller: Some(view_controller),
            rigidbody: Some(rigid_body),
        }
    }
}

impl Prefab for PlayerPrefab {
    /// Creates a player entity from the prefab and adds it to the world.
    ///
    /// # Arguments
    ///
    /// * `builder` - The entity builder to use for creating the entity.
    /// * `prefab` - The player prefab instance to create the entity from.
    ///
    /// # Returns
    ///
    /// The created player entity.
    fn from_prefab(builder: &mut impl EntityBuilder, mut prefab: Self) -> Entity {
        builder
            .new_entity()
            .add_component(PlayerMarker)
            .add_component(prefab.pos3.take().unwrap())
            .add_component(prefab.model_source.take().unwrap())
            .add_component(prefab.movement_controller.take().unwrap())
            .add_component(prefab.view_controller.take().unwrap())
            .add_component(prefab.rigidbody.take().unwrap())
            .build()
    }
}
