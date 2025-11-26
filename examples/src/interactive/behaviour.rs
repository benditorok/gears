use gears_app::prelude::*;
use log::info;
use rand::Rng;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum CharacterState {
    Idle,
    Attack,
    AttackApproach,
    AttackStrike,
    AttackRetreat,
    Defend,
    Escape,
}

impl std::fmt::Display for CharacterState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl StateIdentifier for CharacterState {
    fn as_str(&self) -> &'static str {
        match self {
            CharacterState::Idle => "Idle",
            CharacterState::Attack => "Attack",
            CharacterState::AttackApproach => "AttackApproach",
            CharacterState::AttackStrike => "AttackStrike",
            CharacterState::AttackRetreat => "AttackRetreat",
            CharacterState::Defend => "Defend",
            CharacterState::Escape => "Escape",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum PathfindingBehavior {
    Pursue,   // Move towards target using A*
    Maintain, // Keep safe distance from target
    Flee,     // Move away from target
    Wander,   // Random exploration
    Guard,    // Stay in area
}

// ================================
// FSM STATE IMPLEMENTATIONS
// ================================

#[derive(Debug)]
pub(super) struct IdleState;

impl State<CharacterState> for IdleState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("idle_timer", 0.0);
        context.set_float("speed", 8.0);
        context.set_float("wander_timer", 0.0);

        // Generate initial random wander point within map bounds (-40 to 40)
        let mut rng = rand::rng();
        let wander_x = rng.random_range(-40.0..40.0);
        let wander_z = rng.random_range(-40.0..40.0);
        context.set_vector3(
            "wander_target",
            cgmath::Vector3::new(wander_x, 1.0, wander_z),
        );
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("idle_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("idle_timer", timer);

        let wander_timer = context.get_float("wander_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("wander_timer", wander_timer);

        // Generate new wander point every 8-15 seconds
        if wander_timer > 10.0 {
            let mut rng = rand::rng();
            let wander_x = rng.random_range(-40.0..40.0);
            let wander_z = rng.random_range(-40.0..40.0);
            context.set_vector3(
                "wander_target",
                cgmath::Vector3::new(wander_x, 1.0, wander_z),
            );
            context.set_float("wander_timer", 0.0);
        }
    }

    fn check_transitions(&self, context: &StateContext) -> Option<CharacterState> {
        let health = context.get_float("health").unwrap_or(100.0);
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);

        if health < 30.0 {
            Some(CharacterState::Escape)
        } else if enemy_distance < 15.0 && health > 60.0 {
            Some(CharacterState::Attack)
        } else if enemy_distance < 8.0 && health <= 60.0 {
            Some(CharacterState::Defend)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(super) struct AttackState;

impl State<CharacterState> for AttackState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("speed", 12.0);
        context.set_vector3("color", [0.8, 0.4, 0.1].into()); // Orange
        info!("AI entered Attack state");
    }

    fn on_update(&mut self, _context: &mut StateContext, _dt: Duration) {
        // Attack logic handled by sub-states
    }

    fn on_exit(&mut self, _context: &mut StateContext) {
        info!("AI exiting Attack state");
    }

    fn check_transitions(&self, context: &StateContext) -> Option<CharacterState> {
        let health = context.get_float("health").unwrap_or(100.0);
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);

        if health < 30.0 {
            Some(CharacterState::Escape)
        } else if enemy_distance > 20.0 {
            Some(CharacterState::Idle)
        } else if health < 60.0 {
            Some(CharacterState::Defend)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(super) struct AttackApproachState;

impl State<CharacterState> for AttackApproachState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("speed", 12.0);
        context.set_float("approach_timer", 0.0);
        context.set_vector3("color", [0.8, 0.4, 0.1].into()); // Orange
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("approach_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("approach_timer", timer);
    }

    fn check_transitions(&self, context: &StateContext) -> Option<CharacterState> {
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);
        let approach_timer = context.get_float("approach_timer").unwrap_or(0.0);

        if enemy_distance < 5.0 {
            Some(CharacterState::AttackStrike)
        } else if approach_timer > 15.0 {
            // Been approaching for too long without reaching target, might be stuck
            info!("Approach timeout, returning to Idle");
            Some(CharacterState::Idle)
        } else {
            // Let parent Attack state handle distance-based transitions
            None
        }
    }
}

#[derive(Debug)]
pub(super) struct AttackStrikeState;

impl State<CharacterState> for AttackStrikeState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("strike_timer", 0.0);
        context.set_float("speed", 2.0);
        context.set_vector3("color", [1.0, 0.1, 0.1].into()); // Bright red
        info!("AI striking!");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("strike_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("strike_timer", timer);
    }

    fn check_transitions(&self, context: &StateContext) -> Option<CharacterState> {
        let timer = context.get_float("strike_timer").unwrap_or(0.0);
        if timer > 1.0 {
            Some(CharacterState::AttackRetreat)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(super) struct AttackRetreatState;

impl State<CharacterState> for AttackRetreatState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("retreat_timer", 0.0);
        context.set_float("speed", 10.0);
        context.set_vector3("color", [0.6, 0.2, 0.2].into()); // Dark red
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("retreat_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("retreat_timer", timer);
    }

    fn check_transitions(&self, context: &StateContext) -> Option<CharacterState> {
        let timer = context.get_float("retreat_timer").unwrap_or(0.0);
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);

        if timer > 0.5 && enemy_distance > 8.0 {
            Some(CharacterState::AttackApproach)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(super) struct DefendState;

impl State<CharacterState> for DefendState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("defend_timer", 0.0);
        context.set_float("speed", 8.0);
        context.set_vector3("color", [0.8, 0.8, 0.2].into()); // Yellow
        info!("AI entered Defend state");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("defend_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("defend_timer", timer);
    }

    fn on_exit(&mut self, _context: &mut StateContext) {
        info!("AI exiting Defend state");
    }

    fn check_transitions(&self, context: &StateContext) -> Option<CharacterState> {
        let health = context.get_float("health").unwrap_or(100.0);
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);
        let defend_timer = context.get_float("defend_timer").unwrap_or(0.0);

        if health < 30.0 {
            Some(CharacterState::Escape)
        } else if health > 70.0 && enemy_distance < 10.0 {
            Some(CharacterState::Attack)
        } else if enemy_distance > 20.0 {
            Some(CharacterState::Idle)
        } else if defend_timer > 10.0 && enemy_distance > 12.0 {
            // Defended long enough and enemy is at medium distance
            Some(CharacterState::Idle)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(super) struct EscapeState;

impl State<CharacterState> for EscapeState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("escape_timer", 0.0);
        context.set_float("speed", 20.0);
        context.set_vector3("color", [0.8, 0.2, 0.8].into()); // Magenta
        info!("AI fleeing!");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("escape_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("escape_timer", timer);
    }

    fn on_exit(&mut self, _context: &mut StateContext) {}

    fn check_transitions(&self, context: &StateContext) -> Option<CharacterState> {
        let health = context.get_float("health").unwrap_or(100.0);
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);

        if enemy_distance > 25.0 && health > 40.0 {
            Some(CharacterState::Idle)
        } else if health > 70.0 && enemy_distance < 15.0 {
            Some(CharacterState::Attack)
        } else {
            None
        }
    }
}
