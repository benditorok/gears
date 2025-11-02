use crate::Component;
use cgmath::One;
use gears_macro::Component;

/// A trait for getting the position of an object.
pub trait Pos {
    /// Gets the position of the object.
    ///
    /// # Returns
    ///
    /// The position of the object as a [`cgmath::Vector3<f32>`].
    fn get_pos(&self) -> cgmath::Vector3<f32>;
}

/// A component that stores the position of any object.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Pos3 {
    /// The position of the object in 3D space.
    pub pos: cgmath::Vector3<f32>,
    /// The rotation of the object as a quaternion.
    pub rot: cgmath::Quaternion<f32>,
}

impl Pos3 {
    /// Creates a new position component with default rotation.
    ///
    /// # Arguments
    ///
    /// * `pos` - The position of the object.
    ///
    /// # Returns
    ///
    /// A new [`Pos3`] instance.
    pub fn new(pos: cgmath::Vector3<f32>) -> Self {
        Self {
            pos,
            rot: cgmath::Quaternion::one(),
        }
    }

    /// Creates a new position component with a specified rotation.
    ///
    /// # Arguments
    ///
    /// * `pos` - The position of the object.
    /// * `rot` - The rotation of the object.
    ///
    /// # Returns
    ///
    /// A new [`Pos3`] instance.
    pub fn new_with_rot(pos: cgmath::Vector3<f32>, rot: cgmath::Quaternion<f32>) -> Self {
        Self { pos, rot }
    }
}

impl Pos for Pos3 {
    /// Gets the position of the object.
    ///
    /// # Returns
    ///
    /// The position of the object as a [`cgmath::Vector3<f32>`].
    fn get_pos(&self) -> cgmath::Vector3<f32> {
        self.pos
    }
}

impl Default for Pos3 {
    /// Creates a new position component with default values.
    ///
    /// # Returns
    ///
    /// A new [`Pos3`] instance.
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
