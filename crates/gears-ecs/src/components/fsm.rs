//! # Hierarchical Finite State Machine (FSM) Component
//!
//! This module provides a complete implementation of a hierarchical finite state machine
//! that integrates seamlessly with the Entity Component System (ECS).
//!
//! ## Features
//!
//! - **State Management**: Define states with enter, update, and exit callbacks
//! - **Hierarchical Support**: States can contain sub-states for complex behaviors
//! - **Context Data**: Share data between states using a flexible context system
//! - **Transition Logic**: Automatic state transitions based on conditions
//! - **ECS Integration**: Works as a standard ECS component
//!
//! ## Example Usage
//!
//! ```rust
//! use gears_ecs::components::fsm::*;
//! use std::time::Duration;
//!
//! // Define state constants
//! const IDLE: StateId = "idle";
//! const MOVING: StateId = "moving";
//!
//! // Create a simple state
//! #[derive(Debug)]
//! struct IdleState;
//!
//! impl State for IdleState {
//!     fn on_enter(&mut self, context: &mut StateContext) {
//!         context.set_float("speed", 0.0);
//!         println!("Entered idle state");
//!     }
//!
//!     fn check_transitions(&self, context: &StateContext) -> Option<StateId> {
//!         if context.get_bool("should_move").unwrap_or(false) {
//!             Some(MOVING)
//!         } else {
//!             None
//!         }
//!     }
//! }
//!
//! // Create and configure the FSM
//! let mut fsm = FiniteStateMachine::new();
//! fsm.add_state(IDLE, Box::new(IdleState));
//! fsm.set_initial_state(IDLE);
//!
//! // Update the FSM each frame
//! fsm.update(Duration::from_millis(16));
//! ```

use crate::Component;
use gears_macro::Component;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A trait that defines the behavior of a state in the finite state machine
///
/// States are the core building blocks of the FSM. Each state can have custom logic
/// for entering, updating, and exiting, as well as conditions for transitioning
/// to other states.
pub trait State: std::fmt::Debug + Send + Sync {
    /// Called when entering this state
    fn on_enter(&mut self, _context: &mut StateContext) {}

    /// Called every frame while in this state
    fn on_update(&mut self, _context: &mut StateContext, _dt: Duration) {}

    /// Called when exiting this state
    fn on_exit(&mut self, _context: &mut StateContext) {}

    /// Check for state transitions and return the next state ID if a transition should occur
    fn check_transitions(&self, _context: &StateContext) -> Option<StateId> {
        None
    }

    /// Get the sub-states of this state (for hierarchical FSM)
    fn get_sub_states(&self) -> Option<&HashMap<StateId, Box<dyn State>>> {
        None
    }

    /// Get the current active sub-state ID
    fn get_current_sub_state(&self) -> Option<StateId> {
        None
    }

    /// Set the current active sub-state
    fn set_current_sub_state(&mut self, _state_id: Option<StateId>) {}
}

/// Unique identifier for states using a strongly-typed enum
///
/// State IDs are enum variants that provide compile-time safety and better performance
/// than string-based identification. Each variant represents a unique state in the FSM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateId {
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

impl StateId {
    /// Convert to string for logging and debugging purposes
    pub fn as_str(&self) -> &'static str {
        match self {
            StateId::Idle => "idle",
            StateId::Attack => "attack",
            StateId::Defend => "defend",
            StateId::Escape => "escape",
            StateId::IdleWander => "idle_wander",
            StateId::IdleWatch => "idle_watch",
            StateId::AttackApproach => "attack_approach",
            StateId::AttackStrike => "attack_strike",
            StateId::AttackRetreat => "attack_retreat",
            StateId::DefendBlock => "defend_block",
            StateId::DefendCounter => "defend_counter",
            StateId::EscapeFlee => "escape_flee",
            StateId::EscapeHide => "escape_hide",
        }
    }

    /// Check if this state is a main state (not a sub-state)
    pub fn is_main_state(&self) -> bool {
        matches!(
            self,
            StateId::Idle | StateId::Attack | StateId::Defend | StateId::Escape
        )
    }

    /// Check if this state is a sub-state of the given parent
    pub fn is_sub_state_of(&self, parent: StateId) -> bool {
        match parent {
            StateId::Idle => matches!(self, StateId::IdleWander | StateId::IdleWatch),
            StateId::Attack => matches!(
                self,
                StateId::AttackApproach | StateId::AttackStrike | StateId::AttackRetreat
            ),
            StateId::Defend => matches!(self, StateId::DefendBlock | StateId::DefendCounter),
            StateId::Escape => matches!(self, StateId::EscapeFlee | StateId::EscapeHide),
            _ => false,
        }
    }
}

impl std::fmt::Display for StateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Context passed to states for decision making and data access
///
/// The StateContext provides a way for states to:
/// - Store and retrieve shared data
/// - Access timing information
/// - Track state history
/// - Communicate between states
///
/// Data is stored using a flexible enum system that supports common game types.
#[derive(Debug, Clone)]
pub struct StateContext {
    /// Shared data that states can read and modify
    pub data: HashMap<String, StateData>,
    /// Time since the current state was entered
    pub time_in_state: Duration,
    /// Previous state ID
    pub previous_state: Option<StateId>,
    /// Current state stack (for hierarchical states)
    pub state_stack: Vec<StateId>,
}

/// Generic data container for state context
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

impl StateContext {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            time_in_state: Duration::ZERO,
            previous_state: None,
            state_stack: Vec::new(),
        }
    }

    pub fn set_float(&mut self, key: &str, value: f32) {
        self.data.insert(key.to_string(), StateData::Float(value));
    }

    pub fn get_float(&self, key: &str) -> Option<f32> {
        match self.data.get(key) {
            Some(StateData::Float(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.data.insert(key.to_string(), StateData::Bool(value));
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.data.get(key) {
            Some(StateData::Bool(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn set_vector3(&mut self, key: &str, value: cgmath::Vector3<f32>) {
        self.data.insert(key.to_string(), StateData::Vector3(value));
    }

    pub fn get_vector3(&self, key: &str) -> Option<cgmath::Vector3<f32>> {
        match self.data.get(key) {
            Some(StateData::Vector3(value)) => Some(*value),
            _ => None,
        }
    }
}

/// The main finite state machine component
///
/// This is the primary component that manages state transitions and execution.
/// It can be attached to any entity that needs state-based behavior.
///
/// ## Usage in ECS
///
/// ```rust
/// // Add to an entity using the new_entity! macro
/// let entity = new_entity!(
///     app,
///     Name("Character"),
///     Pos3::default(),
///     character_fsm, // Your configured FSM
/// );
/// ```
///
/// ## State Management
///
/// The FSM manages a collection of states and handles transitions between them
/// based on the logic defined in each state's `check_transitions` method.
#[derive(Component, Debug)]
pub struct FiniteStateMachine {
    states: HashMap<StateId, Box<dyn State>>,
    current_state: Option<StateId>,
    context: StateContext,
    state_enter_time: Instant,
    enabled: bool,
}

impl FiniteStateMachine {
    /// Create a new finite state machine
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            current_state: None,
            context: StateContext::new(),
            state_enter_time: Instant::now(),
            enabled: true,
        }
    }

    /// Add a state to the FSM
    pub fn add_state(&mut self, id: StateId, state: Box<dyn State>) {
        self.states.insert(id, state);
    }

    /// Set the initial state
    pub fn set_initial_state(&mut self, state_id: StateId) {
        if self.states.contains_key(&state_id) {
            self.current_state = Some(state_id);
            self.state_enter_time = Instant::now();
            self.context.time_in_state = Duration::ZERO;
            self.context.state_stack.clear();
            self.context.state_stack.push(state_id);

            if let Some(state) = self.states.get_mut(&state_id) {
                state.on_enter(&mut self.context);
            }
        }
    }

    /// Get the current state ID
    pub fn current_state(&self) -> Option<StateId> {
        self.current_state
    }

    /// Get the current state stack (for hierarchical states)
    pub fn state_stack(&self) -> &[StateId] {
        &self.context.state_stack
    }

    /// Force a transition to a specific state
    pub fn transition_to(&mut self, new_state_id: StateId) {
        if !self.states.contains_key(&new_state_id) {
            return;
        }

        // Exit current state
        if let Some(current_id) = self.current_state {
            if let Some(state) = self.states.get_mut(&current_id) {
                state.on_exit(&mut self.context);
            }
            self.context.previous_state = Some(current_id);
        }

        // Enter new state
        self.current_state = Some(new_state_id);
        self.state_enter_time = Instant::now();
        self.context.time_in_state = Duration::ZERO;

        // Update state stack for hierarchical FSM
        self.context.state_stack.clear();
        self.context.state_stack.push(new_state_id);

        if let Some(state) = self.states.get_mut(&new_state_id) {
            state.on_enter(&mut self.context);
        }
    }

    /// Update the FSM
    pub fn update(&mut self, dt: Duration) {
        if !self.enabled {
            return;
        }

        self.context.time_in_state = self.state_enter_time.elapsed();

        if let Some(current_id) = self.current_state {
            // Update current state
            if let Some(state) = self.states.get_mut(&current_id) {
                state.on_update(&mut self.context, dt);

                // Check for transitions
                if let Some(next_state) = state.check_transitions(&self.context) {
                    self.transition_to(next_state);
                    return;
                }

                // Handle hierarchical states
                self.update_hierarchical_state(current_id, dt);
            }
        }
    }

    /// Update hierarchical sub-states
    fn update_hierarchical_state(&mut self, state_id: StateId, dt: Duration) {
        if let Some(state) = self.states.get_mut(&state_id) {
            if state.get_sub_states().is_some() {
                if let Some(current_sub_state) = state.get_current_sub_state() {
                    // Check for sub-state transitions first
                    if let Some(sub_states) = state.get_sub_states() {
                        if let Some(sub_state) = sub_states.get(&current_sub_state) {
                            if let Some(next_sub_state) = sub_state.check_transitions(&self.context)
                            {
                                self.transition_sub_state(state_id, next_sub_state);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Transition to a sub-state within a hierarchical state
    fn transition_sub_state(&mut self, parent_state_id: StateId, new_sub_state_id: StateId) {
        if let Some(parent_state) = self.states.get_mut(&parent_state_id) {
            // Exit current sub-state
            if let Some(current_sub_state_id) = parent_state.get_current_sub_state() {
                if let Some(sub_states) = parent_state.get_sub_states() {
                    if let Some(_current_sub_state) = sub_states.get(&current_sub_state_id) {
                        // Note: We can't call on_exit here due to borrowing rules
                        // This is handled in the HierarchicalState implementation
                    }
                }
                // Remove the old sub-state from stack
                if let Some(last) = self.context.state_stack.last() {
                    if *last == current_sub_state_id {
                        self.context.state_stack.pop();
                    }
                }
            }

            // Set new sub-state
            parent_state.set_current_sub_state(Some(new_sub_state_id));

            // Add new sub-state to stack
            self.context.state_stack.push(new_sub_state_id);

            // Enter new sub-state (handled in HierarchicalState implementation)
        }
    }

    /// Enable or disable the FSM
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if the FSM is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get mutable access to the context
    pub fn context_mut(&mut self) -> &mut StateContext {
        &mut self.context
    }

    /// Get read-only access to the context
    pub fn context(&self) -> &StateContext {
        &self.context
    }

    /// Get the current active sub-state if in a hierarchical state
    pub fn current_sub_state(&self) -> Option<StateId> {
        if let Some(current_state_id) = self.current_state {
            if let Some(state) = self.states.get(&current_state_id) {
                return state.get_current_sub_state();
            }
        }
        None
    }
}

impl Default for FiniteStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience macro for creating simple states
#[macro_export]
macro_rules! simple_state {
    ($name:ident, $on_enter:expr, $on_update:expr, $on_exit:expr, $check_transitions:expr) => {
        #[derive(Debug)]
        struct $name;

        impl State for $name {
            fn on_enter(&mut self, context: &mut StateContext) {
                $on_enter(context);
            }

            fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
                $on_update(context, dt);
            }

            fn on_exit(&mut self, context: &mut StateContext) {
                $on_exit(context);
            }

            fn check_transitions(&self, context: &StateContext) -> Option<StateId> {
                $check_transitions(context)
            }
        }
    };
}

/// A hierarchical state that can contain sub-states
///
/// HierarchicalState allows you to create complex state machines where a single
/// state can contain multiple sub-states. This is useful for creating layered
/// behavior systems.
///
/// ## Example
///
/// ```rust
/// // Create a hierarchical combat state with sub-states
/// let mut combat_state = HierarchicalState::new()
///     .with_enter_callback(|ctx| {
///         ctx.set_bool("in_combat", true);
///     })
///     .with_exit_callback(|ctx| {
///         ctx.set_bool("in_combat", false);
///     });
///
/// combat_state.add_sub_state("attacking", Box::new(AttackState));
/// combat_state.add_sub_state("defending", Box::new(DefendState));
/// combat_state.set_initial_sub_state("attacking");
/// ```
pub struct HierarchicalState {
    sub_states: HashMap<StateId, Box<dyn State>>,
    current_sub_state: Option<StateId>,
    enter_callback: Option<Box<dyn Fn(&mut StateContext) + Send + Sync>>,
    update_callback: Option<Box<dyn Fn(&mut StateContext, Duration) + Send + Sync>>,
    exit_callback: Option<Box<dyn Fn(&mut StateContext) + Send + Sync>>,
    transition_callback: Option<Box<dyn Fn(&StateContext) -> Option<StateId> + Send + Sync>>,
}

/// Custom Debug implementation for HierarchicalState
///
/// Since function pointers can't be debugged directly, we show information
/// about which callbacks are configured instead.
impl std::fmt::Debug for HierarchicalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HierarchicalState")
            .field("sub_states", &self.sub_states.keys().collect::<Vec<_>>())
            .field("current_sub_state", &self.current_sub_state)
            .field("has_enter_callback", &self.enter_callback.is_some())
            .field("has_update_callback", &self.update_callback.is_some())
            .field("has_exit_callback", &self.exit_callback.is_some())
            .field(
                "has_transition_callback",
                &self.transition_callback.is_some(),
            )
            .finish()
    }
}

impl HierarchicalState {
    /// Create a new hierarchical state with no sub-states or callbacks
    pub fn new() -> Self {
        Self {
            sub_states: HashMap::new(),
            current_sub_state: None,
            enter_callback: None,
            update_callback: None,
            exit_callback: None,
            transition_callback: None,
        }
    }

    /// Add a sub-state to this hierarchical state
    pub fn add_sub_state(&mut self, id: StateId, state: Box<dyn State>) {
        self.sub_states.insert(id, state);
    }

    /// Set the initial sub-state that will be entered when this state becomes active
    pub fn set_initial_sub_state(&mut self, state_id: StateId) {
        if self.sub_states.contains_key(&state_id) {
            self.current_sub_state = Some(state_id);
        }
    }

    /// Add a callback that will be executed when entering this state
    pub fn with_enter_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut StateContext) + Send + Sync + 'static,
    {
        self.enter_callback = Some(Box::new(callback));
        self
    }

    /// Add a callback that will be executed every frame while in this state
    pub fn with_update_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut StateContext, Duration) + Send + Sync + 'static,
    {
        self.update_callback = Some(Box::new(callback));
        self
    }

    /// Add a callback that will be executed when exiting this state
    pub fn with_exit_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut StateContext) + Send + Sync + 'static,
    {
        self.exit_callback = Some(Box::new(callback));
        self
    }

    /// Add a callback that determines when this state should transition to another
    pub fn with_transition_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&StateContext) -> Option<StateId> + Send + Sync + 'static,
    {
        self.transition_callback = Some(Box::new(callback));
        self
    }
}

impl State for HierarchicalState {
    fn on_enter(&mut self, context: &mut StateContext) {
        if let Some(callback) = &self.enter_callback {
            callback(context);
        }

        // Enter the initial sub-state if available
        if let Some(sub_state_id) = self.current_sub_state {
            if let Some(sub_state) = self.sub_states.get_mut(&sub_state_id) {
                context.state_stack.push(sub_state_id);
                sub_state.on_enter(context);
            }
        }
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        if let Some(callback) = &self.update_callback {
            callback(context, dt);
        }

        // Update current sub-state
        if let Some(sub_state_id) = self.current_sub_state {
            if let Some(sub_state) = self.sub_states.get_mut(&sub_state_id) {
                sub_state.on_update(context, dt);

                // Check for sub-state transitions
                if let Some(next_sub_state) = sub_state.check_transitions(context) {
                    self.transition_to_sub_state(next_sub_state, context);
                }
            }
        }
    }

    fn on_exit(&mut self, context: &mut StateContext) {
        // Exit current sub-state
        if let Some(sub_state_id) = self.current_sub_state {
            if let Some(sub_state) = self.sub_states.get_mut(&sub_state_id) {
                sub_state.on_exit(context);
                context.state_stack.pop();
            }
        }

        if let Some(callback) = &self.exit_callback {
            callback(context);
        }
    }

    fn check_transitions(&self, context: &StateContext) -> Option<StateId> {
        if let Some(callback) = &self.transition_callback {
            callback(context)
        } else {
            None
        }
    }

    fn get_sub_states(&self) -> Option<&HashMap<StateId, Box<dyn State>>> {
        Some(&self.sub_states)
    }

    fn get_current_sub_state(&self) -> Option<StateId> {
        self.current_sub_state
    }

    fn set_current_sub_state(&mut self, state_id: Option<StateId>) {
        self.current_sub_state = state_id;
    }
}

impl HierarchicalState {
    /// Transition to a different sub-state within this hierarchical state
    fn transition_to_sub_state(&mut self, new_sub_state_id: StateId, context: &mut StateContext) {
        // Exit current sub-state
        if let Some(current_sub_state_id) = self.current_sub_state {
            if let Some(current_sub_state) = self.sub_states.get_mut(&current_sub_state_id) {
                current_sub_state.on_exit(context);
            }
            // Remove from state stack
            if let Some(last) = context.state_stack.last() {
                if *last == current_sub_state_id {
                    context.state_stack.pop();
                }
            }
        }

        // Enter new sub-state
        if self.sub_states.contains_key(&new_sub_state_id) {
            self.current_sub_state = Some(new_sub_state_id);
            context.state_stack.push(new_sub_state_id);

            if let Some(new_sub_state) = self.sub_states.get_mut(&new_sub_state_id) {
                new_sub_state.on_enter(context);
            }
        }
    }
}
