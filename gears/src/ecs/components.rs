use super::traits::Component;
use crate::renderer;

/// A component that stores the position of any object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pos3 {
    pub pos: cgmath::Vector3<f32>,
    pub rot: Option<cgmath::Quaternion<f32>>,
}

impl Component for Pos3 {}

impl renderer::traits::Pos for Pos3 {
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

// impl From<&[f32; 3]> for cgmath::Vector3<f32> {
//     fn from(val: &[f32; 3]) -> Self {
//         cgmath::Vector3::new(val[0], val[1], val[2])
//     }
// }

// impl From<Pos3> for cgmath::Point3<f32> {
//     fn from(val: Pos3) -> Self {
//         cgmath::Point3::new(val.x, val.y, val.z)
//     }
// }

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

/// A component that stores the light type.
#[derive(Debug, Copy, Clone)]
pub enum Light {
    Point {
        radius: f32,
        intensity: f32,
    },
    PointColoured {
        radius: f32,
        color: [f32; 3],
        intensity: f32,
    },
    Ambient {
        intensity: f32,
    },
    AmbientColoured {
        color: [f32; 3],
        intensity: f32,
    },
    Directional {
        direction: [f32; 3],
        intensity: f32,
    },
    DirectionalColoured {
        direction: [f32; 3],
        color: [f32; 3],
        intensity: f32,
    },
}

impl Component for Light {}

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

/// Axis-aligned bounding box component.
#[derive(Debug, Copy, Clone)]
pub(crate) struct AABB {
    pub min: cgmath::Vector3<f32>,
    pub max: cgmath::Vector3<f32>,
}

impl Component for AABB {}

impl AABB {
    fn new(min: cgmath::Vector3<f32>, max: cgmath::Vector3<f32>) -> Self {
        Self { min, max }
    }
}

impl renderer::traits::Collider for AABB {
    fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    fn move_to(&mut self, pos: impl renderer::traits::Pos) {
        let pos = pos.get_pos();
        let diff = pos - cgmath::Vector3::new(self.min.x, self.min.y, self.min.z);
        self.min += diff;
        self.max += diff;
    }
}

/// Collider component.
#[derive(Debug, Copy, Clone)]
pub struct Collider(AABB);

impl Component for Collider {}

impl Collider {
    pub fn new(min: cgmath::Vector3<f32>, max: cgmath::Vector3<f32>) -> Self {
        Self(AABB::new(min, max))
    }
}
