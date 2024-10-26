pub mod light;
pub mod physics;
pub mod transform;

use super::traits::Component;
use cgmath::{InnerSpace, Rotation3};

/// A component that stores the camera type.
#[derive(Debug, Copy, Clone)]
pub enum Camera {
    FPS {
        look_at: cgmath::Point3<f32>,
        speed: f32,
        sensitivity: f32,
    },
    Fixed {
        look_at: cgmath::Point3<f32>,
    },
}

impl Component for Camera {}

// TODO remove this
/// A component that stores the model type.
#[derive(Debug, Copy, Clone)]
pub enum Model<'a> {
    Dynamic { obj_path: &'a str },
    Static { obj_path: &'a str },
}

impl Component for Model<'static> {}

/// A component that stores the name of an object.
pub struct Name(pub &'static str);

impl Component for Name {}
