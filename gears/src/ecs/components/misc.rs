use std::default;

use crate::ecs::traits::Component;
use gears_macro::Component;

#[derive(Component, Debug, Clone, Copy)]
pub enum Marker {
    Player,
    StaticCamera,
    DynamicCamera,
    RigidBody,
    Light,
}
// TODO include the name &str in the Marker enum

impl Marker {
    pub fn requirements(&self) -> &'static str {
        match self {
            Marker::Player => {
                "Required components: Pos3, ModelSource, MovementController, ViewController"
            }
            Marker::StaticCamera => "Required components: Camera, Pos3",
            Marker::DynamicCamera => {
                "Required components: Camera, Pos3, MovementController, ViewController"
            }
            Marker::RigidBody => "Required components: Pos3, RigidBody, ModelSource",
            Marker::Light => "Required components: Pos3, Light",
        }
    }
}

/// A component that stores the name of an object.ű
#[derive(Component, Debug, Clone)]
pub struct Name(pub &'static str);

/// A component that stores the camera type.
#[derive(Component, Debug, Clone)]
pub struct Camera {
    pub look_at: cgmath::Point3<f32>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            look_at: cgmath::Point3::new(0.0, 0.0, 0.0),
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
