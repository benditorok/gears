use crate::ecs::components::{self, PlayerMovement, PlayerStats};
use crate::ecs::traits::{Component, Tick};
use cgmath::One;
use gears_macro::Component;
use std::sync::Arc;
use std::time;

use super::Camera;

#[derive(Component, Debug, Clone)]
pub struct Player {
    pub camera: components::Camera,
    pub body: components::physics::RigidBody,
    pub movement: PlayerMovement,
    pub stats: PlayerStats,
}

impl Player {
    pub fn new(position: cgmath::Vector3<f32>, look_at: Option<cgmath::Point3<f32>>) -> Self {
        let body = components::physics::RigidBody {
            position,
            collision_box: components::physics::CollisionBox {
                min: cgmath::Vector3::new(-0.4, 0.0, -0.4),
                max: cgmath::Vector3::new(0.4, 1.8, 0.4),
            },
            mass: 70.0, // typical human mass
            acceleration: cgmath::Vector3::new(0.0, -10.0, 0.0),
            ..Default::default()
        };

        let look_at = look_at.unwrap_or_else(|| cgmath::Point3::new(0.0, 0.0, 1.0));

        let camera = components::Camera::Player {
            position: body.position,
            look_at,
            y_offset: 1.6, // typical eye height
            speed: 5.0,
            sensitivity: 1.0,
            keycodes: components::CameraKeycodes::default(),
        };

        Self {
            camera,
            body,
            movement: PlayerMovement::default(),
            stats: PlayerStats::default(),
        }
    }

    pub fn with_stats(mut self, health: f32, stamina: f32) -> Self {
        self.stats = PlayerStats {
            health,
            max_health: health,
            stamina,
            max_stamina: stamina,
        };
        self
    }

    pub fn with_movement(mut self, move_speed: f32, jump_force: f32) -> Self {
        self.movement = PlayerMovement {
            move_speed,
            jump_force,
            can_jump: true,
        };
        self
    }
}

impl Tick for Player {
    fn on_tick(&mut self, dt: time::Duration) {
        // Update the body position from the camera position
        if let components::Camera::Player { position, .. } = &self.camera {
            let pos = self.body.position + cgmath::Vector3::new(0.0, -1.6, 0.0) * dt.as_secs_f32();
            self.body.position = pos;
        }
    }
}
