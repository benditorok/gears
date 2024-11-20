use std::time;

use crate::{ecs::traits::Tick, prelude::Component, SAFE_FRAC_PI_2};
use cgmath::{InnerSpace, Rotation3};
use gears_macro::Component;
use winit::{event::ElementState, keyboard::KeyCode};

use super::transforms::Pos3;

#[derive(Component, Debug, Clone)]
pub struct MovementController {
    pub speed: f32,
    pub keycodes: MovementKeycodes,
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
            speed: 5.0,
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
    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
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

    pub fn update_pos(&self, pos3: &mut Pos3, dt: f32) {
        pos3.pos.x += (self.amount_right - self.amount_left) * self.speed * dt;
        pos3.pos.y += (self.amount_up - self.amount_down) * self.speed * dt;
        pos3.pos.z += (self.amount_forward - self.amount_backward) * self.speed * dt;
    }
}

#[derive(Component, Debug, Clone)]
pub struct ViewController {
    pub sensitivity: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    yaw: cgmath::Rad<f32>,
    pitch: cgmath::Rad<f32>,
}

impl Default for ViewController {
    fn default() -> Self {
        Self {
            sensitivity: 1.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
        }
    }
}

impl ViewController {
    pub fn process_mouse(&mut self, dx: f32, dy: f32) {
        self.rotate_horizontal += dx * self.sensitivity;
        self.rotate_vertical += dy * self.sensitivity;
    }

    pub fn update_rot(&mut self, pos3: &mut Pos3, dt: f32) {
        // Rotate
        self.yaw += cgmath::Rad(self.rotate_horizontal) * self.sensitivity * dt;
        self.pitch += cgmath::Rad(-self.rotate_vertical) * self.sensitivity * dt;

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
