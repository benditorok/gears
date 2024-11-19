pub mod light;
pub mod model;
pub mod physics;
pub mod prefabs;
pub mod transform;

use super::traits::Component;
use gears_macro::Component;

/// A component that stores the name of an object.ű
#[derive(Component, Debug, Clone)]
pub struct Name(pub &'static str);

/// A component that stores the camera type.
#[derive(Component, Debug, Copy, Clone)]
pub enum Camera {
    FPS {
        look_at: cgmath::Point3<f32>,
        speed: f32,
        sensitivity: f32,
        keycodes: CameraKeycodes,
    },
    Fixed {
        look_at: cgmath::Point3<f32>,
    },
}

#[derive(Debug, Copy, Clone)]
pub struct CameraKeycodes {
    pub forward: winit::keyboard::KeyCode,
    pub backward: winit::keyboard::KeyCode,
    pub left: winit::keyboard::KeyCode,
    pub right: winit::keyboard::KeyCode,
    pub up: winit::keyboard::KeyCode,
    pub down: winit::keyboard::KeyCode,
}

impl Default for CameraKeycodes {
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

#[derive(Component, Debug, Clone, Default)]
pub struct AnimationQueue {
    animations: Vec<&'static str>,
    pub(crate) is_current_finished: bool,
}

impl AnimationQueue {
    pub fn new(animations: Vec<&'static str>) -> Self {
        Self {
            animations,
            is_current_finished: false,
        }
    }

    pub fn push(&mut self, animation: &'static str) {
        if !self.animations.contains(&animation) {
            self.animations.push(animation);
        }
    }

    pub fn pop(&mut self) -> Option<&'static str> {
        self.animations.pop()
    }
}
