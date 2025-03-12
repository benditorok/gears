use super::{
    physics::{CollisionBox, RigidBody},
    transforms::Pos3,
};
use crate::Component;
use cgmath::{InnerSpace, Point3, Rotation3, Vector3};
use gears_core::SAFE_FRAC_PI_2;
use gears_macro::Component;
use log::info;
use std::time::{Duration, Instant};
use winit::{event::ElementState, keyboard::KeyCode};

const MOVE_ACCELERATION: f32 = 15.0;
const JUMP_FORCE: f32 = 20.0;
const GROUND_CHECK_DISTANCE: f32 = 0.15;
const AIR_CONTROL_FACTOR: f32 = 0.4;
const GROUNDED_TIME_THRESHOLD: Duration = Duration::from_millis(50);
const JUMP_COOLDOWN: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq)]
enum JumpState {
    Grounded,
    Rising,
    Falling,
    JumpReleased,
}

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
    jump_state: JumpState,
    prev_jump_pressed: bool,
    // Track time-based events for more reliable jump detection
    grounded_time: Option<Instant>, // When player first contacted ground
    last_jump_time: Option<Instant>, // When player last jumped
    can_jump: bool,                 // Flag to determine if player can jump
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            speed: 20.0,
            keycodes: MovementKeycodes::default(),
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            jump_state: JumpState::Grounded,
            prev_jump_pressed: false,
            grounded_time: None,
            last_jump_time: None,
            can_jump: true,
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
            jump_state: JumpState::Grounded,
            prev_jump_pressed: false,
            grounded_time: None,
            last_jump_time: None,
            can_jump: true,
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

            // Improved jump button handling
            if state == ElementState::Released {
                // When jump key is released, track it to allow for next jump
                if self.jump_state == JumpState::Rising || self.jump_state == JumpState::Falling {
                    self.jump_state = JumpState::JumpReleased;
                }
            }
            self.prev_jump_pressed = state == ElementState::Pressed;
            true
        } else if key == self.keycodes.down {
            self.amount_down = amount;
            true
        } else {
            false
        }
    }

    fn check_ground_contact(&mut self, rb: &RigidBody<impl CollisionBox>) -> bool {
        // Consider the player grounded if:
        // 1. They have a small negative (or zero) y velocity (falling very slowly or stationary)
        // 2. AND they've been in this state for a certain amount of time
        let is_almost_stationary_vertically = rb.velocity.y > -0.5 && rb.velocity.y < 0.2;

        if is_almost_stationary_vertically {
            // Starting or continuing ground contact
            if self.grounded_time.is_none() {
                self.grounded_time = Some(Instant::now());
            }

            // Check if they've been grounded long enough
            if let Some(grounded_since) = self.grounded_time {
                if grounded_since.elapsed() >= GROUNDED_TIME_THRESHOLD {
                    return true;
                }
            }
        } else {
            // Not on ground, reset tracking
            self.grounded_time = None;
        }

        false
    }

    // Helper function to safely normalize a vector or return zero if magnitude is too small
    fn normalize_or_zero(vec: Vector3<f32>) -> Vector3<f32> {
        let mag = vec.magnitude();
        if mag > 0.0001 {
            vec / mag // Safe normalization
        } else {
            Vector3::new(0.0, 0.0, 0.0) // Return zero vector
        }
    }

    pub fn update_pos(
        &mut self,
        view_controller: &ViewController,
        pos3: &mut Pos3,
        rigid_body: Option<&mut RigidBody<impl CollisionBox>>,
        dt: f32,
    ) {
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

            // Check ground contact using the new method
            let is_grounded = self.check_ground_contact(rb);

            // Update jump state
            match self.jump_state {
                JumpState::Rising if rb.velocity.y <= 0.0 => {
                    self.jump_state = JumpState::Falling;
                }
                JumpState::Falling | JumpState::JumpReleased if is_grounded => {
                    self.jump_state = JumpState::Grounded;
                    // Reset jump cooldown when truly landing
                    self.can_jump = true;
                }
                _ => {}
            }

            // Apply movement with improved air control
            let target_velocity = movement * MOVE_ACCELERATION;

            // Apply different movement logic based on grounded state
            if is_grounded {
                // Direct control on ground
                rb.velocity.x = target_velocity.x;
                rb.velocity.z = target_velocity.z;
            } else {
                // More responsive air control by blending with current velocity
                // instead of completely replacing it
                if movement.magnitude() > 0.0 {
                    // Only apply air acceleration when there's input
                    rb.velocity.x = rb.velocity.x * (1.0 - AIR_CONTROL_FACTOR)
                        + target_velocity.x * AIR_CONTROL_FACTOR;
                    rb.velocity.z = rb.velocity.z * (1.0 - AIR_CONTROL_FACTOR)
                        + target_velocity.z * AIR_CONTROL_FACTOR;
                }

                // Add a small boost when changing direction in air for better feel
                if movement.magnitude() > 0.1 {
                    // Using our safe normalization helper
                    let current_dir =
                        Self::normalize_or_zero(Vector3::new(rb.velocity.x, 0.0, rb.velocity.z));
                    let input_dir = Self::normalize_or_zero(Vector3::new(
                        target_velocity.x,
                        0.0,
                        target_velocity.z,
                    ));

                    // If changing direction substantially, add a small directional boost
                    if current_dir.dot(input_dir) < 0.0 && current_dir.magnitude() > 0.1 {
                        rb.velocity.x += input_dir.x * 1.0; // Small directional boost
                        rb.velocity.z += input_dir.z * 1.0;
                    }
                }
            }

            // Handle jumping with cooldown to prevent repeated jumps
            let can_jump = is_grounded && self.can_jump;
            let jump_cooldown_elapsed = self
                .last_jump_time
                .is_none_or(|time| time.elapsed() >= JUMP_COOLDOWN);

            if can_jump && jump_cooldown_elapsed && self.amount_up > 0.0 && self.prev_jump_pressed {
                rb.velocity.y = JUMP_FORCE;
                self.jump_state = JumpState::Rising;
                self.last_jump_time = Some(Instant::now());
                self.grounded_time = None;
                self.can_jump = false; // Require key release before next jump
            }

            // Cap velocity after applying changes
            rb.cap_velocity();
        } else {
            // Flying movement (unchanged)
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
    pub rotate_horizontal: f32,
    pub rotate_vertical: f32,
    pub yaw: cgmath::Rad<f32>,
    pub pitch: cgmath::Rad<f32>,
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

    pub fn process_mouse(&mut self, dx: f64, dy: f64, dt: f32) {
        info!("Processing mouse motion: ({}, {})", dx, dy);
        self.rotate_horizontal = (dx as f32) * self.sensitivity;
        self.rotate_vertical = (dy as f32) * self.sensitivity;
    }

    pub fn update_rot(&mut self, pos3: &mut Pos3, dt: f32) {
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
