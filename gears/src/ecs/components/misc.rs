use crate::ecs::traits::{Component, Marker};
use gears_macro::Component;

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerMarker;

impl Marker for PlayerMarker {
    fn describe() -> &'static str {
        "Required components: Pos3, ModelSource, MovementController, ViewController"
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct DynamicCameraMarker;

impl Marker for DynamicCameraMarker {
    fn describe() -> &'static str {
        "Required components: Camera, Pos3, MovementController, ViewController"
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct StaticCameraMarker;

impl Marker for StaticCameraMarker {
    fn describe() -> &'static str {
        "Required components: Camera, Pos3"
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

/// A component that stores the name of an object.ű
#[derive(Component, Debug, Clone, Copy)]
pub struct Name(pub &'static str);

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
