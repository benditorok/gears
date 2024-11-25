use super::transforms::Pos3;
use crate::{ecs::traits::Component, prelude::ViewController};
use cgmath::InnerSpace;
use gears_macro::Component;

const MAX_HORIZONTAL_VELOCITY: f32 = 20.0;
const MAX_VERTICAL_VELOCITY: f32 = 40.0;

#[derive(Component, Debug, Clone)]
pub struct CollisionBox {
    pub min: cgmath::Vector3<f32>,
    pub max: cgmath::Vector3<f32>,
}

#[derive(Component, Debug, Clone)]
pub struct RigidBody {
    pub mass: f32,
    pub velocity: cgmath::Vector3<f32>,
    pub acceleration: cgmath::Vector3<f32>,
    pub(crate) collision_box: CollisionBox,
    pub(crate) is_static: bool,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
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
        mass: f32,
        velocity: cgmath::Vector3<f32>,
        acceleration: cgmath::Vector3<f32>,
        collision_box: CollisionBox,
    ) -> Self {
        Self {
            mass,
            velocity,
            acceleration,
            collision_box,
            is_static: false,
        }
    }

    pub fn new_static(collision_box: CollisionBox) -> Self {
        Self {
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

    pub fn update_pos(&mut self, pos3: &mut Pos3, dt: f32) {
        if (!self.is_static) {
            let acceleration_threshold = 0.01;
            let is_accelerating = self.acceleration.magnitude() > acceleration_threshold;

            // Use different damping coefficients based on acceleration state
            let damping_coefficient = if is_accelerating {
                2.0 // Normal damping when accelerating
            } else {
                6.0 // Strong damping when no acceleration
            };

            let damping_factor = (-damping_coefficient * dt).exp();
            let min_velocity = 0.01;

            // Update velocity based on acceleration
            self.velocity += self.acceleration * dt;

            // Apply damping to velocity
            self.velocity *= damping_factor;

            // Set velocity to zero if it's below the minimum threshold
            if self.velocity.magnitude() < min_velocity {
                self.velocity = cgmath::Vector3::new(0.0, 0.0, 0.0);
            }

            // // Print debug information
            // println!(
            //     "Velocity: ({:.2}, {:.2}, {:.2}), Acceleration: ({:.2}, {:.2}, {:.2}), Is Accelerating: {}",
            //     self.velocity.x,
            //     self.velocity.y,
            //     self.velocity.z,
            //     self.acceleration.x,
            //     self.acceleration.y,
            //     self.acceleration.z,
            //     is_accelerating
            // );

            // Update position based on velocity
            pos3.pos += self.velocity * dt;
        }
    }

    pub fn check_and_resolve_collision(
        obj_a: &mut Self,
        obj_a_pos3: &mut Pos3,
        obj_b: &mut Self,
        obj_b_pos3: &mut Pos3,
    ) {
        let a_min = obj_a_pos3.pos + obj_a.collision_box.min;
        let a_max = obj_a_pos3.pos + obj_a.collision_box.max;
        let b_min = obj_b_pos3.pos + obj_b.collision_box.min;
        let b_max = obj_b_pos3.pos + obj_b.collision_box.max;

        if a_min.x < b_max.x
            && a_max.x > b_min.x
            && a_min.y < b_max.y
            && a_max.y > b_min.y
            && a_min.z < b_max.z
            && a_max.z > b_min.z
        {
            // Calculate overlap depths
            let overlap_x = (a_max.x.min(b_max.x)) - (a_min.x.max(b_min.x));
            let overlap_y = (a_max.y.min(b_max.y)) - (a_min.y.max(b_min.y));
            let overlap_z = (a_max.z.min(b_max.z)) - (a_min.z.max(b_min.z));

            // Find axis of minimum penetration
            let (min_overlap, axis) = if overlap_x < overlap_y && overlap_x < overlap_z {
                (overlap_x, 0)
            } else if overlap_y < overlap_z {
                (overlap_y, 1)
            } else {
                (overlap_z, 2)
            };

            let center_diff = obj_b_pos3.pos - obj_a_pos3.pos;
            let normal = match axis {
                0 => cgmath::Vector3::new(if center_diff.x > 0.0 { 1.0 } else { -1.0 }, 0.0, 0.0),
                1 => cgmath::Vector3::new(0.0, if center_diff.y > 0.0 { 1.0 } else { -1.0 }, 0.0),
                _ => cgmath::Vector3::new(0.0, 0.0, if center_diff.z > 0.0 { 1.0 } else { -1.0 }),
            };

            let relative_velocity = obj_b.velocity - obj_a.velocity;
            let vel_along_normal = relative_velocity.dot(normal);

            // Only resolve collision if objects are moving toward each other
            if vel_along_normal > 0.0 {
                return;
            }

            let restitution = 0.3; // Reduced bounciness further
            let friction = 0.8; // Add friction coefficient

            let inv_mass_a = if obj_a.is_static {
                0.0
            } else {
                1.0 / obj_a.mass
            };
            let inv_mass_b = if obj_b.is_static {
                0.0
            } else {
                1.0 / obj_b.mass
            };

            // Separate objects based on overlap
            if !obj_a.is_static {
                obj_a_pos3.pos -= normal * min_overlap * (inv_mass_a / (inv_mass_a + inv_mass_b));
            }
            if !obj_b.is_static {
                obj_b_pos3.pos += normal * min_overlap * (inv_mass_b / (inv_mass_a + inv_mass_b));
            }

            // Apply impulse only if relative velocity is above threshold
            let velocity_threshold = 0.1;
            if vel_along_normal.abs() > velocity_threshold {
                let impulse_scalar =
                    -(1.0 + restitution) * vel_along_normal / (inv_mass_a + inv_mass_b);
                let impulse = normal * impulse_scalar;

                if !obj_a.is_static {
                    obj_a.velocity -= impulse * inv_mass_a;
                    // Apply friction
                    let tangent_velocity = relative_velocity - (normal * vel_along_normal);
                    if tangent_velocity.magnitude() > 0.0 {
                        obj_a.velocity -= tangent_velocity.normalize()
                            * friction
                            * impulse_scalar.abs()
                            * inv_mass_a;
                    }
                }
                if !obj_b.is_static {
                    obj_b.velocity += impulse * inv_mass_b;
                    // Apply friction
                    let tangent_velocity = relative_velocity - (normal * vel_along_normal);
                    if tangent_velocity.magnitude() > 0.0 {
                        obj_b.velocity += tangent_velocity.normalize()
                            * friction
                            * impulse_scalar.abs()
                            * inv_mass_b;
                    }
                }
            } else {
                // If velocity is below threshold, stop the movement along the collision normal
                if !obj_a.is_static {
                    obj_a.velocity -= normal * vel_along_normal * 0.5;
                }
                if !obj_b.is_static {
                    obj_b.velocity += normal * vel_along_normal * 0.5;
                }
            }
        }
    }

    pub fn cap_velocity(&mut self) {
        // Cap horizontal velocity (x and z)
        let horizontal_velocity = cgmath::Vector3::new(self.velocity.x, 0.0, self.velocity.z);
        if horizontal_velocity.magnitude() > MAX_HORIZONTAL_VELOCITY {
            let normalized = horizontal_velocity.normalize();
            self.velocity.x = normalized.x * MAX_HORIZONTAL_VELOCITY;
            self.velocity.z = normalized.z * MAX_HORIZONTAL_VELOCITY;
        }

        // Cap vertical velocity (y)
        self.velocity.y = self
            .velocity
            .y
            .clamp(-MAX_VERTICAL_VELOCITY, MAX_VERTICAL_VELOCITY);
    }
}
