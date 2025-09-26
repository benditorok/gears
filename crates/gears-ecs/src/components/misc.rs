use crate::Component;
use core::time;
use gears_macro::Component;
use std::{ops::Deref, time::Instant};

pub trait Tick {
    fn on_tick(&mut self, delta_time: time::Duration);
}

pub trait Marker {
    fn describe() -> &'static str;
}

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

/// Enemy marker for entities that track the player
#[derive(Component, Debug, Clone, Copy)]
pub struct EnemyMarker;

impl Marker for EnemyMarker {
    fn describe() -> &'static str {
        "Required components: Pos3, ModelSource, MovementController, ViewController, PathfindingComponent, PathfindingFollower"
    }
}

/// Marker for obstacles that should be avoided during pathfinding
#[derive(Component, Debug, Clone, Copy)]
pub struct ObstacleMarker;

impl Marker for ObstacleMarker {
    fn describe() -> &'static str {
        "Required components: Pos3, AABBCollisionBox, RigidBody"
    }
}

/// A component that stores the name of an object.
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

    pub fn get_health(&self) -> f32 {
        self.health
    }

    pub fn get_max_health(&self) -> f32 {
        self.max_health
    }

    pub fn set_health(&mut self, health: f32) {
        self.health = health.clamp(0.0, self.max_health);
    }
}

#[derive(Component, Debug, Clone)]
pub struct AnimationQueue {
    pub time: Instant,
    animations: Vec<String>,
    pub is_current_finished: bool,
    pub current_animation: Option<String>,
    pub transition_duration: f32,
    pub auto_transition: bool,
}

impl Default for AnimationQueue {
    fn default() -> Self {
        Self {
            time: Instant::now(),
            animations: Vec::new(),
            is_current_finished: false,
            current_animation: None,
            transition_duration: 0.2,
            auto_transition: true,
        }
    }
}

impl AnimationQueue {
    pub fn new(animations: Vec<String>) -> Self {
        Self {
            time: Instant::now(),
            animations,
            is_current_finished: false,
            current_animation: None,
            transition_duration: 0.2,
            auto_transition: true,
        }
    }

    pub fn push(&mut self, animation: impl Into<String>) {
        let animation_name = animation.into();
        if !self.animations.contains(&animation_name) {
            self.animations.push(animation_name);
        }
    }

    pub fn pop(&mut self) -> Option<String> {
        self.animations.pop()
    }

    pub fn play_animation(&mut self, animation: impl Into<String>) {
        self.current_animation = Some(animation.into());
        self.time = Instant::now();
        self.is_current_finished = false;
    }

    pub fn stop_current(&mut self) {
        self.current_animation = None;
        self.is_current_finished = true;
    }

    pub fn has_queued_animations(&self) -> bool {
        !self.animations.is_empty()
    }

    pub fn current_animation(&self) -> Option<&String> {
        self.current_animation.as_ref()
    }

    pub fn set_transition_duration(&mut self, duration: f32) {
        self.transition_duration = duration.max(0.0);
    }

    pub fn set_auto_transition(&mut self, auto: bool) {
        self.auto_transition = auto;
    }

    pub fn play_next(&mut self) -> Option<String> {
        if let Some(next_animation) = self.pop() {
            self.play_animation(next_animation.clone());
            Some(next_animation)
        } else {
            None
        }
    }

    pub fn clear_queue(&mut self) {
        self.animations.clear();
    }

    pub fn queue_length(&self) -> usize {
        self.animations.len()
    }
}
