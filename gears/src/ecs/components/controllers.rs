use crate::{ecs::traits::Tick, prelude::Component, SAFE_FRAC_PI_2};
use cgmath::{InnerSpace, Point3, Rotation3, Vector3};
use gears_macro::Component;
use log::info;
use std::time;
use winit::{event::ElementState, keyboard::KeyCode};

use super::{physics::RigidBody, transforms::Pos3};

const MOVE_ACCELERATION: f32 = 15.0; // Reduced from 50.0
const JUMP_FORCE: f32 = 5.0; // Reduced from 10.0
const GROUND_CHECK_DISTANCE: f32 = 0.1;
const AIR_CONTROL_FACTOR: f32 = 0.2; // Reduced from 0.3

#[derive(Component, Debug, Clone)]
pub struct MovementController {
    pub(crate) speed: f32,
    pub(crate) keycodes: MovementKeycodes,
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            speed: 10.0,
            keycodes: MovementKeycodes::default(),
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
        }
    }
}

impl MovementController {
    pub fn new(speed: f32, keycodes: MovementKeycodes) -> Self {
        Self {
            speed,
            keycodes,
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
        }
    }

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        info!("Processing keyboard input: {:?}, {:?}", key, state);
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };

        if key == self.keycodes.forward {
            self.amount_forward = amount;
            true
        } else if key == self.keycodes.backward {
            self.amount_backward = amount;
            true
        } else if key == self.keycodes.left {
            self.amount_left = amount;
            true
        } else if key == self.keycodes.right {
            self.amount_right = amount;
            true
        } else if key == self.keycodes.up {
            self.amount_up = amount;
            true
        } else if key == self.keycodes.down {
            self.amount_down = amount;
            true
        } else {
            false
        }
    }

    pub fn update_pos(
        &self,
        view_controller: &ViewController,
        pos3: &mut Pos3,
        rigid_body: Option<&mut RigidBody>,
        dt: f32,
    ) {
        info!(
            "Updating position: left: {}, right: {}, up: {}, down: {}, forward: {}, backward: {}",
            self.amount_left,
            self.amount_right,
            self.amount_up,
            self.amount_down,
            self.amount_forward,
            self.amount_backward
        );

        // Calculate forward and right vectors from yaw
        let (sin_yaw, cos_yaw) = view_controller.yaw.0.sin_cos();
        let forward = Vector3::new(cos_yaw, 0.0, sin_yaw);
        let right = Vector3::new(-sin_yaw, 0.0, cos_yaw);
        let up = Vector3::new(0.0, 1.0, 0.0);

        if let Some(rb) = rigid_body {
            // Physics-based movement
            let mut movement = forward * (self.amount_forward - self.amount_backward)
                + right * (self.amount_right - self.amount_left);

            if movement.magnitude() > 0.0 {
                movement = movement.normalize();
            }

            // Check if on ground (simple ray cast down)
            let is_grounded = rb.velocity.y.abs() < GROUND_CHECK_DISTANCE && rb.velocity.y <= 0.0;

            // Apply movement directly to velocity instead of acceleration
            let movement_factor = if is_grounded { 1.0 } else { AIR_CONTROL_FACTOR };
            let target_velocity = movement * MOVE_ACCELERATION;

            // Only modify horizontal velocity (x and z components)
            rb.velocity.x = target_velocity.x * movement_factor;
            rb.velocity.z = target_velocity.z * movement_factor;

            // Handle jumping
            if is_grounded && self.amount_up > 0.0 {
                rb.velocity.y = JUMP_FORCE;
            }

            // Cap velocity after applying changes
            rb.cap_velocity();
        } else {
            // Flying movement (original behavior)
            let movement = forward * (self.amount_forward - self.amount_backward) * self.speed * dt
                + right * (self.amount_right - self.amount_left) * self.speed * dt
                + up * (self.amount_up - self.amount_down) * self.speed * dt;

            pos3.pos += movement;
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct ViewController {
    pub sensitivity: f32,
    pub head_offset: f32,
    pub(crate) rotate_horizontal: f32,
    pub(crate) rotate_vertical: f32,
    pub(crate) yaw: cgmath::Rad<f32>,
    pub(crate) pitch: cgmath::Rad<f32>,
}

impl Default for ViewController {
    fn default() -> Self {
        Self {
            sensitivity: 0.8,
            head_offset: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
        }
    }
}

impl ViewController {
    pub fn new(sensitivity: f32, head_offset: f32) -> Self {
        Self {
            sensitivity,
            head_offset,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
        }
    }

    pub fn new_look_at<V: Into<Point3<f32>>>(
        position: V,
        target: V,
        sensitivity: f32,
        head_offset: f32,
    ) -> Self {
        let position = position.into();
        let target = target.into();
        let direction = (target - position).normalize();
        let pitch = direction.y.asin();
        let yaw = direction.z.atan2(direction.x);

        Self {
            sensitivity,
            head_offset,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            yaw: cgmath::Rad(yaw),
            pitch: cgmath::Rad(pitch),
        }
    }

    pub fn process_mouse(&mut self, dx: f64, dy: f64) {
        info!("Processing mouse motion: ({}, {})", dx, dy);
        self.rotate_horizontal += (dx as f32) * self.sensitivity;
        self.rotate_vertical += (dy as f32) * self.sensitivity;
    }

    pub fn update_rot(&mut self, pos3: &mut Pos3, dt: f32) {
        info!(
            "Updating rotation: yaw: {}, pitch: {}",
            self.yaw.0, self.pitch.0
        );
        // Rotate
        self.yaw += cgmath::Rad(self.rotate_horizontal) * dt;
        self.pitch += cgmath::Rad(-self.rotate_vertical) * dt;

        // Update the rotation quaternion
        pos3.rot = cgmath::Quaternion::from_angle_y(self.yaw)
            * cgmath::Quaternion::from_angle_x(self.pitch);

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the view will rotate
        // when moving in a non-cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if self.pitch < -cgmath::Rad(SAFE_FRAC_PI_2) {
            self.pitch = -cgmath::Rad(SAFE_FRAC_PI_2);
        } else if self.pitch > cgmath::Rad(SAFE_FRAC_PI_2) {
            self.pitch = cgmath::Rad(SAFE_FRAC_PI_2);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MovementKeycodes {
    pub forward: winit::keyboard::KeyCode,
    pub backward: winit::keyboard::KeyCode,
    pub left: winit::keyboard::KeyCode,
    pub right: winit::keyboard::KeyCode,
    pub up: winit::keyboard::KeyCode,
    pub down: winit::keyboard::KeyCode,
}

impl Default for MovementKeycodes {
    fn default() -> Self {
        Self {
            forward: winit::keyboard::KeyCode::KeyW,
            backward: winit::keyboard::KeyCode::KeyS,
            left: winit::keyboard::KeyCode::KeyA,
            right: winit::keyboard::KeyCode::KeyD,
            up: winit::keyboard::KeyCode::Space,
            down: winit::keyboard::KeyCode::ShiftLeft,
        }
    }
}
