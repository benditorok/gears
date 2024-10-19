/// A component that stores the position of any object.
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

/// A component that stores the camera type.
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

/// A component that stores the model type.
pub enum Model<'a> {
    Dynamic { obj_path: &'a str },
    // TODO Static: can't update the pos, etc
}

pub struct Name(pub &'static str);

/// A component that stores the light type.
pub enum Light {
    Point { radius: f32 },
    PointColoured { radius: f32, color: [f32; 3] },
    Ambient,
    AmbientColoured { color: [f32; 3] },
    Directional,
    DirectionalColoured { color: [f32; 3] },
}

/// A component that stores the scale of an object.
pub enum Scale {
    Uniform(f32),
    NonUniform { x: f32, y: f32, z: f32 },
}

/// A component that stores the rotation of an object.
pub enum Flip {
    Horizontal,
    Vertical,
    Both,
}
