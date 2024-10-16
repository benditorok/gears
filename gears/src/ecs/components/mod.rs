/// A component that stores the position of a 3D object.
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

impl From<Pos3> for cgmath::Point3<f32> {
    fn from(val: Pos3) -> Self {
        cgmath::Point3::new(val.x, val.y, val.z)
    }
}

/// A component that stores the path source of a model.
#[derive(Clone, Copy, Debug)]
pub struct ModelSource<'a>(pub &'a str);

pub enum Camera {
    FPS {
        look_at: Pos3,
        speed: f32,
        sensitivity: f32,
    },
    Fixed {
        look_at: Pos3,
    },
}
