use crate::ecs::{components::Marker, Component};
use gears_macro::Component;
use std::ops::Deref;

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerMarker;

impl Marker for PlayerMarker {
    fn describe() -> &'static str {
        "Required components: Pos3, ModelSource, MovementController, ViewController"
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct CameraMarker;

impl Marker for CameraMarker {
    fn describe() -> &'static str {
        "Required components: Pos3, ViewController"
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct RigidBodyMarker;

impl Marker for RigidBodyMarker {
    fn describe() -> &'static str {
        "Required components: Pos3, RigidBody, ModelSource"
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct LightMarker;

impl Marker for LightMarker {
    fn describe() -> &'static str {
        "Required components: Pos3, Light"
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct StaticModelMarker;

impl Marker for StaticModelMarker {
    fn describe() -> &'static str {
        "Required components: Name, Pos3, ModelSource"
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct TargetMarker;

impl Marker for TargetMarker {
    fn describe() -> &'static str {
        "Required components: Pos3, RigidBody, ModelSource, Name, Health"
    }
}

/// A component that stores the name of an object.ű
#[derive(Component, Debug, Clone, Copy)]
pub struct Name(pub &'static str);

impl Deref for Name {
    type Target = &'static str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub(crate) health: f32,
    pub(crate) max_health: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            health: 100.0,
            max_health: 100.0,
        }
    }
}

impl Health {
    pub fn new(health: f32, max_health: f32) -> Self {
        Self { health, max_health }
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
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
