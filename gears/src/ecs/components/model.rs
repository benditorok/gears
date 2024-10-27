use crate::ecs::traits::Component;
use cgmath::{InnerSpace, Rotation3};

/// A drawable model component. Does not have any physics properties nor collision.
#[derive(Debug, Copy, Clone)]
pub struct StaticModel {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

impl Component for StaticModel {}

/// A component that stores the source of a model.
#[derive(Debug, Copy, Clone)]
pub struct ModelSource {
    pub obj_path: &'static str,
}

impl Component for ModelSource {}
