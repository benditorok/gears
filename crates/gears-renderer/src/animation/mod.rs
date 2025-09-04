//! Animation system
//!
//! This module supports:
//! - Animation states and transitions
//! - Animation blending and mixing
//! - Multiple interpolation modes
//! - Animation events and callbacks
//! - Layered animation support
//! - Complex timing controls

pub mod clip;
pub mod controller;
pub mod mixer;
pub mod state;
pub mod timeline;
pub mod track;

pub use clip::*;
pub use controller::{AnimationController, TransitionSettings};
pub use mixer::*;
pub use state::{
    AnimationStateMachine, ParameterCondition, StateParameters, StateTransition,
    TransitionCondition,
};
pub use timeline::*;
pub use track::*;

use cgmath::{Quaternion, Vector3};
use gears_ecs::Component;
use gears_macro::Component;
use std::time::{Duration, Instant};

/// The target of an animation track (what property is being animated)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnimationTarget {
    /// Position/translation of the object
    Translation,
    /// Rotation of the object
    Rotation,
    /// Scale of the object
    Scale,
    /// Custom property with string identifier
    Custom(String),
}

/// Different interpolation modes for animation keyframes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterpolationMode {
    /// Linear interpolation between keyframes
    Linear,
    /// Step interpolation (no smoothing)
    Step,
    /// Cubic spline interpolation
    CubicSpline,
    /// Custom interpolation function
    Custom,
}

/// Animation loop modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoopMode {
    /// Play once and stop
    Once,
    /// Loop infinitely
    Repeat,
    /// Loop a specific number of times
    RepeatCount(u32),
    /// Ping-pong (forward then backward)
    PingPong,
}

/// Animation playback state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
    Finished,
}

/// Animation data types that can be interpolated
#[derive(Debug, Clone)]
pub enum AnimationValue {
    Float(f32),
    Vector3(Vector3<f32>),
    Quaternion(Quaternion<f32>),
    FloatArray(Vec<f32>),
}

impl AnimationValue {
    /// Linear interpolation between two animation values
    pub fn lerp(&self, other: &Self, t: f32) -> Option<Self> {
        match (self, other) {
            (AnimationValue::Float(a), AnimationValue::Float(b)) => {
                Some(AnimationValue::Float(a + (b - a) * t))
            }
            (AnimationValue::Vector3(a), AnimationValue::Vector3(b)) => {
                Some(AnimationValue::Vector3(a + (b - a) * t))
            }
            (AnimationValue::Quaternion(a), AnimationValue::Quaternion(b)) => {
                Some(AnimationValue::Quaternion(a.slerp(*b, t)))
            }
            (AnimationValue::FloatArray(a), AnimationValue::FloatArray(b)) => {
                if a.len() == b.len() {
                    let result: Vec<f32> = a
                        .iter()
                        .zip(b.iter())
                        .map(|(x, y)| x + (y - x) * t)
                        .collect();
                    Some(AnimationValue::FloatArray(result))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get the value as a Vector3 if possible
    pub fn as_vector3(&self) -> Option<Vector3<f32>> {
        match self {
            AnimationValue::Vector3(v) => Some(*v),
            AnimationValue::FloatArray(arr) if arr.len() >= 3 => {
                Some(Vector3::new(arr[0], arr[1], arr[2]))
            }
            _ => None,
        }
    }

    /// Get the value as a Quaternion if possible
    pub fn as_quaternion(&self) -> Option<Quaternion<f32>> {
        match self {
            AnimationValue::Quaternion(q) => Some(*q),
            AnimationValue::FloatArray(arr) if arr.len() >= 4 => {
                Some(Quaternion::new(arr[3], arr[0], arr[1], arr[2])) // w, x, y, z
            }
            _ => None,
        }
    }

    /// Get the value as a float if possible
    pub fn as_float(&self) -> Option<f32> {
        match self {
            AnimationValue::Float(f) => Some(*f),
            _ => None,
        }
    }
}

/// A keyframe in an animation with timestamp and value
#[derive(Debug, Clone)]
pub struct Keyframe {
    /// Time of this keyframe in seconds
    pub time: f32,
    /// The animated value at this time
    pub value: AnimationValue,
    /// Interpolation mode to use when transitioning from this keyframe
    pub interpolation: InterpolationMode,
}

impl Keyframe {
    pub fn new(time: f32, value: AnimationValue) -> Self {
        Self {
            time,
            value,
            interpolation: InterpolationMode::Linear,
        }
    }

    pub fn with_interpolation(mut self, interpolation: InterpolationMode) -> Self {
        self.interpolation = interpolation;
        self
    }
}

/// An animation event that can be triggered at a specific time
#[derive(Debug, Clone)]
pub struct AnimationEvent {
    /// Time when this event should trigger
    pub time: f32,
    /// Event identifier
    pub name: String,
    /// Optional event data
    pub data: Option<String>,
}

impl AnimationEvent {
    pub fn new(time: f32, name: impl Into<String>) -> Self {
        Self {
            time,
            name: name.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }
}

/// Animation metrics and timing information
#[derive(Debug, Clone)]
pub struct AnimationMetrics {
    /// Current playback time
    pub current_time: f32,
    /// Total duration of the animation
    pub duration: f32,
    /// Current playback speed multiplier
    pub speed: f32,
    /// Current loop iteration (for looped animations)
    pub loop_count: u32,
    /// Time when animation was started
    pub start_time: Instant,
    /// Time when animation was last updated
    pub last_update: Instant,
}

impl Default for AnimationMetrics {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            current_time: 0.0,
            duration: 0.0,
            speed: 1.0,
            loop_count: 0,
            start_time: now,
            last_update: now,
        }
    }
}

impl AnimationMetrics {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            ..Default::default()
        }
    }

    /// Update the metrics with the given delta time
    pub fn update(&mut self, dt: Duration) {
        self.last_update = Instant::now();
        self.current_time += dt.as_secs_f32() * self.speed;
    }

    /// Get the normalized time (0.0 to 1.0) of the animation
    pub fn normalized_time(&self) -> f32 {
        if self.duration > 0.0 {
            (self.current_time / self.duration).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Check if the animation has finished (for non-looping animations)
    pub fn is_finished(&self) -> bool {
        self.current_time >= self.duration
    }

    /// Reset the animation to the beginning
    pub fn reset(&mut self) {
        self.current_time = 0.0;
        self.loop_count = 0;
        self.start_time = Instant::now();
        self.last_update = self.start_time;
    }

    /// Set the playback speed
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.max(0.0);
    }
}

/// Component that holds animation data for an entity
#[derive(Component, Debug)]
pub struct AnimationComponent {
    /// The animation controller managing this entity's animations
    pub controller: AnimationController,
    /// Whether animations are currently enabled
    pub enabled: bool,
    /// Global animation speed multiplier for this entity
    pub global_speed: f32,
}

impl Default for AnimationComponent {
    fn default() -> Self {
        Self {
            controller: AnimationController::new(),
            enabled: true,
            global_speed: 1.0,
        }
    }
}

impl AnimationComponent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_controller(controller: AnimationController) -> Self {
        Self {
            controller,
            enabled: true,
            global_speed: 1.0,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_global_speed(&mut self, speed: f32) {
        self.global_speed = speed.max(0.0);
    }

    /// Update the animation component with the given delta time
    pub fn update(&mut self, dt: Duration) -> Vec<AnimationEvent> {
        if !self.enabled {
            return Vec::new();
        }

        let adjusted_dt = Duration::from_secs_f32(dt.as_secs_f32() * self.global_speed);
        self.controller.update(adjusted_dt)
    }

    /// Play an animation by name
    pub fn play(&mut self, name: &str) -> Result<(), String> {
        self.controller.play(name)
    }

    /// Stop the currently playing animation
    pub fn stop(&mut self) {
        self.controller.stop();
    }

    /// Pause the currently playing animation
    pub fn pause(&mut self) {
        self.controller.pause();
    }

    /// Resume a paused animation
    pub fn resume(&mut self) {
        self.controller.resume();
    }

    /// Add an animation clip to the controller
    pub fn add_clip(&mut self, clip: AnimationClip) {
        self.controller.add_clip(clip);
    }

    /// Get the current playback state
    pub fn playback_state(&self) -> PlaybackState {
        self.controller.current_state()
    }
}
