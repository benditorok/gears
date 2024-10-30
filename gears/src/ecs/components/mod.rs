pub mod light;
pub mod model;
pub mod physics;
pub mod transform;


use super::traits::Component;

/// A component that stores the name of an object.
pub struct Name(pub &'static str);

impl Component for Name {}

/// A component that stores the camera type.
#[derive(Debug, Copy, Clone)]
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

impl Component for Camera {}

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
