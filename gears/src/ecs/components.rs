use cgmath::Rotation3;

use super::traits::Component;
use crate::renderer;

/// A component that stores the position of any object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pos3 {
    pub pos: cgmath::Vector3<f32>,
    pub rot: Option<cgmath::Quaternion<f32>>,
}

impl Component for Pos3 {}

impl super::traits::Pos for Pos3 {
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

impl super::traits::Collider for AABB {
    fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    fn move_to(&mut self, pos: impl super::traits::Pos) {
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

#[derive(Debug, Clone)]
pub struct CollisionBox {
    pub min: cgmath::Vector3<f32>,
    pub max: cgmath::Vector3<f32>,
}

#[derive(Debug, Clone)]
pub struct PhysicsBody {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub mass: f32,
    pub velocity: cgmath::Vector3<f32>,
    pub acceleration: cgmath::Vector3<f32>,
    pub collision_box: CollisionBox,
}

impl Component for PhysicsBody {}

impl Default for PhysicsBody {
    fn default() -> Self {
        Self {
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Quaternion::from_angle_x(cgmath::Deg(0.0)),
            mass: 1.0,
            velocity: cgmath::Vector3::new(0.0, 0.0, 0.0),
            acceleration: cgmath::Vector3::new(0.0, 0.0, 0.0),
            collision_box: CollisionBox {
                min: cgmath::Vector3::new(-0.5, -0.5, -0.5),
                max: cgmath::Vector3::new(0.5, 0.5, 0.5),
            },
        }
    }
}

impl PhysicsBody {
    pub fn new(
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
        mass: f32,
        velocity: cgmath::Vector3<f32>,
        acceleration: cgmath::Vector3<f32>,
        collision_box: CollisionBox,
    ) -> Self {
        Self {
            position,
            rotation,
            mass,
            velocity,
            acceleration,
            collision_box,
        }
    }

    pub fn check_collision(&self, other: &Self) -> bool {
        let a_min = other.position + other.collision_box.min;
        let a_max = other.position + other.collision_box.max;
        let b_min = other.position + other.collision_box.min;
        let b_max = other.position + other.collision_box.max;

        a_min.x <= b_max.x
            && a_max.x >= b_min.x
            && a_min.y <= b_max.y
            && a_max.y >= b_min.y
            && a_min.z <= b_max.z
            && a_max.z >= b_min.z
    }

    pub fn resolve_collision(&mut self, other: &mut Self) {
        let overlap_x = (other.collision_box.max.x - other.collision_box.min.x)
            .abs()
            .min((other.collision_box.max.x - other.collision_box.min.x).abs());
        let overlap_y = (other.collision_box.max.y - other.collision_box.min.y)
            .abs()
            .min((other.collision_box.max.y - other.collision_box.min.y).abs());
        let overlap_z = (other.collision_box.max.z - other.collision_box.min.z)
            .abs()
            .min((other.collision_box.max.z - other.collision_box.min.z).abs());

        if overlap_x < overlap_y && overlap_x < overlap_z {
            other.position.x -= overlap_x / 2.0;
            other.position.x += overlap_x / 2.0;
        } else if overlap_y < overlap_x && overlap_y < overlap_z {
            other.position.y -= overlap_y / 2.0;
            other.position.y += overlap_y / 2.0;
        } else {
            other.position.z -= overlap_z / 2.0;
            other.position.z += overlap_z / 2.0;
        }
    }
}
