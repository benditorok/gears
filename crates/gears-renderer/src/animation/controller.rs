//! Animation controller for managing animation states, transitions, and playback.

use super::{AnimationClip, AnimationEvent, AnimationMetrics, LoopMode, PlaybackState};
use std::collections::HashMap;
use std::time::Duration;

/// The animation controller manages multiple animation clips and their playback
#[derive(Debug)]
pub struct AnimationController {
    /// All available animation clips indexed by name
    clips: HashMap<String, AnimationClip>,
    /// Currently active animation states
    active_states: Vec<AnimationState>,
    /// Global playback speed multiplier
    global_speed: f32,
    /// Whether the controller is paused
    paused: bool,
    /// Transition settings
    transition_settings: TransitionSettings,
}

/// Represents an active animation state
#[derive(Debug, Clone)]
pub struct AnimationState {
    /// Name of the animation clip being played
    pub clip_name: String,
    /// Playback metrics for this animation
    pub metrics: AnimationMetrics,
    /// Current playback state
    pub state: PlaybackState,
    /// Weight for blending (0.0 to 1.0)
    pub weight: f32,
    /// Layer this animation plays on (higher numbers override lower)
    pub layer: i32,
    /// Whether this animation should loop
    pub loop_mode: LoopMode,
    /// Fade in/out progress for smooth transitions
    pub fade_progress: f32,
    /// Target weight for transitions
    pub target_weight: f32,
    /// Transition duration
    pub transition_duration: f32,
    /// Time since transition started
    pub transition_time: f32,
}

impl AnimationState {
    pub fn new(clip_name: String, clip: &AnimationClip) -> Self {
        Self {
            clip_name,
            metrics: AnimationMetrics::new(clip.duration),
            state: PlaybackState::Playing,
            weight: 1.0,
            layer: 0,
            loop_mode: clip.loop_mode,
            fade_progress: 1.0,
            target_weight: 1.0,
            transition_duration: 0.0,
            transition_time: 0.0,
        }
    }

    /// Update this animation state
    pub fn update(&mut self, dt: Duration) -> bool {
        if self.state != PlaybackState::Playing {
            return false;
        }

        // Update animation timing
        self.metrics.update(dt);

        // Handle looping
        match self.loop_mode {
            LoopMode::Once => {
                if self.metrics.is_finished() {
                    self.state = PlaybackState::Finished;
                    return false;
                }
            }
            LoopMode::Repeat => {
                if self.metrics.current_time >= self.metrics.duration {
                    self.metrics.current_time = 0.0;
                    self.metrics.loop_count += 1;
                }
            }
            LoopMode::RepeatCount(count) => {
                if self.metrics.current_time >= self.metrics.duration {
                    self.metrics.loop_count += 1;
                    if self.metrics.loop_count >= count {
                        self.state = PlaybackState::Finished;
                        return false;
                    } else {
                        self.metrics.current_time = 0.0;
                    }
                }
            }
            LoopMode::PingPong => {
                if self.metrics.current_time >= self.metrics.duration {
                    // Reverse playback speed for ping-pong
                    self.metrics.speed = -self.metrics.speed;
                    self.metrics.current_time = self.metrics.duration;
                }
                if self.metrics.current_time <= 0.0 && self.metrics.speed < 0.0 {
                    self.metrics.speed = -self.metrics.speed;
                    self.metrics.current_time = 0.0;
                }
            }
        }

        // Update transition/fade
        if self.transition_duration > 0.0 {
            self.transition_time += dt.as_secs_f32();
            let progress = (self.transition_time / self.transition_duration).min(1.0);

            // Smooth interpolation for weight changes
            self.weight = self.weight + (self.target_weight - self.weight) * progress;

            if progress >= 1.0 {
                self.transition_duration = 0.0;
                self.transition_time = 0.0;
                self.weight = self.target_weight;
            }
        }

        true
    }

    /// Start a transition to a new weight
    pub fn transition_to_weight(&mut self, target_weight: f32, duration: f32) {
        self.target_weight = target_weight.clamp(0.0, 1.0);
        self.transition_duration = duration.max(0.0);
        self.transition_time = 0.0;
    }

    /// Check if this state is active (weight > 0 and not finished)
    pub fn is_active(&self) -> bool {
        self.weight > 0.0 && self.state != PlaybackState::Finished
    }
}

/// Settings for animation transitions
#[derive(Debug, Clone)]
pub struct TransitionSettings {
    /// Default transition duration when switching animations
    pub default_transition_duration: f32,
    /// Whether to use crossfading for transitions
    pub use_crossfade: bool,
    /// Maximum number of simultaneously playing animations
    pub max_active_animations: usize,
}

impl Default for TransitionSettings {
    fn default() -> Self {
        Self {
            default_transition_duration: 0.2, // 200ms default transition
            use_crossfade: true,
            max_active_animations: 4,
        }
    }
}

impl AnimationController {
    /// Create a new animation controller
    pub fn new() -> Self {
        Self {
            clips: HashMap::new(),
            active_states: Vec::new(),
            global_speed: 1.0,
            paused: false,
            transition_settings: TransitionSettings::default(),
        }
    }

    /// Add an animation clip to the controller
    pub fn add_clip(&mut self, clip: AnimationClip) {
        self.clips.insert(clip.name.clone(), clip);
    }

    /// Remove an animation clip from the controller
    pub fn remove_clip(&mut self, name: &str) -> Option<AnimationClip> {
        // Stop any active animations using this clip
        self.active_states.retain(|state| state.clip_name != name);
        self.clips.remove(name)
    }

    /// Get a reference to an animation clip
    pub fn get_clip(&self, name: &str) -> Option<&AnimationClip> {
        self.clips.get(name)
    }

    /// Get a mutable reference to an animation clip
    pub fn get_clip_mut(&mut self, name: &str) -> Option<&mut AnimationClip> {
        self.clips.get_mut(name)
    }

    /// Play an animation by name
    pub fn play(&mut self, name: &str) -> Result<(), String> {
        let clip = self
            .clips
            .get(name)
            .ok_or_else(|| format!("Animation clip '{}' not found", name))?;

        // Check if this animation is already playing
        if let Some(existing_state) = self.active_states.iter_mut().find(|s| s.clip_name == name) {
            existing_state.state = PlaybackState::Playing;
            existing_state.metrics.reset();
            return Ok(());
        }

        // Create new animation state
        let mut new_state = AnimationState::new(name.to_string(), clip);

        // Handle transitions if crossfade is enabled
        if self.transition_settings.use_crossfade && !self.active_states.is_empty() {
            // Fade out existing animations
            for state in &mut self.active_states {
                if state.is_active() {
                    state.transition_to_weight(
                        0.0,
                        self.transition_settings.default_transition_duration,
                    );
                }
            }

            // Fade in new animation
            new_state.weight = 0.0;
            new_state
                .transition_to_weight(1.0, self.transition_settings.default_transition_duration);
        }

        self.active_states.push(new_state);

        // Limit the number of active animations
        if self.active_states.len() > self.transition_settings.max_active_animations {
            self.active_states.sort_by(|a, b| {
                a.weight
                    .partial_cmp(&b.weight)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.active_states
                .truncate(self.transition_settings.max_active_animations);
        }

        Ok(())
    }

    /// Play an animation with custom settings
    pub fn play_with_settings(
        &mut self,
        name: &str,
        loop_mode: LoopMode,
        layer: i32,
        transition_duration: Option<f32>,
    ) -> Result<(), String> {
        let clip = self
            .clips
            .get(name)
            .ok_or_else(|| format!("Animation clip '{}' not found", name))?;

        let mut new_state = AnimationState::new(name.to_string(), clip);
        new_state.loop_mode = loop_mode;
        new_state.layer = layer;

        let transition_dur =
            transition_duration.unwrap_or(self.transition_settings.default_transition_duration);

        // Handle transitions for animations on the same layer
        if self.transition_settings.use_crossfade {
            for state in &mut self.active_states {
                if state.layer == layer && state.is_active() {
                    state.transition_to_weight(0.0, transition_dur);
                }
            }

            new_state.weight = 0.0;
            new_state.transition_to_weight(1.0, transition_dur);
        }

        self.active_states.push(new_state);
        Ok(())
    }

    /// Stop all animations
    pub fn stop(&mut self) {
        self.active_states.clear();
    }

    /// Stop a specific animation
    pub fn stop_animation(&mut self, name: &str) {
        self.active_states.retain(|state| state.clip_name != name);
    }

    /// Pause all animations
    pub fn pause(&mut self) {
        self.paused = true;
        for state in &mut self.active_states {
            if state.state == PlaybackState::Playing {
                state.state = PlaybackState::Paused;
            }
        }
    }

    /// Resume all paused animations
    pub fn resume(&mut self) {
        self.paused = false;
        for state in &mut self.active_states {
            if state.state == PlaybackState::Paused {
                state.state = PlaybackState::Playing;
            }
        }
    }

    /// Set global playback speed
    pub fn set_global_speed(&mut self, speed: f32) {
        self.global_speed = speed.max(0.0);
    }

    /// Get current playback state (returns the primary animation's state)
    pub fn current_state(&self) -> PlaybackState {
        self.active_states
            .iter()
            .max_by(|a, b| {
                a.weight
                    .partial_cmp(&b.weight)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|state| state.state)
            .unwrap_or(PlaybackState::Stopped)
    }

    /// Update the animation controller
    pub fn update(&mut self, dt: Duration) -> Vec<AnimationEvent> {
        if self.paused {
            return Vec::new();
        }

        let adjusted_dt = Duration::from_secs_f32(dt.as_secs_f32() * self.global_speed);
        let mut events = Vec::new();

        // Update all active animation states
        let mut states_to_remove = Vec::new();
        for (index, state) in self.active_states.iter_mut().enumerate() {
            let was_active = state.update(adjusted_dt);

            if !was_active || state.weight <= 0.001 {
                states_to_remove.push(index);
            } else if let Some(clip) = self.clips.get(&state.clip_name) {
                // Collect events from this animation
                let prev_time = state.metrics.current_time - adjusted_dt.as_secs_f32();
                let current_time = state.metrics.current_time;

                for event in clip.get_events_in_range(prev_time, current_time) {
                    events.push(event.clone());
                }
            }
        }

        // Remove finished/inactive states
        for &index in states_to_remove.iter().rev() {
            self.active_states.remove(index);
        }

        events
    }

    /// Get the current animation sample for all active animations
    pub fn sample(
        &self,
        time_override: Option<f32>,
    ) -> HashMap<super::AnimationTarget, super::AnimationValue> {
        let mut result: HashMap<super::AnimationTarget, super::AnimationValue> = HashMap::new();
        let mut weights_per_target: HashMap<super::AnimationTarget, f32> = HashMap::new();

        // Sample all active animations and blend them
        for state in &self.active_states {
            if !state.is_active() {
                continue;
            }

            if let Some(clip) = self.clips.get(&state.clip_name) {
                let sample_time = time_override.unwrap_or(state.metrics.current_time);
                let (loop_time, _) = clip.calculate_loop_time(sample_time);
                let clip_sample = clip.sample(loop_time);

                for (target, value) in clip_sample {
                    let weighted_value = if let Some(existing) = result.get(&target) {
                        // Blend with existing value
                        let existing_weight =
                            weights_per_target.get(&target).copied().unwrap_or(0.0);
                        let total_weight = existing_weight + state.weight;

                        if total_weight > 0.0 {
                            let blend_factor = state.weight / total_weight;
                            existing.lerp(&value, blend_factor).unwrap_or(value)
                        } else {
                            value
                        }
                    } else {
                        value
                    };

                    result.insert(target.clone(), weighted_value);
                    weights_per_target.insert(
                        target.clone(),
                        weights_per_target.get(&target).unwrap_or(&0.0) + state.weight,
                    );
                }
            }
        }

        result
    }

    /// Get all active animation names
    pub fn get_active_animations(&self) -> Vec<String> {
        self.active_states
            .iter()
            .filter(|state| state.is_active())
            .map(|state| state.clip_name.clone())
            .collect()
    }

    /// Check if a specific animation is playing
    pub fn is_playing(&self, name: &str) -> bool {
        self.active_states
            .iter()
            .any(|state| state.clip_name == name && state.state == PlaybackState::Playing)
    }

    /// Get the weight of a specific animation
    pub fn get_animation_weight(&self, name: &str) -> Option<f32> {
        self.active_states
            .iter()
            .find(|state| state.clip_name == name)
            .map(|state| state.weight)
    }

    /// Set the weight of a specific animation
    pub fn set_animation_weight(&mut self, name: &str, weight: f32) -> Result<(), String> {
        if let Some(state) = self
            .active_states
            .iter_mut()
            .find(|state| state.clip_name == name)
        {
            state.weight = weight.clamp(0.0, 1.0);
            Ok(())
        } else {
            Err(format!("Animation '{}' is not currently active", name))
        }
    }

    /// Configure transition settings
    pub fn set_transition_settings(&mut self, settings: TransitionSettings) {
        self.transition_settings = settings;
    }

    /// Get the number of loaded clips
    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }

    /// Get the number of active animations
    pub fn active_animation_count(&self) -> usize {
        self.active_states.len()
    }

    /// Get all available animation names
    pub fn get_animation_names(&self) -> Vec<String> {
        self.clips.keys().cloned().collect()
    }
}

impl Default for AnimationController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::{
        AnimationClip, AnimationTarget, AnimationTrack, AnimationValue, Keyframe,
    };

    #[test]
    fn test_controller_creation() {
        let controller = AnimationController::new();
        assert_eq!(controller.clip_count(), 0);
        assert_eq!(controller.active_animation_count(), 0);
    }

    #[test]
    fn test_add_and_play_clip() {
        let mut controller = AnimationController::new();

        let mut clip = AnimationClip::new("test_clip");
        clip.duration = 2.0;

        controller.add_clip(clip);
        assert_eq!(controller.clip_count(), 1);

        controller.play("test_clip").unwrap();
        assert_eq!(controller.active_animation_count(), 1);
        assert!(controller.is_playing("test_clip"));
    }

    #[test]
    fn test_animation_update() {
        let mut controller = AnimationController::new();

        let mut clip = AnimationClip::new("test_clip");
        clip.duration = 1.0;
        clip.loop_mode = LoopMode::Once;

        controller.add_clip(clip);
        controller.play("test_clip").unwrap();

        // Update animation
        controller.update(Duration::from_millis(500));
        assert!(controller.is_playing("test_clip"));

        // Finish animation
        controller.update(Duration::from_millis(600));
        assert!(!controller.is_playing("test_clip"));
    }
}
