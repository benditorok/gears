use super::{
    controllers::{MovementController, ViewController},
    models::ModelSource,
    physics::RigidBody,
    transforms::Pos3,
};

pub struct Player {
    pub pos3: Option<Pos3>,
    pub model_source: Option<ModelSource>,
    pub movement_controller: Option<MovementController>,
    pub view_controller: Option<ViewController>,
    pub rigidbody: Option<RigidBody>,
}

impl Default for Player {
    fn default() -> Self {
        let rigid_body = RigidBody {
            mass: 80.0, // Average human mass in kg
            velocity: cgmath::Vector3::new(0.0, 0.0, 0.0),
            acceleration: cgmath::Vector3::new(0.0, 0.0, 0.0),
            collision_box: super::physics::CollisionBox {
                min: cgmath::Vector3::new(-0.5, -2.0, -0.5),
                max: cgmath::Vector3::new(0.5, 2.0, 0.5),
            },
            is_static: false,
        };

        let view_controller = ViewController::new(0.2, 1.8);

        Self {
            pos3: Some(Pos3::default()),
            model_source: Some(ModelSource::Obj("res/models/sphere/sphere.obj")),
            movement_controller: Some(MovementController::default()),
            view_controller: Some(view_controller),
            rigidbody: Some(rigid_body),
        }
    }
}

// impl Prefab for Player {
//     fn unpack_prefab(&mut self) -> Vec<Box<impl crate::prelude::Component>> {
//         vec![
//             Box::new(PlayerMarker),
//             Box::new(self.pos3.take().unwrap()),
//             Box::new(self.model_source.take().unwrap()),
//             Box::new(self.movement_controller.take().unwrap()),
//             Box::new(self.view_controller.take().unwrap()),
//         ]
//     }
// }
