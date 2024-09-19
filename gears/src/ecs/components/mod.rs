use crate::renderer;

/// A component that stores the positiobn of a 3D object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pos3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Pos3 {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl Pos3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// A component that stores the path source of a model.
#[derive(Clone, Copy, Debug)]
pub struct ModelSource<'a>(pub &'a str);

/// A component that stores a type of light source.
#[derive(Clone, Copy, Debug)]
pub enum LightSource {
    Ambient,
    Directional,
    Point,
    Spot,
}

#[derive(Debug, Copy, Clone)]
pub struct PointLight {
    pub position: cgmath::Vector3<f32>,
    pub color: cgmath::Vector3<f32>,
}

impl PointLight {
    fn to_raw(&self) -> renderer::light::LightUniform {
        renderer::light::LightUniform {
            position: self.position.into(),
            _padding: 0,
            color: self.color.into(),
            _padding2: 0,
        }
    }
}
