use crate::ecs::traits::Component;
use gears_macro::Component;

/// A drawable model component. Does not have any physics properties nor collision.
#[derive(Component, Debug, Clone)]
pub struct StaticModel {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

/// A component that stores the source of a model.
#[derive(Component, Debug, Copy, Clone)]
pub enum ModelSource {
    Obj(&'static str),
    Gltf(&'static str),
}
