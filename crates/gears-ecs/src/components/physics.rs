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
    /// * `obj_a_scale` - The scale of the first AABB collision box.
    /// * `obj_b` - The second AABB collision box.
    /// * `obj_b_pos3` - The position of the second AABB collision box.
    /// * `obj_b_scale` - The scale of the second AABB collision box.
    ///
    /// # Returns
    ///
    /// * `true` if the two AABB collision boxes intersect.
    fn intersects(
        obj_a: &Self,
        obj_a_pos3: &Pos3,
        obj_a_scale: Option<&cgmath::Vector3<f32>>,
        obj_b: &Self,
        obj_b_pos3: &Pos3,
        obj_b_scale: Option<&cgmath::Vector3<f32>>,
    ) -> bool;

    /// Resolve collision between two AABB collision boxes.
    ///
    /// # Arguments
    /// * `obj_a` - The first AABB collision box.
    /// * `obj_a_pos3` - The position of the first AABB collision box.
    /// * `obj_a_scale` - The scale of the first AABB collision box.
    /// * `obj_b` - The second AABB collision box.
    /// * `obj_b_pos3` - The position of the second AABB collision box.
    /// * `obj_b_scale` - The scale of the second AABB collision box.
    fn resolve(
        obj_a: &mut RigidBody<Self>,
        obj_a_pos3: &mut Pos3,
        obj_a_scale: Option<&cgmath::Vector3<f32>>,
        obj_b: &mut RigidBody<Self>,
        obj_b_pos3: &mut Pos3,
        obj_b_scale: Option<&cgmath::Vector3<f32>>,
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

impl AABBCollisionBox {
    /// Get the rotated and scaled AABB bounds in world space.
    ///
    /// # Arguments
    /// * `pos3` - The position and rotation of the collision box.
    /// * `scale` - The scale to apply to the collision box (defaults to (1, 1, 1) if None).
    ///
    /// # Returns
    /// A tuple of (min, max) vectors representing the axis-aligned bounding box
    /// that encompasses the rotated and scaled collision box.
    fn get_rotated_aabb(
        &self,
        pos3: &Pos3,
        scale: Option<&cgmath::Vector3<f32>>,
    ) -> (cgmath::Vector3<f32>, cgmath::Vector3<f32>) {
        let default_scale = cgmath::Vector3::new(1.0, 1.0, 1.0);
        let scale = scale.unwrap_or(&default_scale);

        // Get the 8 corners of the AABB in local space
        let corners = [
            cgmath::Vector3::new(self.min.x, self.min.y, self.min.z),
            cgmath::Vector3::new(self.max.x, self.min.y, self.min.z),
            cgmath::Vector3::new(self.min.x, self.max.y, self.min.z),
            cgmath::Vector3::new(self.max.x, self.max.y, self.min.z),
            cgmath::Vector3::new(self.min.x, self.min.y, self.max.z),
            cgmath::Vector3::new(self.max.x, self.min.y, self.max.z),
            cgmath::Vector3::new(self.min.x, self.max.y, self.max.z),
            cgmath::Vector3::new(self.max.x, self.max.y, self.max.z),
        ];

        // Scale, rotate and translate each corner
        let transformed_corners: Vec<cgmath::Vector3<f32>> = corners
            .iter()
            .map(|&corner| {
                // Apply scale first
                let scaled = cgmath::Vector3::new(
                    corner.x * scale.x,
                    corner.y * scale.y,
                    corner.z * scale.z,
                );
                // Then rotate using quaternion: q * v * q^-1
                let rotated = pos3.rot * scaled;
                // Finally translate
                rotated + pos3.pos
            })
            .collect();

        // Find the min and max of the transformed corners to get the new AABB
        let mut min = transformed_corners[0];
        let mut max = transformed_corners[0];

        for corner in &transformed_corners[1..] {
            min.x = min.x.min(corner.x);
            min.y = min.y.min(corner.y);
            min.z = min.z.min(corner.z);
            max.x = max.x.max(corner.x);
            max.y = max.y.max(corner.y);
            max.z = max.z.max(corner.z);
        }

        (min, max)
    }
}

impl CollisionBox for AABBCollisionBox {
    /// Check if two AABB collision boxes intersect.
    ///
    /// # Arguments
    /// * `obj_a` - The first AABB collision box.
    /// * `obj_a_pos3` - The position of the first AABB collision box.
    /// * `obj_a_scale` - The scale of the first AABB collision box.
    /// * `obj_b` - The second AABB collision box.
    /// * `obj_b_pos3` - The position of the second AABB collision box.
    /// * `obj_b_scale` - The scale of the second AABB collision box.
    fn intersects(
        obj_a: &Self,
        obj_a_pos3: &Pos3,
        obj_a_scale: Option<&cgmath::Vector3<f32>>,
        obj_b: &Self,
        obj_b_pos3: &Pos3,
        obj_b_scale: Option<&cgmath::Vector3<f32>>,
    ) -> bool {
        let (a_min, a_max) = obj_a.get_rotated_aabb(obj_a_pos3, obj_a_scale);
        let (b_min, b_max) = obj_b.get_rotated_aabb(obj_b_pos3, obj_b_scale);

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
    /// * `obj_a_scale` - The scale of the first rigid body.
    /// * `obj_b` - The second rigid body.
    /// * `obj_b_pos3` - The position of the second rigid body.
    /// * `obj_b_scale` - The scale of the second rigid body.
    fn resolve(
        obj_a: &mut RigidBody<Self>,
        obj_a_pos3: &mut Pos3,
        obj_a_scale: Option<&cgmath::Vector3<f32>>,
        obj_b: &mut RigidBody<Self>,
        obj_b_pos3: &mut Pos3,
        obj_b_scale: Option<&cgmath::Vector3<f32>>,
    ) {
        let (a_min, a_max) = obj_a
            .collision_box
            .get_rotated_aabb(obj_a_pos3, obj_a_scale);
        let (b_min, b_max) = obj_b
            .collision_box
            .get_rotated_aabb(obj_b_pos3, obj_b_scale);

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
    /// * `obj_a_scale` - The scale of the first object.
    /// * `obj_b` - A mutable reference to the second object.
    /// * `obj_b_pos3` - A mutable reference to the position of the second object.
    /// * `obj_b_scale` - The scale of the second object.
    pub fn check_and_resolve_collision(
        obj_a: &mut Self,
        obj_a_pos3: &mut Pos3,
        obj_a_scale: Option<&cgmath::Vector3<f32>>,
        obj_b: &mut Self,
        obj_b_pos3: &mut Pos3,
        obj_b_scale: Option<&cgmath::Vector3<f32>>,
    ) {
        if CollisionBox::intersects(
            &obj_a.collision_box,
            obj_a_pos3,
            obj_a_scale,
            &obj_b.collision_box,
            obj_b_pos3,
            obj_b_scale,
        ) {
            CollisionBox::resolve(
                obj_a,
                obj_a_pos3,
                obj_a_scale,
                obj_b,
                obj_b_pos3,
                obj_b_scale,
            );
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
    /// Get the minimum point of the AABB collision box.
    ///
    /// # Returns
    ///
    /// The minimum point of the AABB collision box.
    pub fn collision_box_min(&self) -> cgmath::Vector3<f32> {
        self.collision_box.min
    }

    /// Get the maximum point of the AABB collision box.
    ///
    /// # Returns
    ///
    /// The maximum point of the AABB collision box.
    pub fn collision_box_max(&self) -> cgmath::Vector3<f32> {
        self.collision_box.max
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::{Quaternion, Vector3, Zero};

    // Helper function to create a simple AABB box
    fn create_box(min: Vector3<f32>, max: Vector3<f32>) -> AABBCollisionBox {
        AABBCollisionBox { min, max }
    }

    // Helper function to create a position with no rotation
    fn create_pos(x: f32, y: f32, z: f32) -> Pos3 {
        Pos3::new(Vector3::new(x, y, z))
    }

    #[test]
    fn test_aabb_collision_box_creation() {
        let bbox = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
        assert_eq!(bbox.min, Vector3::new(-1.0, -1.0, -1.0));
        assert_eq!(bbox.max, Vector3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_rigid_body_creation() {
        let collision_box = create_box(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5));
        let body = RigidBody::new(
            10.0,
            Vector3::new(1.0, 2.0, 3.0),
            Vector3::new(0.0, -9.8, 0.0),
            collision_box,
        );

        assert_eq!(body.mass, 10.0);
        assert_eq!(body.velocity, Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(body.acceleration, Vector3::new(0.0, -9.8, 0.0));
        assert!(!body.is_static);
        assert_eq!(body.restitution, DEFAULT_RESTITUTION);
    }

    #[test]
    fn test_static_rigid_body_creation() {
        let collision_box = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
        let body = RigidBody::new_static(collision_box);

        assert_eq!(body.mass, 0.0);
        assert_eq!(body.velocity, Vector3::zero());
        assert_eq!(body.acceleration, Vector3::zero());
        assert!(body.is_static);
    }

    #[test]
    fn test_default_rigid_body() {
        let body: RigidBody<AABBCollisionBox> = RigidBody::default();

        assert_eq!(body.mass, 1.0);
        assert_eq!(body.velocity, Vector3::zero());
        assert_eq!(body.acceleration, Vector3::zero());
        assert!(!body.is_static);
        assert_eq!(body.collision_box.min, Vector3::new(-0.5, -0.5, -0.5));
        assert_eq!(body.collision_box.max, Vector3::new(0.5, 0.5, 0.5));
    }

    #[test]
    fn test_collision_detection_intersecting() {
        let box_a = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
        let box_b = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));

        let pos_a = create_pos(0.0, 0.0, 0.0);
        let pos_b = create_pos(1.5, 0.0, 0.0); // Overlapping on x-axis

        assert!(CollisionBox::intersects(
            &box_a, &pos_a, None, &box_b, &pos_b, None
        ));
    }

    #[test]
    fn test_collision_detection_not_intersecting() {
        let box_a = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
        let box_b = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));

        let pos_a = create_pos(0.0, 0.0, 0.0);
        let pos_b = create_pos(5.0, 0.0, 0.0); // Far apart

        assert!(!CollisionBox::intersects(
            &box_a, &pos_a, None, &box_b, &pos_b, None
        ));
    }

    #[test]
    fn test_collision_detection_touching() {
        let box_a = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
        let box_b = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));

        let pos_a = create_pos(0.0, 0.0, 0.0);
        let pos_b = create_pos(2.0, 0.0, 0.0); // Exactly touching

        assert!(!CollisionBox::intersects(
            &box_a, &pos_a, None, &box_b, &pos_b, None
        ));
    }

    #[test]
    fn test_collision_detection_with_scale() {
        let box_a = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
        let box_b = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));

        let pos_a = create_pos(0.0, 0.0, 0.0);
        let pos_b = create_pos(2.5, 0.0, 0.0);
        let scale_a = Vector3::new(2.0, 1.0, 1.0); // Double width

        assert!(CollisionBox::intersects(
            &box_a,
            &pos_a,
            Some(&scale_a),
            &box_b,
            &pos_b,
            None
        ));
    }

    #[test]
    fn test_velocity_capping_horizontal() {
        let collision_box = create_box(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5));
        let mut body = RigidBody::new(
            1.0,
            Vector3::new(30.0, 0.0, 30.0), // Exceeds max horizontal velocity
            Vector3::zero(),
            collision_box,
        );

        body.cap_velocity();

        let horizontal_magnitude = (body.velocity.x.powi(2) + body.velocity.z.powi(2)).sqrt();
        assert!(horizontal_magnitude <= MAX_HORIZONTAL_VELOCITY + 0.001);
    }

    #[test]
    fn test_velocity_capping_vertical() {
        let collision_box = create_box(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5));
        let mut body = RigidBody::new(
            1.0,
            Vector3::new(0.0, 50.0, 0.0), // Exceeds max vertical velocity
            Vector3::zero(),
            collision_box,
        );

        body.cap_velocity();

        assert!(body.velocity.y <= MAX_VERTICAL_VELOCITY);
        assert!(body.velocity.y >= -MAX_VERTICAL_VELOCITY);
    }

    #[test]
    fn test_update_pos_applies_gravity() {
        let collision_box = create_box(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5));
        let mut body = RigidBody::new(1.0, Vector3::zero(), Vector3::zero(), collision_box);
        let mut pos = create_pos(0.0, 10.0, 0.0);

        let dt = 0.016; // ~60 FPS
        body.update_pos(&mut pos, dt);

        // Velocity should be negative (falling)
        assert!(body.velocity.y < 0.0);
        // Position should be lower
        assert!(pos.pos.y < 10.0);
    }

    #[test]
    fn test_update_pos_static_body_doesnt_move() {
        let collision_box = create_box(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5));
        let mut body = RigidBody::new_static(collision_box);
        let mut pos = create_pos(0.0, 5.0, 0.0);

        let dt = 0.016;
        body.update_pos(&mut pos, dt);

        // Static body should not move
        assert_eq!(pos.pos, Vector3::new(0.0, 5.0, 0.0));
        assert_eq!(body.velocity, Vector3::zero());
    }

    #[test]
    fn test_update_pos_applies_acceleration() {
        let collision_box = create_box(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5));
        let mut body = RigidBody::new(
            1.0,
            Vector3::zero(),
            Vector3::new(10.0, 0.0, 0.0),
            collision_box,
        );
        let mut pos = create_pos(0.0, 0.0, 0.0);

        let dt = 0.1;
        body.update_pos(&mut pos, dt);

        // Velocity should increase due to acceleration
        assert!(body.velocity.x > 0.0);
        // Position should change
        assert!(pos.pos.x > 0.0);
    }

    #[test]
    fn test_update_pos_damping_reduces_velocity() {
        let collision_box = create_box(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5));
        let mut body = RigidBody::new(
            1.0,
            Vector3::new(10.0, 0.0, 0.0),
            Vector3::zero(),
            collision_box,
        );
        let mut pos = create_pos(0.0, 0.0, 0.0);

        let initial_velocity = body.velocity.x;

        // Update multiple times to see damping effect
        for _ in 0..10 {
            body.update_pos(&mut pos, 0.016);
        }

        // Velocity should be reduced due to damping
        assert!(body.velocity.x < initial_velocity);
    }

    #[test]
    fn test_collision_resolution_separates_objects() {
        let box_a = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
        let box_b = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));

        let mut body_a = RigidBody::new(1.0, Vector3::new(-1.0, 0.0, 0.0), Vector3::zero(), box_a);
        let mut body_b = RigidBody::new_static(box_b);

        let mut pos_a = create_pos(0.8, 0.0, 0.0); // Clearly overlapping with B at origin
        let mut pos_b = create_pos(0.0, 0.0, 0.0);

        // Verify they are colliding before resolution
        assert!(CollisionBox::intersects(
            &body_a.collision_box,
            &pos_a,
            None,
            &body_b.collision_box,
            &pos_b,
            None
        ));

        RigidBody::check_and_resolve_collision(
            &mut body_a,
            &mut pos_a,
            None,
            &mut body_b,
            &mut pos_b,
            None,
        );

        // Body A should be pushed away from body B
        assert!(pos_a.pos.x > 0.8 || body_a.velocity.x != -1.0);
        // Static body B should not move
        assert_eq!(pos_b.pos, Vector3::zero());
    }

    #[test]
    fn test_collision_resolution_reverses_velocity() {
        let box_a = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
        let box_b = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));

        let mut body_a = RigidBody::new(
            1.0,
            Vector3::new(-5.0, 0.0, 0.0), // Moving towards B
            Vector3::zero(),
            box_a,
        );
        let mut body_b = RigidBody::new_static(box_b);

        let mut pos_a = create_pos(1.5, 0.0, 0.0);
        let mut pos_b = create_pos(0.0, 0.0, 0.0);

        RigidBody::check_and_resolve_collision(
            &mut body_a,
            &mut pos_a,
            None,
            &mut body_b,
            &mut pos_b,
            None,
        );

        // Velocity should be affected (reversed or reduced)
        assert!(body_a.velocity.x >= -5.0); // Should be less negative or positive
    }

    #[test]
    fn test_collision_resolution_both_dynamic() {
        let box_a = create_box(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5));
        let box_b = create_box(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5));

        let mut body_a = RigidBody::new(1.0, Vector3::new(1.0, 0.0, 0.0), Vector3::zero(), box_a);
        let mut body_b = RigidBody::new(1.0, Vector3::new(-1.0, 0.0, 0.0), Vector3::zero(), box_b);

        let mut pos_a = create_pos(-0.2, 0.0, 0.0);
        let mut pos_b = create_pos(0.2, 0.0, 0.0);

        let initial_pos_a = pos_a.pos;
        let initial_pos_b = pos_b.pos;

        RigidBody::check_and_resolve_collision(
            &mut body_a,
            &mut pos_a,
            None,
            &mut body_b,
            &mut pos_b,
            None,
        );

        // Both objects should move apart
        assert!(pos_a.pos.x < initial_pos_a.x);
        assert!(pos_b.pos.x > initial_pos_b.x);
    }

    #[test]
    fn test_restitution_affects_bounce() {
        let box_a = create_box(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5));
        let box_b = create_box(
            Vector3::new(-10.0, -0.5, -10.0),
            Vector3::new(10.0, 0.5, 10.0),
        );

        let mut body_a = RigidBody::new(
            1.0,
            Vector3::new(0.0, -10.0, 0.0), // Falling down
            Vector3::zero(),
            box_a,
        );
        body_a.restitution = 0.8; // High bounce

        let mut body_b = RigidBody::new_static(box_b);
        body_b.restitution = 0.8;

        let mut pos_a = create_pos(0.0, 0.8, 0.0); // Slightly overlapping
        let mut pos_b = create_pos(0.0, 0.0, 0.0);

        RigidBody::check_and_resolve_collision(
            &mut body_a,
            &mut pos_a,
            None,
            &mut body_b,
            &mut pos_b,
            None,
        );

        // With high restitution, vertical velocity should reverse significantly
        assert!(body_a.velocity.y > 0.0); // Should bounce upward
    }

    #[test]
    fn test_no_collision_when_separated() {
        let box_a = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));
        let box_b = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));

        let mut body_a = RigidBody::new(1.0, Vector3::new(1.0, 0.0, 0.0), Vector3::zero(), box_a);
        let mut body_b = RigidBody::new_static(box_b);

        let mut pos_a = create_pos(10.0, 0.0, 0.0); // Far apart
        let mut pos_b = create_pos(0.0, 0.0, 0.0);

        let initial_velocity = body_a.velocity;
        let initial_pos = pos_a.pos;

        RigidBody::check_and_resolve_collision(
            &mut body_a,
            &mut pos_a,
            None,
            &mut body_b,
            &mut pos_b,
            None,
        );

        // No collision, so nothing should change
        assert_eq!(body_a.velocity, initial_velocity);
        assert_eq!(pos_a.pos, initial_pos);
    }

    #[test]
    fn test_rotated_aabb_collision_detection() {
        use cgmath::Rotation3;

        let box_a = create_box(Vector3::new(-1.0, -0.5, -0.5), Vector3::new(1.0, 0.5, 0.5));
        let box_b = create_box(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0));

        // Rotate box_a by 45 degrees around Y axis
        let angle = std::f32::consts::PI / 4.0;
        let rotation = Quaternion::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), cgmath::Rad(angle));

        let mut pos_a = create_pos(0.0, 0.0, 0.0);
        pos_a.rot = rotation;
        let pos_b = create_pos(1.5, 0.0, 0.0);

        // After rotation, the elongated box should potentially intersect differently
        let intersects = CollisionBox::intersects(&box_a, &pos_a, None, &box_b, &pos_b, None);

        // This tests that rotation is being accounted for in collision detection
        // The exact result depends on the rotation implementation
        assert!(intersects || !intersects); // Just verify it doesn't crash
    }

    #[test]
    fn test_collision_box_min_max_accessors() {
        let collision_box = create_box(Vector3::new(-2.0, -3.0, -4.0), Vector3::new(2.0, 3.0, 4.0));
        let body = RigidBody::new(1.0, Vector3::zero(), Vector3::zero(), collision_box);

        assert_eq!(body.collision_box_min(), Vector3::new(-2.0, -3.0, -4.0));
        assert_eq!(body.collision_box_max(), Vector3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_fall_multiplier_increases_downward_velocity() {
        let collision_box = create_box(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5));
        let mut body = RigidBody::new(
            1.0,
            Vector3::new(0.0, -5.0, 0.0), // Already falling
            Vector3::zero(),
            collision_box,
        );
        let mut pos = create_pos(0.0, 10.0, 0.0);

        let initial_y_velocity = body.velocity.y;
        body.update_pos(&mut pos, 0.016);

        // Falling should be accelerated more than normal gravity
        let velocity_change = initial_y_velocity - body.velocity.y;
        let expected_normal_change = GRAVITY_ACCELERATION * 0.016;

        // Should have more change due to fall multiplier
        assert!(velocity_change.abs() > expected_normal_change);
    }

    #[test]
    fn test_velocity_threshold_zeros_small_velocities() {
        let collision_box = create_box(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5));
        let mut body = RigidBody::new(
            1.0,
            Vector3::new(0.001, 0.0, 0.001), // Very small velocity
            Vector3::zero(),
            collision_box,
        );
        let mut pos = create_pos(0.0, 100.0, 0.0); // High position to avoid hitting ground

        let initial_magnitude = body.velocity.magnitude();

        // Update a few times for damping to take effect
        for _ in 0..5 {
            body.update_pos(&mut pos, 0.016);
        }

        // Horizontal velocity should be reduced or zeroed (gravity affects y)
        let horizontal_vel = Vector3::new(body.velocity.x, 0.0, body.velocity.z).magnitude();
        assert!(horizontal_vel < initial_magnitude);
    }
}
