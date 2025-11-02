use super::transforms::Pos3;
use crate::Component;
use cgmath::{InnerSpace, Zero};
use gears_macro::Component;
use std::fmt::Debug;

const MAX_HORIZONTAL_VELOCITY: f32 = 20.0;
const MAX_VERTICAL_VELOCITY: f32 = 40.0;
const POSITION_CORRECTION_FACTOR: f32 = 0.4;
const POSITION_CORRECTION_SLOP: f32 = 0.01;
const FRICTION_COEFFICIENT: f32 = 0.8;
const VELOCITY_THRESHOLD: f32 = 0.05;

// Default restitution moved to object-specific property
const DEFAULT_RESTITUTION: f32 = 0.2;

// Gravity parameters
const GRAVITY_ACCELERATION: f32 = 28.0;
const FALL_MULTIPLIER: f32 = 1.75;
const APEX_MULTIPLIER: f32 = 0.6;

pub trait CollisionBox {
    /// Check if two AABB collision boxes intersect.
    ///
    /// # Arguments
    /// * `obj_a` - The first AABB collision box.
    /// * `obj_a_pos3` - The position of the first AABB collision box.
    /// * `obj_b` - The second AABB collision box.
    /// * `obj_b_pos3` - The position of the second AABB collision box.
    ///
    /// # Returns
    ///
    /// * `true` if the two AABB collision boxes intersect.
    fn intersects(obj_a: &Self, obj_a_pos3: &Pos3, obj_b: &Self, obj_b_pos3: &Pos3) -> bool;

    /// Resolve collision between two AABB collision boxes.
    ///
    /// # Arguments
    /// * `obj_a` - The first AABB collision box.
    /// * `obj_a_pos3` - The position of the first AABB collision box.
    /// * `obj_b` - The second AABB collision box.
    /// * `obj_b_pos3` - The position of the second AABB collision box.
    fn resolve(
        obj_a: &mut RigidBody<Self>,
        obj_a_pos3: &mut Pos3,
        obj_b: &mut RigidBody<Self>,
        obj_b_pos3: &mut Pos3,
    ) where
        Self: Sized;
}

/// AABB collision box used for collision detection and resolution.
#[derive(Component, Debug, Clone)]
pub struct AABBCollisionBox {
    /// Minimum point - front lower left corner of the AABB collision box
    pub min: cgmath::Vector3<f32>,
    /// Maximum point - back upper right corner of the AABB collision box
    pub max: cgmath::Vector3<f32>,
}

impl CollisionBox for AABBCollisionBox {
    /// Check if two AABB collision boxes intersect.
    ///
    /// # Arguments
    /// * `obj_a` - The first AABB collision box.
    /// * `obj_a_pos3` - The position of the first AABB collision box.
    /// * `obj_b` - The second AABB collision box.
    /// * `obj_b_pos3` - The position of the second AABB collision box.
    fn intersects(obj_a: &Self, obj_a_pos3: &Pos3, obj_b: &Self, obj_b_pos3: &Pos3) -> bool {
        let a_min = obj_a_pos3.pos + obj_a.min;
        let a_max = obj_a_pos3.pos + obj_a.max;
        let b_min = obj_b_pos3.pos + obj_b.min;
        let b_max = obj_b_pos3.pos + obj_b.max;

        a_min.x < b_max.x
            && a_max.x > b_min.x
            && a_min.y < b_max.y
            && a_max.y > b_min.y
            && a_min.z < b_max.z
            && a_max.z > b_min.z
    }

    /// Resolve collision between two AABB collision boxes.
    ///
    /// # Arguments
    /// * `obj_a` - The first rigid body.
    /// * `obj_a_pos3` - The position of the first rigid body.
    /// * `obj_b` - The second rigid body.
    /// * `obj_b_pos3` - The position of the second rigid body.
    fn resolve(
        obj_a: &mut RigidBody<Self>,
        obj_a_pos3: &mut Pos3,
        obj_b: &mut RigidBody<Self>,
        obj_b_pos3: &mut Pos3,
    ) {
        let a_min = obj_a_pos3.pos + obj_a.collision_box.min;
        let a_max = obj_a_pos3.pos + obj_a.collision_box.max;
        let b_min = obj_b_pos3.pos + obj_b.collision_box.min;
        let b_max = obj_b_pos3.pos + obj_b.collision_box.max;

        // Calculate overlap depths
        let overlap_x = (a_max.x.min(b_max.x)) - (a_min.x.max(b_min.x));
        let overlap_y = (a_max.y.min(b_max.y)) - (a_min.y.max(b_min.y));
        let overlap_z = (a_max.z.min(b_max.z)) - (a_min.z.max(b_min.z));

        // Calculate center points for better normal direction
        let a_center = (a_min + a_max) * 0.5;
        let b_center = (b_min + b_max) * 0.5;
        let center_diff = b_center - a_center;

        // Find axis of minimum penetration with direction awareness
        let (min_overlap, normal) = if overlap_x <= overlap_y && overlap_x <= overlap_z {
            let dir = if center_diff.x > 0.0 { 1.0 } else { -1.0 };
            (overlap_x, cgmath::Vector3::new(dir, 0.0, 0.0))
        } else if overlap_y <= overlap_z {
            let dir = if center_diff.y > 0.0 { 1.0 } else { -1.0 };
            (overlap_y, cgmath::Vector3::new(0.0, dir, 0.0))
        } else {
            let dir = if center_diff.z > 0.0 { 1.0 } else { -1.0 };
            (overlap_z, cgmath::Vector3::new(0.0, 0.0, dir))
        };

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
        let total_inv_mass = inv_mass_a + inv_mass_b;

        if total_inv_mass <= 0.0 {
            return;
        } // Both static, nothing to do

        // Calculate relative velocity
        let relative_velocity = obj_b.velocity - obj_a.velocity;
        let vel_along_normal = relative_velocity.dot(normal);

        // Check if objects are separating
        if vel_along_normal > 0.0 {
            return;
        }

        // Apply position correction to prevent sinking (Baumgarte stabilization)
        if min_overlap > POSITION_CORRECTION_SLOP {
            let correction_magnitude = (min_overlap - POSITION_CORRECTION_SLOP)
                * POSITION_CORRECTION_FACTOR
                / total_inv_mass;
            let correction = normal * correction_magnitude;

            if !obj_a.is_static {
                obj_a_pos3.pos -= correction * inv_mass_a;
            }
            if !obj_b.is_static {
                obj_b_pos3.pos += correction * inv_mass_b;
            }
        }

        // Use the average restitution of the two objects
        let restitution = if vel_along_normal.abs() < VELOCITY_THRESHOLD {
            0.0 // No restitution for slow collisions to prevent bouncing when almost at rest
        } else {
            (obj_a.restitution + obj_b.restitution) * 0.5 // Average restitution of both objects
        };

        // Calculate impulse scalar
        let impulse_scalar = -(1.0 + restitution) * vel_along_normal / total_inv_mass;

        // Apply impulse
        let impulse = normal * impulse_scalar;
        if !obj_a.is_static {
            obj_a.velocity -= impulse * inv_mass_a;
        }
        if !obj_b.is_static {
            obj_b.velocity += impulse * inv_mass_b;
        }

        // Apply friction with improved model
        let tangent_velocity = relative_velocity - normal * vel_along_normal;
        let tangent_speed = tangent_velocity.magnitude();

        if tangent_speed > 0.001 {
            let tangent = tangent_velocity / tangent_speed;
            let friction_impulse = FRICTION_COEFFICIENT * impulse_scalar.abs();

            // Apply friction impulse with clamping to prevent energy gain
            let max_friction = tangent_speed; // Limit friction to current tangent velocity
            let effective_friction_impulse = friction_impulse.min(max_friction * total_inv_mass);

            if !obj_a.is_static {
                obj_a.velocity -= tangent * effective_friction_impulse * inv_mass_a;
            }
            if !obj_b.is_static {
                obj_b.velocity += tangent * effective_friction_impulse * inv_mass_b;
            }
        }

        // Stabilize very slow movements to prevent jittering
        if !obj_a.is_static && obj_a.velocity.magnitude() < VELOCITY_THRESHOLD * 0.5 {
            obj_a.velocity = cgmath::Vector3::zero();
        }
        if !obj_b.is_static && obj_b.velocity.magnitude() < VELOCITY_THRESHOLD * 0.5 {
            obj_b.velocity = cgmath::Vector3::zero();
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RigidBody<T: CollisionBox> {
    /// Mass of the rigid body.
    pub mass: f32,
    /// Velocity of the rigid body.
    pub velocity: cgmath::Vector3<f32>,
    // Acceleration of the rigid body.
    pub acceleration: cgmath::Vector3<f32>,
    // Collision box used to detect the intersection of objects.
    pub collision_box: T,
    /// If a rigid body is static, it cannot be moved or rotated based on external forces.
    pub is_static: bool,
    /// Restitution coefficient (bounciness) - 0.0 means no bounce, 1.0 means perfect bounce
    pub restitution: f32,
}

impl Component for RigidBody<AABBCollisionBox> {}

impl Default for RigidBody<AABBCollisionBox> {
    /// Creates a new rigid body with default properties.
    ///
    /// # Returns
    ///
    /// A new [`RigidBody`] instance.
    fn default() -> Self {
        Self {
            mass: 1.0,
            velocity: cgmath::Vector3::new(0.0, 0.0, 0.0),
            acceleration: cgmath::Vector3::new(0.0, 0.0, 0.0),
            collision_box: AABBCollisionBox {
                min: cgmath::Vector3::new(-0.5, -0.5, -0.5),
                max: cgmath::Vector3::new(0.5, 0.5, 0.5),
            },
            is_static: false,
            restitution: DEFAULT_RESTITUTION,
        }
    }
}

impl<T: CollisionBox> RigidBody<T> {
    /// Creates a new rigid body with the given properties.
    ///
    /// # Arguments
    ///
    /// * `mass` - The mass of the rigid body.
    /// * `velocity` - The initial velocity of the rigid body.
    /// * `acceleration` - The initial acceleration of the rigid body.
    /// * `collision_box` - The collision box of the rigid body.
    ///
    /// # Returns
    ///
    /// A new [`RigidBody`] instance.
    pub fn new(
        mass: f32,
        velocity: cgmath::Vector3<f32>,
        acceleration: cgmath::Vector3<f32>,
        collision_box: T,
    ) -> Self {
        Self {
            mass,
            velocity,
            acceleration,
            collision_box,
            is_static: false,
            restitution: DEFAULT_RESTITUTION,
        }
    }

    /// Creates a new static rigid body with the given collision box.
    ///
    /// # Arguments
    ///
    /// * `collision_box` - The collision box of the static rigid body.
    ///
    /// # Returns
    ///
    /// A new static [`RigidBody`] instance.
    pub fn new_static(collision_box: T) -> Self {
        Self {
            mass: 0.0,
            velocity: cgmath::Vector3::new(0.0, 0.0, 0.0),
            acceleration: cgmath::Vector3::new(0.0, 0.0, 0.0),
            collision_box,
            is_static: true,
            restitution: DEFAULT_RESTITUTION,
        }
    }

    /// Checks if the object is static.
    ///
    /// # Returns
    ///
    /// Returns `true` if the object is static.
    pub fn is_static(&self) -> bool {
        self.is_static
    }

    /// Updates the position of the object based on its velocity and acceleration.
    ///
    /// # Arguments
    ///
    /// * `pos3` - A mutable reference to the position of the object.
    /// * `dt` - The time step for the update.
    pub fn update_pos(&mut self, pos3: &mut Pos3, dt: f32) {
        if !self.is_static {
            let acceleration_threshold = 0.01;
            let falling = self.velocity.y < 0.0;
            let near_apex = self.velocity.y.abs() < 2.0 && self.velocity.y > 0.0;

            // Apply gravity with adaptive system for better jump feel
            if !self.is_static {
                // Choose gravity multiplier based on state
                let gravity_multiplier = if falling {
                    FALL_MULTIPLIER
                } else if near_apex {
                    APEX_MULTIPLIER // Lighter gravity at the peak of jump for hang time
                } else {
                    1.0
                };

                // Apply gravity with appropriate multiplier
                self.velocity.y -= GRAVITY_ACCELERATION * gravity_multiplier * dt;
            }

            let is_accelerating = self.acceleration.magnitude() > acceleration_threshold;

            // Adjust damping for different states
            let damping_coefficient = if is_accelerating {
                1.6 // Reduced damping when accelerating for more responsive movement
            } else if falling {
                1.8 // Slightly reduced damping when falling for better control
            } else {
                3.5 // Reduced general damping for better responsiveness
            };

            let damping_factor = (-damping_coefficient * dt).exp();
            let min_velocity = 0.005;

            // Apply selective damping based on movement state
            if falling {
                // Apply damping only to horizontal components when falling
                self.velocity.x *= damping_factor;
                self.velocity.z *= damping_factor;
            } else {
                // Apply full damping when not falling
                self.velocity *= damping_factor;
            }

            // Add acceleration to velocity (for non-gravity forces)
            self.velocity.x += self.acceleration.x * dt;
            self.velocity.z += self.acceleration.z * dt;

            // Set velocity to zero if it's below the minimum threshold
            if self.velocity.magnitude() < min_velocity {
                self.velocity = cgmath::Vector3::zero();
            }

            // Update position based on velocity
            pos3.pos += self.velocity * dt;
        }
    }

    /// Checks for collision between two objects and resolves it if necessary.
    ///
    /// # Arguments
    ///
    /// * `obj_a` - A mutable reference to the first object.
    /// * `obj_a_pos3` - A mutable reference to the position of the first object.
    /// * `obj_b` - A mutable reference to the second object.
    /// * `obj_b_pos3` - A mutable reference to the position of the second object.
    pub fn check_and_resolve_collision(
        obj_a: &mut Self,
        obj_a_pos3: &mut Pos3,
        obj_b: &mut Self,
        obj_b_pos3: &mut Pos3,
    ) {
        if CollisionBox::intersects(
            &obj_a.collision_box,
            obj_a_pos3,
            &obj_b.collision_box,
            obj_b_pos3,
        ) {
            CollisionBox::resolve(obj_a, obj_a_pos3, obj_b, obj_b_pos3);
        }
    }

    /// Caps the velocity of the object to prevent it from exceeding the maximum allowed velocity.
    ///
    /// # Arguments
    ///
    /// * `self` - A mutable reference to the object whose velocity is to be capped.
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

impl RigidBody<AABBCollisionBox> {
    /// Returns the minimum point of the collision box.
    ///
    /// # Returns
    ///
    /// The minimum point of the AABB collision box.
    pub fn collision_box_min(&self) -> cgmath::Vector3<f32> {
        self.collision_box.min
    }

    /// Returns the maximum point of the collision box.
    ///
    /// # Returns
    ///
    /// The maximum point of the AABB collision box.
    pub fn collision_box_max(&self) -> cgmath::Vector3<f32> {
        self.collision_box.max
    }
}
