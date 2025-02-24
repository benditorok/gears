use crate::Component;
use cgmath::One;
use gears_macro::Component;

pub trait Pos {
    fn get_pos(&self) -> cgmath::Vector3<f32>;
}

/// A component that stores the position of any object.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Pos3 {
    pub pos: cgmath::Vector3<f32>,
    pub rot: cgmath::Quaternion<f32>,
}

impl Pos3 {
    pub fn new(pos: cgmath::Vector3<f32>) -> Self {
        Self {
            pos,
            rot: cgmath::Quaternion::one(),
        }
    }

    pub fn new_with_rot(pos: cgmath::Vector3<f32>, rot: cgmath::Quaternion<f32>) -> Self {
        Self { pos, rot }
    }
}

impl Pos for Pos3 {
    fn get_pos(&self) -> cgmath::Vector3<f32> {
        self.pos
    }
}

impl Default for Pos3 {
    fn default() -> Self {
        Self {
            pos: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rot: cgmath::Quaternion::one(),
        }
    }
}

/// A component that stores the scale of an object.
#[derive(Component, Debug, Copy, Clone)]
pub enum Scale {
    Uniform(f32),
    NonUniform { x: f32, y: f32, z: f32 },
}

/// A component that stores the rotation of an object.
#[derive(Component, Debug, Copy, Clone)]
pub enum Flip {
    Horizontal,
    Vertical,
    Both,
}
