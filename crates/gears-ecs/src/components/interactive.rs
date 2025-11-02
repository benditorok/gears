use super::{
    misc::Health,
    physics::{AABBCollisionBox, RigidBody},
};
use crate::{Component, components::controllers::ViewController, components::transforms::Pos3};
use cgmath::{InnerSpace, Vector3};
use gears_macro::Component;

/// A component that tracks shooting intent (set by input handler).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ShootingIntent {
    /// Whether a shoot action should be performed this frame.
    pub should_shoot: bool,
}

impl ShootingIntent {
    /// Creates a new shooting intent.
    pub fn new() -> Self {
        Self {
            should_shoot: false,
        }
    }

    /// Triggers a shoot action.
    pub fn trigger(&mut self) {
        self.should_shoot = true;
    }

    /// Resets the shoot action (should be called after handling).
    pub fn reset(&mut self) {
        self.should_shoot = false;
    }

    /// Checks if shooting is intended.
    pub fn is_shooting(&self) -> bool {
        self.should_shoot
    }
}

/// A component representing a weapon that can be used to attack other entities.
#[derive(Component, Debug, Clone, Copy)]
pub struct Weapon {
    /// The damage dealt by the weapon.
    pub damage: f32,
}

impl Weapon {
    /// Creates a new weapon with the specified damage.
    ///
    /// # Arguments
    ///
    /// * `damage` - The damage dealt by the weapon.
    ///
    /// # Returns
    ///
    /// A new [`Weapon`] instance.
    pub fn new(damage: f32) -> Self {
        Self { damage }
    }

    /// Shoots the weapon at the specified target using raycasting.
    ///
    /// # Arguments
    ///
    /// * `self_pos3` - The position of the shooter.
    /// * `self_view` - The view controller of the shooter.
    /// * `target_pos3` - The position of the target.
    /// * `target_body` - The rigid body of the target.
    /// * `target_health` - The health component of the target.
    ///
    /// # Returns
    ///
    /// `true` if the shot hit the target, `false` otherwise.
    pub fn shoot(
        &self,
        self_pos3: &Pos3,
        self_view: &ViewController,
        target_pos3: &Pos3,
        target_body: &RigidBody<AABBCollisionBox>,
        target_health: &mut Health,
    ) -> bool {
        // Get the shooting direction from the view controller
        let shoot_direction = self_view.get_forward();

        // Ray origin is the shooter's position with head offset (to match camera position)
        let ray_origin = Vector3::new(
            self_pos3.pos.x,
            self_pos3.pos.y + self_view.head_offset,
            self_pos3.pos.z,
        );

        log::debug!(
            "Ray cast - Origin: [{:.2}, {:.2}, {:.2}], Direction: [{:.2}, {:.2}, {:.2}]",
            ray_origin.x,
            ray_origin.y,
            ray_origin.z,
            shoot_direction.x,
            shoot_direction.y,
            shoot_direction.z
        );

        // Perform ray-AABB intersection test
        if Self::ray_intersects_aabb(ray_origin, shoot_direction, target_pos3, target_body) {
            // Hit! Apply damage
            let current_health = target_health.get_health();
            target_health.set_health(current_health - self.damage);
            true
        } else {
            false
        }
    }

    /// Performs ray-AABB intersection test.
    ///
    /// # Arguments
    ///
    /// * `ray_origin` - The origin point of the ray.
    /// * `ray_direction` - The normalized direction of the ray.
    /// * `target_pos3` - The position of the target.
    /// * `target_body` - The rigid body containing the AABB collision box.
    ///
    /// # Returns
    ///
    /// `true` if the ray intersects the AABB, `false` otherwise.
    fn ray_intersects_aabb(
        ray_origin: Vector3<f32>,
        ray_direction: Vector3<f32>,
        target_pos3: &Pos3,
        target_body: &RigidBody<AABBCollisionBox>,
    ) -> bool {
        // Get AABB bounds in world space
        let aabb_min = target_pos3.pos + target_body.collision_box_min();
        let aabb_max = target_pos3.pos + target_body.collision_box_max();

        log::debug!(
            "Target AABB - Min: [{:.2}, {:.2}, {:.2}], Max: [{:.2}, {:.2}, {:.2}]",
            aabb_min.x,
            aabb_min.y,
            aabb_min.z,
            aabb_max.x,
            aabb_max.y,
            aabb_max.z
        );

        // Slab method for ray-AABB intersection
        let mut tmin = 0.0f32;
        let mut tmax = f32::MAX;

        for i in 0..3 {
            let origin_component = ray_origin[i];
            let dir_component = ray_direction[i];
            let min_component = aabb_min[i];
            let max_component = aabb_max[i];

            if dir_component.abs() < 1e-8 {
                // Ray is parallel to slab
                if origin_component < min_component || origin_component > max_component {
                    return false;
                }
            } else {
                let inv_dir = 1.0 / dir_component;
                let mut t1 = (min_component - origin_component) * inv_dir;
                let mut t2 = (max_component - origin_component) * inv_dir;

                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                }

                tmin = tmin.max(t1);
                tmax = tmax.min(t2);

                if tmin > tmax {
                    return false;
                }
            }
        }

        // Check if intersection is valid
        let hit = tmax >= 0.0 && tmin <= tmax;

        log::debug!(
            "Ray-AABB result - tmin: {:.4}, tmax: {:.4}, hit: {}",
            tmin,
            tmax,
            hit
        );

        hit
    }
}
