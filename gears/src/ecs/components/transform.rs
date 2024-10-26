use crate::ecs::traits::Component;

/// A component that stores the position of any object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pos3 {
    pub pos: cgmath::Vector3<f32>,
    pub rot: Option<cgmath::Quaternion<f32>>,
}

impl Component for Pos3 {}

impl crate::ecs::traits::Pos for Pos3 {
    fn get_pos(&self) -> cgmath::Vector3<f32> {
        self.pos
    }
}

impl Default for Pos3 {
    fn default() -> Self {
        Self {
            pos: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rot: None,
        }
    }
}

impl Pos3 {
    pub fn new(pos: cgmath::Vector3<f32>) -> Self {
        Self { pos, rot: None }
    }

    pub fn with_rot(pos: cgmath::Vector3<f32>, rot: cgmath::Quaternion<f32>) -> Self {
        Self {
            pos,
            rot: Some(rot),
        }
    }
}

// impl From<Pos3> for cgmath::Point3<f32> {
//     fn from(val: Pos3) -> Self {
//         cgmath::Point3::new(val.x, val.y, val.z)
//     }
// }

/// A component that stores the scale of an object.
#[derive(Debug, Copy, Clone)]
pub enum Scale {
    Uniform(f32),
    NonUniform { x: f32, y: f32, z: f32 },
}

impl Component for Scale {}

/// A component that stores the rotation of an object.
#[derive(Debug, Copy, Clone)]
pub enum Flip {
    Horizontal,
    Vertical,
    Both,
}

impl Component for Flip {}
