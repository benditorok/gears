//! Animation state management for creating animation state machines.

use super::{AnimationEvent, LoopMode, PlaybackState};
use std::collections::HashMap;
use std::time::Duration;

/// Represents a condition that can trigger a state transition
#[derive(Debug, Clone)]
pub enum TransitionCondition {
    /// Transition when animation finishes
    OnAnimationEnd,
    /// Transition after a specific time
    OnTime(f32),
    /// Transition when a parameter meets a condition
    OnParameter(String, ParameterCondition),
    /// Transition when an event is triggered
    OnEvent(String),
    /// Custom transition condition
    Custom(String),
    /// Immediate transition (useful for testing)
    Immediate,
}

/// Parameter-based transition conditions
#[derive(Debug, Clone)]
pub enum ParameterCondition {
    /// Float parameter equals value (with tolerance)
    FloatEquals(f32, f32),
    /// Float parameter is greater than value
    FloatGreater(f32),
    /// Float parameter is less than value
    FloatLess(f32),
    /// Boolean parameter equals value
    BoolEquals(bool),
    /// Integer parameter equals value
    IntEquals(i32),
    /// Integer parameter is greater than value
    IntGreater(i32),
    /// Integer parameter is less than value
    IntLess(i32),
}

/// A transition between animation states
#[derive(Debug, Clone)]
pub struct StateTransition {
    /// Target state to transition to
    pub target_state: String,
    /// Condition that triggers this transition
    pub condition: TransitionCondition,
    /// Duration of the transition (for blending)
    pub transition_duration: f32,
    /// Priority of this transition (higher values take precedence)
    pub priority: i32,
    /// Whether this transition can interrupt other transitions
    pub can_interrupt: bool,
    /// Whether this transition should reset the target animation
    pub reset_target: bool,
}

impl StateTransition {
    /// Create a new state transition
    pub fn new(target_state: String, condition: TransitionCondition) -> Self {
        Self {
            target_state,
            condition,
            transition_duration: 0.2, // Default 200ms transition
            priority: 0,
            can_interrupt: false,
            reset_target: true,
        }
    }

    /// Set the transition duration
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.transition_duration = duration.max(0.0);
        self
    }

    /// Set the transition priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set whether this transition can interrupt others
    pub fn with_interrupt(mut self, can_interrupt: bool) -> Self {
        self.can_interrupt = can_interrupt;
        self
    }

    /// Set whether to reset the target animation
    pub fn with_reset_target(mut self, reset_target: bool) -> Self {
        self.reset_target = reset_target;
        self
    }

    /// Check if this transition should trigger given the current state
    pub fn should_trigger(
        &self,
        current_time: f32,
        animation_state: PlaybackState,
        parameters: &StateParameters,
        triggered_events: &[String],
    ) -> bool {
        match &self.condition {
            TransitionCondition::OnAnimationEnd => animation_state == PlaybackState::Finished,
            TransitionCondition::OnTime(time) => current_time >= *time,
            TransitionCondition::OnParameter(param_name, condition) => {
                parameters.check_condition(param_name, condition)
            }
            TransitionCondition::OnEvent(event_name) => triggered_events.contains(event_name),
            TransitionCondition::Immediate => true,
            TransitionCondition::Custom(_) => false, // Custom conditions handled externally
        }
    }
}

/// A state in the animation state machine
#[derive(Debug, Clone)]
pub struct AnimationState {
    /// Unique name of this state
    pub name: String,
    /// Animation clip to play in this state
    pub animation_clip: String,
    /// Loop mode for this state
    pub loop_mode: LoopMode,
    /// Playback speed multiplier
    pub speed: f32,
    /// Transitions from this state to other states
    pub transitions: Vec<StateTransition>,
    /// Events that this state can trigger
    pub events: Vec<AnimationEvent>,
    /// Whether this state is a default/entry state
    pub is_entry_state: bool,
    /// Layer this state operates on
    pub layer: i32,
    /// Weight of this state (for blending)
    pub weight: f32,
}

impl AnimationState {
    /// Create a new animation state
    pub fn new(name: String, animation_clip: String) -> Self {
        Self {
            name,
            animation_clip,
            loop_mode: LoopMode::Once,
            speed: 1.0,
            transitions: Vec::new(),
            events: Vec::new(),
            is_entry_state: false,
            layer: 0,
            weight: 1.0,
        }
    }

    /// Set the loop mode for this state
    pub fn with_loop_mode(mut self, loop_mode: LoopMode) -> Self {
        self.loop_mode = loop_mode;
        self
    }

    /// Set the playback speed for this state
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed.max(0.0);
        self
    }

    /// Add a transition from this state
    pub fn add_transition(mut self, transition: StateTransition) -> Self {
        // Insert transition in priority order (higher priority first)
        let insert_pos = self
            .transitions
            .iter()
            .position(|t| t.priority < transition.priority)
            .unwrap_or(self.transitions.len());

        self.transitions.insert(insert_pos, transition);
        self
    }

    /// Mark this state as an entry state
    pub fn as_entry_state(mut self) -> Self {
        self.is_entry_state = true;
        self
    }

    /// Set the layer for this state
    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    /// Set the weight for this state
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Add an animation event to this state
    pub fn add_event(mut self, event: AnimationEvent) -> Self {
        self.events.push(event);
        self
    }

    /// Find the highest priority transition that should trigger
    pub fn find_transition(
        &self,
        current_time: f32,
        animation_state: PlaybackState,
        parameters: &StateParameters,
        triggered_events: &[String],
    ) -> Option<&StateTransition> {
        self.transitions.iter().find(|transition| {
            transition.should_trigger(current_time, animation_state, parameters, triggered_events)
        })
    }
}

/// Parameter values for controlling state transitions
#[derive(Debug, Clone)]
pub struct StateParameters {
    /// Float parameters
    pub floats: HashMap<String, f32>,
    /// Boolean parameters
    pub bools: HashMap<String, bool>,
    /// Integer parameters
    pub ints: HashMap<String, i32>,
    /// String parameters
    pub strings: HashMap<String, String>,
}

impl StateParameters {
    /// Create new empty state parameters
    pub fn new() -> Self {
        Self {
            floats: HashMap::new(),
            bools: HashMap::new(),
            ints: HashMap::new(),
            strings: HashMap::new(),
        }
    }

    /// Set a float parameter
    pub fn set_float(&mut self, name: String, value: f32) {
        self.floats.insert(name, value);
    }

    /// Get a float parameter
    pub fn get_float(&self, name: &str) -> Option<f32> {
        self.floats.get(name).copied()
    }

    /// Set a boolean parameter
    pub fn set_bool(&mut self, name: String, value: bool) {
        self.bools.insert(name, value);
    }

    /// Get a boolean parameter
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.bools.get(name).copied()
    }

    /// Set an integer parameter
    pub fn set_int(&mut self, name: String, value: i32) {
        self.ints.insert(name, value);
    }

    /// Get an integer parameter
    pub fn get_int(&self, name: &str) -> Option<i32> {
        self.ints.get(name).copied()
    }

    /// Set a string parameter
    pub fn set_string(&mut self, name: String, value: String) {
        self.strings.insert(name, value);
    }

    /// Get a string parameter
    pub fn get_string(&self, name: &str) -> Option<&String> {
        self.strings.get(name)
    }

    /// Check if a parameter condition is met
    pub fn check_condition(&self, param_name: &str, condition: &ParameterCondition) -> bool {
        match condition {
            ParameterCondition::FloatEquals(value, tolerance) => {
                if let Some(param_value) = self.get_float(param_name) {
                    (param_value - value).abs() <= *tolerance
                } else {
                    false
                }
            }
            ParameterCondition::FloatGreater(value) => {
                if let Some(param_value) = self.get_float(param_name) {
                    param_value > *value
                } else {
                    false
                }
            }
            ParameterCondition::FloatLess(value) => {
                if let Some(param_value) = self.get_float(param_name) {
                    param_value < *value
                } else {
                    false
                }
            }
            ParameterCondition::BoolEquals(value) => {
                if let Some(param_value) = self.get_bool(param_name) {
                    param_value == *value
                } else {
                    false
                }
            }
            ParameterCondition::IntEquals(value) => {
                if let Some(param_value) = self.get_int(param_name) {
                    param_value == *value
                } else {
                    false
                }
            }
            ParameterCondition::IntGreater(value) => {
                if let Some(param_value) = self.get_int(param_name) {
                    param_value > *value
                } else {
                    false
                }
            }
            ParameterCondition::IntLess(value) => {
                if let Some(param_value) = self.get_int(param_name) {
                    param_value < *value
                } else {
                    false
                }
            }
        }
    }
}

impl Default for StateParameters {
    fn default() -> Self {
        Self::new()
    }
}

/// An animation state machine that manages states and transitions
#[derive(Debug)]
pub struct AnimationStateMachine {
    /// All available states in the state machine
    states: HashMap<String, AnimationState>,
    /// Current active state
    current_state: Option<String>,
    /// Parameters for controlling transitions
    parameters: StateParameters,
    /// Current animation time in the active state
    current_time: f32,
    /// Current animation playback state
    current_playback_state: PlaybackState,
    /// Events triggered in the current frame
    triggered_events: Vec<String>,
    /// Whether the state machine is enabled
    enabled: bool,
    /// Transition progress (0.0 to 1.0) when transitioning between states
    transition_progress: f32,
    /// Duration of current transition
    transition_duration: f32,
    /// Source state for current transition
    transition_from: Option<String>,
    /// Target state for current transition
    transition_to: Option<String>,
}

impl AnimationStateMachine {
    /// Create a new animation state machine
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            current_state: None,
            parameters: StateParameters::new(),
            current_time: 0.0,
            current_playback_state: PlaybackState::Stopped,
            triggered_events: Vec::new(),
            enabled: true,
            transition_progress: 1.0,
            transition_duration: 0.0,
            transition_from: None,
            transition_to: None,
        }
    }

    /// Add a state to the state machine
    pub fn add_state(&mut self, state: AnimationState) {
        let is_entry = state.is_entry_state;
        let state_name = state.name.clone();

        self.states.insert(state_name.clone(), state);

        // Set as current state if it's an entry state and no current state is set
        if is_entry && self.current_state.is_none() {
            self.current_state = Some(state_name);
            self.current_playback_state = PlaybackState::Playing;
        }
    }

    /// Remove a state from the state machine
    pub fn remove_state(&mut self, name: &str) -> Option<AnimationState> {
        // Don't remove if it's the current state
        if self.current_state.as_ref() == Some(&name.to_string()) {
            return None;
        }

        self.states.remove(name)
    }

    /// Get a reference to a state
    pub fn get_state(&self, name: &str) -> Option<&AnimationState> {
        self.states.get(name)
    }

    /// Get a mutable reference to a state
    pub fn get_state_mut(&mut self, name: &str) -> Option<&mut AnimationState> {
        self.states.get_mut(name)
    }

    /// Manually trigger a state transition
    pub fn transition_to(
        &mut self,
        state_name: &str,
        transition_duration: Option<f32>,
    ) -> Result<(), String> {
        if !self.states.contains_key(state_name) {
            return Err(format!("State '{}' not found", state_name));
        }

        let duration = transition_duration.unwrap_or(0.2);

        if duration > 0.0 {
            // Start transition
            self.transition_from = self.current_state.clone();
            self.transition_to = Some(state_name.to_string());
            self.transition_duration = duration;
            self.transition_progress = 0.0;
        } else {
            // Immediate transition
            self.current_state = Some(state_name.to_string());
            self.current_time = 0.0;
            self.current_playback_state = PlaybackState::Playing;
            self.transition_progress = 1.0;
            self.transition_from = None;
            self.transition_to = None;
        }

        Ok(())
    }

    /// Trigger an event that can cause state transitions
    pub fn trigger_event(&mut self, event_name: String) {
        self.triggered_events.push(event_name);
    }

    /// Update the state machine
    pub fn update(&mut self, dt: Duration) -> Vec<AnimationEvent> {
        if !self.enabled {
            return Vec::new();
        }

        let mut events = Vec::new();

        // Update transition if in progress
        if self.transition_progress < 1.0 {
            self.transition_progress += dt.as_secs_f32() / self.transition_duration;

            if self.transition_progress >= 1.0 {
                // Transition complete
                self.transition_progress = 1.0;
                if let Some(target) = self.transition_to.take() {
                    self.current_state = Some(target);
                    self.current_time = 0.0;
                    self.current_playback_state = PlaybackState::Playing;
                }
                self.transition_from = None;
            }
        }

        // Update current animation time
        if let Some(current_state_name) = &self.current_state
            && let Some(current_state) = self.states.get(current_state_name)
        {
            self.current_time += dt.as_secs_f32() * current_state.speed;

            // Collect events from the current state
            for event in &current_state.events {
                if event.time <= self.current_time {
                    events.push(event.clone());
                }
            }

            // Check for state transitions (only if not currently transitioning)
            if self.transition_progress >= 1.0
                && let Some(transition) = current_state.find_transition(
                    self.current_time,
                    self.current_playback_state,
                    &self.parameters,
                    &self.triggered_events,
                )
            {
                // Clone the transition data to avoid borrowing issues
                let target_state = transition.target_state.clone();
                let transition_duration = transition.transition_duration;

                let _ = self.transition_to(&target_state, Some(transition_duration));
            }
        }

        // Clear triggered events after processing
        self.triggered_events.clear();

        events
    }

    /// Get the current state name
    pub fn current_state(&self) -> Option<&String> {
        self.current_state.as_ref()
    }

    /// Get the current animation clip name
    pub fn current_animation_clip(&self) -> Option<String> {
        if let Some(state_name) = &self.current_state {
            self.states
                .get(state_name)
                .map(|state| state.animation_clip.clone())
        } else {
            None
        }
    }

    /// Get the current playback state
    pub fn playback_state(&self) -> PlaybackState {
        self.current_playback_state
    }

    /// Set a parameter value
    pub fn set_float_parameter(&mut self, name: String, value: f32) {
        self.parameters.set_float(name, value);
    }

    /// Set a boolean parameter
    pub fn set_bool_parameter(&mut self, name: String, value: bool) {
        self.parameters.set_bool(name, value);
    }

    /// Set an integer parameter
    pub fn set_int_parameter(&mut self, name: String, value: i32) {
        self.parameters.set_int(name, value);
    }

    /// Get the parameters
    pub fn parameters(&self) -> &StateParameters {
        &self.parameters
    }

    /// Get mutable parameters
    pub fn parameters_mut(&mut self) -> &mut StateParameters {
        &mut self.parameters
    }

    /// Enable or disable the state machine
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if the state machine is transitioning
    pub fn is_transitioning(&self) -> bool {
        self.transition_progress < 1.0
    }

    /// Get transition progress (0.0 to 1.0)
    pub fn transition_progress(&self) -> f32 {
        self.transition_progress
    }

    /// Get all state names
    pub fn get_state_names(&self) -> Vec<String> {
        self.states.keys().cloned().collect()
    }

    /// Reset the state machine to entry state
    pub fn reset(&mut self) {
        // Find entry state
        let entry_state = self
            .states
            .values()
            .find(|state| state.is_entry_state)
            .map(|state| state.name.clone());

        if let Some(entry_state_name) = entry_state {
            self.current_state = Some(entry_state_name);
            self.current_time = 0.0;
            self.current_playback_state = PlaybackState::Playing;
            self.transition_progress = 1.0;
            self.transition_from = None;
            self.transition_to = None;
            self.triggered_events.clear();
        }
    }
}

impl Default for AnimationStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_creation() {
        let sm = AnimationStateMachine::new();
        assert!(sm.current_state().is_none());
        assert_eq!(sm.playback_state(), PlaybackState::Stopped);
    }

    #[test]
    fn test_parameter_conditions() {
        let mut params = StateParameters::new();
        params.set_float("speed".to_string(), 5.0);
        params.set_bool("running".to_string(), true);

        let float_condition = ParameterCondition::FloatGreater(3.0);
        assert!(params.check_condition("speed", &float_condition));

        let bool_condition = ParameterCondition::BoolEquals(true);
        assert!(params.check_condition("running", &bool_condition));
    }

    #[test]
    fn test_state_transitions() {
        let mut sm = AnimationStateMachine::new();

        let idle_state =
            AnimationState::new("idle".to_string(), "idle_anim".to_string()).as_entry_state();

        let run_state = AnimationState::new("run".to_string(), "run_anim".to_string());

        sm.add_state(idle_state);
        sm.add_state(run_state);

        assert_eq!(sm.current_state(), Some(&"idle".to_string()));

        sm.transition_to("run", Some(0.0)).unwrap();
        assert_eq!(sm.current_state(), Some(&"run".to_string()));
    }
}
