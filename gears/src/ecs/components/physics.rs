use crate::ecs::traits::Component;
use cgmath::{InnerSpace, Rotation3};
use gears_macro::Component;

#[derive(Component, Debug, Clone)]
pub struct CollisionBox {
    pub min: cgmath::Vector3<f32>,
    pub max: cgmath::Vector3<f32>,
}

#[derive(Component, Debug, Clone)]
pub struct RigidBody {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub mass: f32,
    pub velocity: cgmath::Vector3<f32>,
    pub acceleration: cgmath::Vector3<f32>,
    pub collision_box: CollisionBox,
    pub is_static: bool,
}

impl Default for RigidBody {
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
            is_static: false,
        }
    }
}

impl RigidBody {
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
            is_static: false,
        }
    }

    pub fn new_static(
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
        collision_box: CollisionBox,
    ) -> Self {
        Self {
            position,
            rotation,
            mass: 0.0,
            velocity: cgmath::Vector3::new(0.0, 0.0, 0.0),
            acceleration: cgmath::Vector3::new(0.0, 0.0, 0.0),
            collision_box,
            is_static: true,
        }
    }

    pub fn is_static(&self) -> bool {
        self.is_static
    }

    pub fn check_and_resolve_collision(&mut self, other: &mut Self) {
        // Compute AABB for self
        let a_min = self.position + self.collision_box.min;
        let a_max = self.position + self.collision_box.max;

        // Compute AABB for other
        let b_min = other.position + other.collision_box.min;
        let b_max = other.position + other.collision_box.max;

        // Check for collision on all three axes
        if a_min.x < b_max.x
            && a_max.x > b_min.x
            && a_min.y < b_max.y
            && a_max.y > b_min.y
            && a_min.z < b_max.z
            && a_max.z > b_min.z
        {
            // Collision detected

            // Calculate the collision normal
            let normal = (other.position - self.position).normalize();

            // Calculate relative velocity
            let relative_velocity = other.velocity - self.velocity;

            // Calculate relative velocity along the normal
            let vel_along_normal = relative_velocity.dot(normal);

            // Do not resolve if velocities are separating
            if vel_along_normal > 0.0 {
                return;
            }

            // Coefficient of restitution (bounciness)
            let restitution = 0.8;

            // Inverse masses, zero if static
            let inv_mass_self = if self.is_static { 0.0 } else { 1.0 / self.mass };
            let inv_mass_other = if other.is_static {
                0.0
            } else {
                1.0 / other.mass
            };

            // Calculate impulse scalar
            let impulse_scalar =
                -(1.0 + restitution) * vel_along_normal / (inv_mass_self + inv_mass_other);

            // Apply impulse to each body
            let impulse = impulse_scalar * normal;
            if !self.is_static {
                self.velocity -= inv_mass_self * impulse;
            }
            if !other.is_static {
                other.velocity += inv_mass_other * impulse;
            }

            // Positional correction to prevent sinking
            let percent = 0.2; // Penetration percentage to correct
            let slop = 0.01; // Penetration allowance

            // Calculate overlap on each axis
            let overlap_x = (a_max.x.min(b_max.x)) - (a_min.x.max(b_min.x));
            let overlap_y = (a_max.y.min(b_max.y)) - (a_min.y.max(b_min.y));
            let overlap_z = (a_max.z.min(b_max.z)) - (a_min.z.max(b_min.z));

            // Calculate penetration depth as minimum overlap
            let penetration = overlap_x.min(overlap_y.min(overlap_z));

            // Calculate correction vector
            let correction = ((penetration - slop).max(0.0) / (inv_mass_self + inv_mass_other))
                * percent
                * normal;

            // Apply positional correction
            if !self.is_static {
                self.position -= inv_mass_self * correction;
            }
            if !other.is_static {
                other.position += inv_mass_other * correction;
            }
        }
    }
}
