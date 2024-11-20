use crate::ecs::{components::misc::PlayerMarker, traits::Prefab};

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
        Self {
            pos3: Some(Pos3::default()),
            model_source: Some(ModelSource::Obj("res/models/sphere/sphere.obj")),
            movement_controller: Some(MovementController::default()),
            view_controller: Some(ViewController::default()),
            rigidbody: Some(RigidBody::default()),
        }
    }
}

impl Prefab for Player {
    fn unpack_prefab(&mut self) -> Vec<Box<dyn crate::prelude::Component>> {
        vec![
            Box::new(PlayerMarker),
            Box::new(self.pos3.take().unwrap()),
            Box::new(self.model_source.take().unwrap()),
            Box::new(self.movement_controller.take().unwrap()),
            Box::new(self.view_controller.take().unwrap()),
        ]
    }
}
