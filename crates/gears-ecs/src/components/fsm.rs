use crate::Component;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Trait that state identifiers must implement.
///
/// This trait allows users to define their own state enums while ensuring.
/// they work properly with the FSM system.
pub trait StateIdentifier:
    std::fmt::Debug + std::fmt::Display + Clone + Copy + std::hash::Hash + Eq + Send + Sync + 'static
{
    /// Convert to string for logging and debugging purposes.
    ///
    /// # Returns
    ///
    /// A static string representation of the state identifier.
    fn as_str(&self) -> &'static str;
}

/// A trait that defines the behavior of a state in the finite state machine.
///
/// States are the core building blocks of the FSM. Each state can have custom logic
/// for entering, updating, and exiting, as well as conditions for transitioning
/// to other states. States are inherently capable of having sub-states.
pub trait State<S: StateIdentifier>: std::fmt::Debug + Send + Sync {
    /// Called when entering this state.
    ///
    /// # Arguments
    ///
    /// * `_context` - A mutable reference to the state context.
    fn on_enter(&mut self, _context: &mut StateContext) {}

    /// Called every frame while in this state.
    ///
    /// # Arguments
    ///
    /// * `_context` - A mutable reference to the state context.
    /// * `_dt` - The duration since the last frame.
    fn on_update(&mut self, _context: &mut StateContext, _dt: Duration) {}

    /// Called when exiting this state.
    ///
    /// # Arguments
    ///
    /// * `_context` - A mutable reference to the state context.
    fn on_exit(&mut self, _context: &mut StateContext) {}

    /// Check for state transitions and return the next state Id if a transition should occur
    ///
    /// # Arguments
    ///
    /// * `_context` - A mutable reference to the state context.
    ///
    /// # Returns
    ///
    /// * The next state Id if a transition should occur.
    fn check_transitions(&self, _context: &StateContext) -> Option<S> {
        None
    }
}

/// Default state identifier enum.
///
/// This provides a sensible default set of states for common game scenarios.
/// Users can define their own state enums by implementing the StateIdentifier trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefaultStateId {
    // Main states
    Idle,
    Attack,
    Defend,
    Escape,

    // Sub-states for Idle
    IdleWander,
    IdleWatch,

    // Sub-states for Attack
    AttackApproach,
    AttackStrike,
    AttackRetreat,

    // Sub-states for Defend
    DefendBlock,
    DefendCounter,

    // Sub-states for Escape
    EscapeFlee,
    EscapeHide,
}

impl StateIdentifier for DefaultStateId {
    /// Convert to string for logging and debugging purposes.
    ///
    /// # Returns
    ///
    /// The string representation of the state identifier.
    fn as_str(&self) -> &'static str {
        match self {
            DefaultStateId::Idle => "idle",
            DefaultStateId::Attack => "attack",
            DefaultStateId::Defend => "defend",
            DefaultStateId::Escape => "escape",
            DefaultStateId::IdleWander => "idle_wander",
            DefaultStateId::IdleWatch => "idle_watch",
            DefaultStateId::AttackApproach => "attack_approach",
            DefaultStateId::AttackStrike => "attack_strike",
            DefaultStateId::AttackRetreat => "attack_retreat",
            DefaultStateId::DefendBlock => "defend_block",
            DefaultStateId::DefendCounter => "defend_counter",
            DefaultStateId::EscapeFlee => "escape_flee",
            DefaultStateId::EscapeHide => "escape_hide",
        }
    }
}

impl DefaultStateId {
    /// Check if this state is a main state (not a sub-state).
    ///
    /// # Returns
    ///
    /// `true` if this state is a main state.
    pub fn is_main_state(&self) -> bool {
        matches!(
            self,
            DefaultStateId::Idle
                | DefaultStateId::Attack
                | DefaultStateId::Defend
                | DefaultStateId::Escape
        )
    }

    /// Check if this state is a sub-state of the given parent.
    ///
    /// # Returns
    ///
    /// `true` if this state is a sub-state of the given parent.
    pub fn is_sub_state_of(&self, parent: DefaultStateId) -> bool {
        match parent {
            DefaultStateId::Idle => {
                matches!(self, DefaultStateId::IdleWander | DefaultStateId::IdleWatch)
            }
            DefaultStateId::Attack => matches!(
                self,
                DefaultStateId::AttackApproach
                    | DefaultStateId::AttackStrike
                    | DefaultStateId::AttackRetreat
            ),
            DefaultStateId::Defend => matches!(
                self,
                DefaultStateId::DefendBlock | DefaultStateId::DefendCounter
            ),
            DefaultStateId::Escape => matches!(
                self,
                DefaultStateId::EscapeFlee | DefaultStateId::EscapeHide
            ),
            _ => false,
        }
    }
}

impl std::fmt::Display for DefaultStateId {
    /// Formats the state Id as a string.
    ///
    /// # Returns
    ///
    /// A string representation of the state Id.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Type alias for backward compatibility
pub type StateId = DefaultStateId;

/// Context passed to states for decision making and data access
///
/// The [`StateContext`] provides a way for states to:
/// - Store and retrieve shared data
/// - Access timing information
/// - Track state history
/// - Communicate between states
#[derive(Debug, Clone)]
pub struct StateContext {
    /// Shared data that states can read and modify,
    pub data: HashMap<String, StateData>,
    /// Time since the current state was entered,
    pub time_in_state: Duration,
}

/// Generic data container for state context.
///
/// StateData provides type-safe storage for common data types used in game logic.
/// This allows states to share information without requiring complex type systems.
#[derive(Debug, Clone)]
pub enum StateData {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Vector3(cgmath::Vector3<f32>),
}

impl Default for StateContext {
    fn default() -> Self {
        Self::new()
    }
}

impl StateContext {
    /// Create a new state context with default values.
    ///
    /// # Returns
    ///
    /// A new [`StateContext`] instance.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            time_in_state: Duration::ZERO,
        }
    }

    /// Set a float value in the state context.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to associate with the value.
    /// * `value` - The float value to set.
    pub fn set_float(&mut self, key: &str, value: f32) {
        self.data.insert(key.to_string(), StateData::Float(value));
    }

    /// Get a float value from the state context.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to retrieve the value for.
    ///
    /// # Returns
    ///
    /// The `f32` value associated with the key.
    pub fn get_float(&self, key: &str) -> Option<f32> {
        match self.data.get(key) {
            Some(StateData::Float(value)) => Some(*value),
            _ => None,
        }
    }

    /// Set a boolean value in the state context.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to associate with the value.
    /// * `value` - The boolean value to set.
    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.data.insert(key.to_string(), StateData::Bool(value));
    }

    /// Get a boolean value from the state context.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to retrieve the value for.
    ///
    /// # Returns
    ///
    /// The `bool` value associated with the key.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.data.get(key) {
            Some(StateData::Bool(value)) => Some(*value),
            _ => None,
        }
    }

    /// Set a vector3 value in the state context.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to associate with the value.
    /// * `value` - The vector3 value to set.
    pub fn set_vector3(&mut self, key: &str, value: cgmath::Vector3<f32>) {
        self.data.insert(key.to_string(), StateData::Vector3(value));
    }

    /// Get a vector3 value from the state context.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to retrieve the value for.
    ///
    /// # Returns
    ///
    /// The `cgmath::Vector3<f32>` value associated with the key.
    pub fn get_vector3(&self, key: &str) -> Option<cgmath::Vector3<f32>> {
        match self.data.get(key) {
            Some(StateData::Vector3(value)) => Some(*value),
            _ => None,
        }
    }
}

/// The main hierarchical finite state machine component
///
/// This FSM is inherently hierarchical - every state can have sub-states, creating
/// natural state hierarchies. Simple states without sub-states work perfectly fine.
///
/// ## Generic Parameter
///
/// The FSM is generic over `S: StateIdentifier` allowing users to define their own
/// state enums. Defaults to [`DefaultStateId`] for convenience.
#[derive(Debug)]
pub struct FiniteStateMachine<S: StateIdentifier = DefaultStateId> {
    /// Main states and their implementations.
    states: HashMap<S, Box<dyn State<S>>>,
    /// Sub-states organized by parent state.
    sub_states: HashMap<S, HashMap<S, Box<dyn State<S>>>>,
    /// Initial sub-state for each parent state.
    initial_sub_states: HashMap<S, S>,
    /// Current active main state.
    current_state: Option<S>,
    /// Current active sub-state for the current main state.
    current_sub_state: Option<S>,
    /// Shared context for state communication.
    context: StateContext,
    /// Timestamp when current state was entered.
    state_enter_time: Instant,
    /// Whether the FSM is currently enabled.
    enabled: bool,
    /// Complete state hierarchy stack (main state + sub-states).
    state_stack: Vec<S>,
    /// Previously active main state.
    previous_state: Option<S>,
}

impl<S: StateIdentifier> FiniteStateMachine<S> {
    /// Create a new hierarchical finite state machine.
    ///
    /// # Returns
    ///
    /// A new [`FiniteStateMachine`] instance.
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            sub_states: HashMap::new(),
            initial_sub_states: HashMap::new(),
            current_state: None,
            current_sub_state: None,
            context: StateContext::new(),
            state_enter_time: Instant::now(),
            enabled: true,
            state_stack: Vec::new(),
            previous_state: None,
        }
    }

    /// Add a main state to the FSM.
    ///
    /// # Arguments
    ///
    /// * `id` - The identifier of the state.
    /// * `state` - The state to add.
    pub fn add_state(&mut self, id: S, state: Box<dyn State<S>>) {
        self.states.insert(id, state);
    }

    /// Add a sub-state to a parent state.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - The identifier of the parent state.
    /// * `sub_id` - The identifier of the sub-state.
    /// * `state` - The state to add.
    pub fn add_sub_state(&mut self, parent_id: S, sub_id: S, state: Box<dyn State<S>>) {
        self.sub_states
            .entry(parent_id)
            .or_default()
            .insert(sub_id, state);
    }

    /// Set the initial sub-state for a parent state.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - The identifier of the parent state.
    /// * `sub_id` - The identifier of the sub-state.
    pub fn set_initial_sub_state(&mut self, parent_id: S, sub_id: S) {
        if self
            .sub_states
            .get(&parent_id)
            .is_some_and(|subs| subs.contains_key(&sub_id))
        {
            self.initial_sub_states.insert(parent_id, sub_id);
        }
    }

    /// Set the initial state and enter it.
    ///
    /// # Arguments
    ///
    /// * `state_id` - The identifier of the state.
    pub fn set_initial_state(&mut self, state_id: S) {
        if self.states.contains_key(&state_id) {
            self.current_state = Some(state_id);
            self.state_enter_time = Instant::now();
            self.context.time_in_state = Duration::ZERO;
            self.state_stack.clear();
            self.state_stack.push(state_id);

            // Enter the main state
            if let Some(state) = self.states.get_mut(&state_id) {
                state.on_enter(&mut self.context);
            }

            // Enter initial sub-state if it exists
            if let Some(&initial_sub) = self.initial_sub_states.get(&state_id) {
                self.current_sub_state = Some(initial_sub);
                self.state_stack.push(initial_sub);

                if let Some(sub_states) = self.sub_states.get_mut(&state_id)
                    && let Some(sub_state) = sub_states.get_mut(&initial_sub)
                {
                    sub_state.on_enter(&mut self.context);
                }
            }
        }
    }

    /// Get the current state Id.
    ///
    /// # Returns
    ///
    /// The current state identifier if it exists.
    pub fn current_state(&self) -> Option<S> {
        self.current_state
    }

    /// Get the current state stack (for hierarchical states).
    ///
    /// # Returns
    ///
    /// The current state stack if it exists.
    pub fn state_stack(&self) -> &[S] {
        &self.state_stack
    }

    /// Get the previous state ID
    pub fn previous_state(&self) -> Option<S> {
        self.previous_state
    }

    /// Force a transition to a specific main state.
    ///
    /// # Arguments
    ///
    /// * `new_state_id` - The new main state identifier.
    pub fn transition_to(&mut self, new_state_id: S) {
        if !self.states.contains_key(&new_state_id) {
            return;
        }

        // Exit current sub-state if exists
        if let Some(current_main) = self.current_state
            && let Some(current_sub) = self.current_sub_state
            && let Some(sub_states) = self.sub_states.get_mut(&current_main)
            && let Some(sub_state) = sub_states.get_mut(&current_sub)
        {
            sub_state.on_exit(&mut self.context);
        }

        // Exit current main state
        if let Some(current_id) = self.current_state {
            if let Some(state) = self.states.get_mut(&current_id) {
                state.on_exit(&mut self.context);
            }
            self.previous_state = Some(current_id);
        }

        // Enter new main state
        self.current_state = Some(new_state_id);
        self.current_sub_state = None;
        self.state_enter_time = Instant::now();
        self.context.time_in_state = Duration::ZERO;
        self.state_stack.clear();
        self.state_stack.push(new_state_id);

        if let Some(state) = self.states.get_mut(&new_state_id) {
            state.on_enter(&mut self.context);
        }

        // Enter initial sub-state if it exists
        if let Some(&initial_sub) = self.initial_sub_states.get(&new_state_id) {
            self.current_sub_state = Some(initial_sub);
            self.state_stack.push(initial_sub);

            if let Some(sub_states) = self.sub_states.get_mut(&new_state_id)
                && let Some(sub_state) = sub_states.get_mut(&initial_sub)
            {
                sub_state.on_enter(&mut self.context);
            }
        }
    }

    /// Transition to a different sub-state within the current main state.
    ///
    /// # Arguments
    ///
    /// * `new_sub_state_id` - The new sub-state identifier.
    pub fn transition_to_sub_state(&mut self, new_sub_state_id: S) {
        let Some(current_main) = self.current_state else {
            return;
        };

        let Some(sub_states) = self.sub_states.get_mut(&current_main) else {
            return;
        };
        if !sub_states.contains_key(&new_sub_state_id) {
            return;
        }

        // Exit current sub-state
        if let Some(current_sub) = self.current_sub_state {
            if let Some(sub_state) = sub_states.get_mut(&current_sub) {
                sub_state.on_exit(&mut self.context);
            }
            // Remove current sub-state from stack
            if let Some(last) = self.state_stack.last()
                && *last == current_sub
            {
                self.state_stack.pop();
            }
        }

        // Enter new sub-state
        self.current_sub_state = Some(new_sub_state_id);
        self.state_stack.push(new_sub_state_id);

        if let Some(sub_state) = sub_states.get_mut(&new_sub_state_id) {
            sub_state.on_enter(&mut self.context);
        }
    }

    /// Update the FSM - handles both main states and sub-states naturally.
    ///
    /// # Arguments
    ///
    /// * `dt` - The duration since the last update.
    pub fn update(&mut self, dt: Duration) {
        if !self.enabled {
            return;
        }

        self.context.time_in_state = self.state_enter_time.elapsed();

        let Some(current_main) = self.current_state else {
            return;
        };

        // Update current sub-state first (if exists)
        if let Some(current_sub) = self.current_sub_state
            && let Some(sub_states) = self.sub_states.get_mut(&current_main)
            && let Some(sub_state) = sub_states.get_mut(&current_sub)
        {
            sub_state.on_update(&mut self.context, dt);

            // Check for sub-state transitions
            if let Some(next_sub_state) = sub_state.check_transitions(&self.context) {
                self.transition_to_sub_state(next_sub_state);
                return;
            }
        }

        // Update current main state
        if let Some(state) = self.states.get_mut(&current_main) {
            state.on_update(&mut self.context, dt);

            // Check for main state transitions
            if let Some(next_state) = state.check_transitions(&self.context) {
                self.transition_to(next_state);
            }
        }
    }

    /// Enable or disable the FSM.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable or disable the FSM.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if the FSM is enabled.
    ///
    /// # Returns
    ///
    /// `true` if the FSM is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get mutable access to the context.
    ///
    /// # Returns
    ///
    /// Mutable reference to the context.
    pub fn context_mut(&mut self) -> &mut StateContext {
        &mut self.context
    }

    /// Get read-only access to the context.
    ///
    /// # Returns
    ///
    /// Read-only reference to the context.
    pub fn context(&self) -> &StateContext {
        &self.context
    }

    /// Get the current active sub-state
    ///
    /// # Returns
    ///
    /// Option containing the current active sub-state.
    pub fn current_sub_state(&self) -> Option<S> {
        self.current_sub_state
    }
}

impl<S: StateIdentifier> Default for FiniteStateMachine<S> {
    /// Create a new FSM with default settings.
    ///
    /// # Returns
    ///
    /// A new [`FiniteStateMachine`] instance.
    fn default() -> Self {
        Self::new()
    }
}

// Manual Component implementation for generic FSM.
impl<S: StateIdentifier> Component for FiniteStateMachine<S> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // Test state implementations
    #[derive(Debug)]
    struct IdleState;
    impl State<DefaultStateId> for IdleState {
        fn on_enter(&mut self, context: &mut StateContext) {
            context.set_bool("idle_entered", true);
        }

        fn on_update(&mut self, context: &mut StateContext, _dt: Duration) {
            let count = context.get_float("update_count").unwrap_or(0.0);
            context.set_float("update_count", count + 1.0);
        }

        fn on_exit(&mut self, context: &mut StateContext) {
            context.set_bool("idle_exited", true);
        }

        fn check_transitions(&self, context: &StateContext) -> Option<DefaultStateId> {
            if context.get_bool("should_attack").unwrap_or(false) {
                Some(DefaultStateId::Attack)
            } else {
                None
            }
        }
    }

    #[derive(Debug)]
    struct AttackState;
    impl State<DefaultStateId> for AttackState {
        fn on_enter(&mut self, context: &mut StateContext) {
            context.set_bool("attack_entered", true);
        }

        fn on_exit(&mut self, context: &mut StateContext) {
            context.set_bool("attack_exited", true);
        }
    }

    #[derive(Debug)]
    struct SubStateWander;
    impl State<DefaultStateId> for SubStateWander {
        fn on_enter(&mut self, context: &mut StateContext) {
            context.set_bool("wander_entered", true);
        }

        fn check_transitions(&self, context: &StateContext) -> Option<DefaultStateId> {
            if context.get_bool("should_watch").unwrap_or(false) {
                Some(DefaultStateId::IdleWatch)
            } else {
                None
            }
        }
    }

    #[derive(Debug)]
    struct SubStateWatch;
    impl State<DefaultStateId> for SubStateWatch {
        fn on_enter(&mut self, context: &mut StateContext) {
            context.set_bool("watch_entered", true);
        }
    }

    #[test]
    fn test_fsm_creation() {
        let fsm: FiniteStateMachine<DefaultStateId> = FiniteStateMachine::new();
        assert!(fsm.current_state().is_none());
        assert!(fsm.is_enabled());
    }

    #[test]
    fn test_fsm_default() {
        let fsm: FiniteStateMachine<DefaultStateId> = FiniteStateMachine::default();
        assert!(fsm.current_state().is_none());
        assert!(fsm.is_enabled());
    }

    #[test]
    fn test_add_and_set_initial_state() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));
        fsm.set_initial_state(DefaultStateId::Idle);

        assert_eq!(fsm.current_state(), Some(DefaultStateId::Idle));
        assert_eq!(fsm.context().get_bool("idle_entered"), Some(true));
    }

    #[test]
    fn test_state_transition() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));
        fsm.add_state(DefaultStateId::Attack, Box::new(AttackState));
        fsm.set_initial_state(DefaultStateId::Idle);

        assert_eq!(fsm.current_state(), Some(DefaultStateId::Idle));

        fsm.transition_to(DefaultStateId::Attack);

        assert_eq!(fsm.current_state(), Some(DefaultStateId::Attack));
        assert_eq!(fsm.context().get_bool("idle_exited"), Some(true));
        assert_eq!(fsm.context().get_bool("attack_entered"), Some(true));
    }

    #[test]
    fn test_previous_state_tracking() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));
        fsm.add_state(DefaultStateId::Attack, Box::new(AttackState));
        fsm.set_initial_state(DefaultStateId::Idle);

        assert_eq!(fsm.previous_state(), None);

        fsm.transition_to(DefaultStateId::Attack);

        assert_eq!(fsm.previous_state(), Some(DefaultStateId::Idle));
    }

    #[test]
    fn test_state_stack() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));
        fsm.set_initial_state(DefaultStateId::Idle);

        let stack = fsm.state_stack();
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0], DefaultStateId::Idle);
    }

    #[test]
    fn test_update_calls_on_update() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));
        fsm.set_initial_state(DefaultStateId::Idle);

        fsm.update(Duration::from_millis(16));

        assert_eq!(fsm.context().get_float("update_count"), Some(1.0));

        fsm.update(Duration::from_millis(16));

        assert_eq!(fsm.context().get_float("update_count"), Some(2.0));
    }

    #[test]
    fn test_automatic_transition_from_check_transitions() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));
        fsm.add_state(DefaultStateId::Attack, Box::new(AttackState));
        fsm.set_initial_state(DefaultStateId::Idle);

        assert_eq!(fsm.current_state(), Some(DefaultStateId::Idle));

        // Set trigger for transition
        fsm.context_mut().set_bool("should_attack", true);
        fsm.update(Duration::from_millis(16));

        // Should have automatically transitioned
        assert_eq!(fsm.current_state(), Some(DefaultStateId::Attack));
    }

    #[test]
    fn test_enable_disable() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));
        fsm.set_initial_state(DefaultStateId::Idle);

        assert!(fsm.is_enabled());

        fsm.set_enabled(false);
        assert!(!fsm.is_enabled());

        // Update should not call on_update when disabled
        fsm.update(Duration::from_millis(16));
        assert_eq!(fsm.context().get_float("update_count"), None);

        fsm.set_enabled(true);
        fsm.update(Duration::from_millis(16));
        assert_eq!(fsm.context().get_float("update_count"), Some(1.0));
    }

    #[test]
    fn test_sub_states() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));
        fsm.add_sub_state(
            DefaultStateId::Idle,
            DefaultStateId::IdleWander,
            Box::new(SubStateWander),
        );
        fsm.add_sub_state(
            DefaultStateId::Idle,
            DefaultStateId::IdleWatch,
            Box::new(SubStateWatch),
        );
        fsm.set_initial_sub_state(DefaultStateId::Idle, DefaultStateId::IdleWander);
        fsm.set_initial_state(DefaultStateId::Idle);

        assert_eq!(fsm.current_state(), Some(DefaultStateId::Idle));
        assert_eq!(fsm.current_sub_state(), Some(DefaultStateId::IdleWander));
        assert_eq!(fsm.context().get_bool("wander_entered"), Some(true));

        // Check state stack has both main and sub state
        let stack = fsm.state_stack();
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0], DefaultStateId::Idle);
        assert_eq!(stack[1], DefaultStateId::IdleWander);
    }

    #[test]
    fn test_sub_state_transition() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));
        fsm.add_sub_state(
            DefaultStateId::Idle,
            DefaultStateId::IdleWander,
            Box::new(SubStateWander),
        );
        fsm.add_sub_state(
            DefaultStateId::Idle,
            DefaultStateId::IdleWatch,
            Box::new(SubStateWatch),
        );
        fsm.set_initial_sub_state(DefaultStateId::Idle, DefaultStateId::IdleWander);
        fsm.set_initial_state(DefaultStateId::Idle);

        assert_eq!(fsm.current_sub_state(), Some(DefaultStateId::IdleWander));

        fsm.transition_to_sub_state(DefaultStateId::IdleWatch);

        assert_eq!(fsm.current_sub_state(), Some(DefaultStateId::IdleWatch));
        assert_eq!(fsm.current_state(), Some(DefaultStateId::Idle)); // Main state unchanged
        assert_eq!(fsm.context().get_bool("watch_entered"), Some(true));
    }

    #[test]
    fn test_automatic_sub_state_transition() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));
        fsm.add_sub_state(
            DefaultStateId::Idle,
            DefaultStateId::IdleWander,
            Box::new(SubStateWander),
        );
        fsm.add_sub_state(
            DefaultStateId::Idle,
            DefaultStateId::IdleWatch,
            Box::new(SubStateWatch),
        );
        fsm.set_initial_sub_state(DefaultStateId::Idle, DefaultStateId::IdleWander);
        fsm.set_initial_state(DefaultStateId::Idle);

        assert_eq!(fsm.current_sub_state(), Some(DefaultStateId::IdleWander));

        // Trigger sub-state transition
        fsm.context_mut().set_bool("should_watch", true);
        fsm.update(Duration::from_millis(16));

        assert_eq!(fsm.current_sub_state(), Some(DefaultStateId::IdleWatch));
    }

    #[test]
    fn test_main_state_transition_exits_sub_state() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));
        fsm.add_state(DefaultStateId::Attack, Box::new(AttackState));
        fsm.add_sub_state(
            DefaultStateId::Idle,
            DefaultStateId::IdleWander,
            Box::new(SubStateWander),
        );
        fsm.set_initial_sub_state(DefaultStateId::Idle, DefaultStateId::IdleWander);
        fsm.set_initial_state(DefaultStateId::Idle);

        assert_eq!(fsm.current_sub_state(), Some(DefaultStateId::IdleWander));

        fsm.transition_to(DefaultStateId::Attack);

        // Sub-state should be cleared when transitioning main state
        assert_eq!(fsm.current_sub_state(), None);
        assert_eq!(fsm.current_state(), Some(DefaultStateId::Attack));
    }

    #[test]
    fn test_state_context_float() {
        let mut context = StateContext::new();
        context.set_float("health", 100.0);
        assert_eq!(context.get_float("health"), Some(100.0));

        context.set_float("health", 75.5);
        assert_eq!(context.get_float("health"), Some(75.5));

        assert_eq!(context.get_float("nonexistent"), None);
    }

    #[test]
    fn test_state_context_bool() {
        let mut context = StateContext::new();
        context.set_bool("is_alive", true);
        assert_eq!(context.get_bool("is_alive"), Some(true));

        context.set_bool("is_alive", false);
        assert_eq!(context.get_bool("is_alive"), Some(false));

        assert_eq!(context.get_bool("nonexistent"), None);
    }

    #[test]
    fn test_state_context_vector3() {
        let mut context = StateContext::new();
        let position = cgmath::Vector3::new(1.0, 2.0, 3.0);
        context.set_vector3("position", position);
        assert_eq!(context.get_vector3("position"), Some(position));

        assert_eq!(context.get_vector3("nonexistent"), None);
    }

    #[test]
    fn test_state_context_type_mismatch() {
        let mut context = StateContext::new();
        context.set_float("value", 42.0);

        // Trying to get as bool should return None
        assert_eq!(context.get_bool("value"), None);
        // Should still work as float
        assert_eq!(context.get_float("value"), Some(42.0));
    }

    #[test]
    fn test_default_state_id_as_str() {
        assert_eq!(DefaultStateId::Idle.as_str(), "idle");
        assert_eq!(DefaultStateId::Attack.as_str(), "attack");
        assert_eq!(DefaultStateId::IdleWander.as_str(), "idle_wander");
        assert_eq!(DefaultStateId::AttackStrike.as_str(), "attack_strike");
    }

    #[test]
    fn test_default_state_id_display() {
        assert_eq!(format!("{}", DefaultStateId::Idle), "idle");
        assert_eq!(format!("{}", DefaultStateId::Attack), "attack");
    }

    #[test]
    fn test_default_state_id_is_main_state() {
        assert!(DefaultStateId::Idle.is_main_state());
        assert!(DefaultStateId::Attack.is_main_state());
        assert!(DefaultStateId::Defend.is_main_state());
        assert!(DefaultStateId::Escape.is_main_state());

        assert!(!DefaultStateId::IdleWander.is_main_state());
        assert!(!DefaultStateId::AttackStrike.is_main_state());
    }

    #[test]
    fn test_default_state_id_is_sub_state_of() {
        assert!(DefaultStateId::IdleWander.is_sub_state_of(DefaultStateId::Idle));
        assert!(DefaultStateId::IdleWatch.is_sub_state_of(DefaultStateId::Idle));
        assert!(DefaultStateId::AttackStrike.is_sub_state_of(DefaultStateId::Attack));

        assert!(!DefaultStateId::IdleWander.is_sub_state_of(DefaultStateId::Attack));
        assert!(!DefaultStateId::Idle.is_sub_state_of(DefaultStateId::Idle));
    }

    #[test]
    fn test_transition_to_nonexistent_state_does_nothing() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));
        fsm.set_initial_state(DefaultStateId::Idle);

        let initial_state = fsm.current_state();

        // Try to transition to a state that wasn't added
        fsm.transition_to(DefaultStateId::Attack);

        // Should remain in the same state
        assert_eq!(fsm.current_state(), initial_state);
    }

    #[test]
    fn test_set_initial_sub_state_validates_existence() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));

        // Try to set initial sub-state without adding it first
        fsm.set_initial_sub_state(DefaultStateId::Idle, DefaultStateId::IdleWander);

        // When we set the initial state, no sub-state should be entered
        fsm.set_initial_state(DefaultStateId::Idle);
        assert_eq!(fsm.current_sub_state(), None);
    }

    #[test]
    fn test_context_time_in_state_updates() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));
        fsm.set_initial_state(DefaultStateId::Idle);

        // Initially should be zero
        assert!(fsm.context().time_in_state < Duration::from_millis(10));

        // Small delay to ensure time passes
        std::thread::sleep(Duration::from_millis(10));

        fsm.update(Duration::from_millis(16));

        // Time should have increased
        assert!(fsm.context().time_in_state >= Duration::from_millis(10));
    }

    #[test]
    fn test_context_mut_and_context() {
        let mut fsm: FiniteStateMachine<DefaultStateId> = FiniteStateMachine::new();

        // Test mutable access
        fsm.context_mut().set_float("test", 42.0);

        // Test read-only access
        assert_eq!(fsm.context().get_float("test"), Some(42.0));
    }

    #[test]
    fn test_state_stack_with_sub_states() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));
        fsm.add_sub_state(
            DefaultStateId::Idle,
            DefaultStateId::IdleWander,
            Box::new(SubStateWander),
        );
        fsm.set_initial_sub_state(DefaultStateId::Idle, DefaultStateId::IdleWander);
        fsm.set_initial_state(DefaultStateId::Idle);

        let stack = fsm.state_stack();
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0], DefaultStateId::Idle);
        assert_eq!(stack[1], DefaultStateId::IdleWander);

        // Transition to different sub-state
        fsm.add_sub_state(
            DefaultStateId::Idle,
            DefaultStateId::IdleWatch,
            Box::new(SubStateWatch),
        );
        fsm.transition_to_sub_state(DefaultStateId::IdleWatch);

        let stack = fsm.state_stack();
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0], DefaultStateId::Idle);
        assert_eq!(stack[1], DefaultStateId::IdleWatch);
    }

    #[test]
    fn test_transition_clears_state_stack() {
        let mut fsm = FiniteStateMachine::new();
        fsm.add_state(DefaultStateId::Idle, Box::new(IdleState));
        fsm.add_state(DefaultStateId::Attack, Box::new(AttackState));
        fsm.add_sub_state(
            DefaultStateId::Idle,
            DefaultStateId::IdleWander,
            Box::new(SubStateWander),
        );
        fsm.set_initial_sub_state(DefaultStateId::Idle, DefaultStateId::IdleWander);
        fsm.set_initial_state(DefaultStateId::Idle);

        assert_eq!(fsm.state_stack().len(), 2);

        fsm.transition_to(DefaultStateId::Attack);

        // Stack should only have the new main state
        assert_eq!(fsm.state_stack().len(), 1);
        assert_eq!(fsm.state_stack()[0], DefaultStateId::Attack);
    }
}
