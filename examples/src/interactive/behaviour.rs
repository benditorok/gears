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
#[allow(unused)]
pub(super) enum PathfindingBehavior {
    Pursue,   // Move towards target using A*
    Maintain, // Keep safe distance from target
    Flee,     // Move away from target
    Wander,   // Random exploration
    Guard,    // Stay in area
}

/// Generate a wander target that is at least `min_distance` away from current position
fn generate_distant_wander_target(
    current_pos: cgmath::Vector3<f32>,
    min_distance: f32,
) -> cgmath::Vector3<f32> {
    let mut rng = rand::rng();

    // Try up to 10 times to find a point far enough away (XZ distance only)
    for _ in 0..10 {
        let wander_x = rng.random_range(-40.0..40.0);
        let wander_z = rng.random_range(-40.0..40.0);

        // Calculate XZ distance only (ignore Y for ground movement)
        let dx = wander_x - current_pos.x;
        let dz = wander_z - current_pos.z;
        let distance_xz = (dx * dx + dz * dz).sqrt();

        if distance_xz >= min_distance {
            // Use current Y position for wander target
            return cgmath::Vector3::new(wander_x, current_pos.y, wander_z);
        }
    }

    // Pick a point in a random direction at min_distance as a fallback
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let offset_x = angle.cos() * min_distance;
    let offset_z = angle.sin() * min_distance;
    cgmath::Vector3::new(
        (current_pos.x + offset_x).clamp(-40.0, 40.0),
        current_pos.y, // Use current Y position
        (current_pos.z + offset_z).clamp(-40.0, 40.0),
    )
}

#[derive(Debug)]
pub(super) struct IdleState;

impl State<CharacterState> for IdleState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("idle_timer", 0.0);
        context.set_float("wander_timer", 0.0);
        context.set_vector3("color", [0.2, 0.2, 0.8].into()); // Blue for Idle

        // Generate initial random wander point within map bounds (-40 to 40)
        // ensuring it's at least 15 units away from current position
        let current_pos = context
            .get_vector3("current_position")
            .unwrap_or(cgmath::Vector3::new(0.0, 1.0, 0.0));
        let wander_target = generate_distant_wander_target(current_pos, 15.0);
        context.set_vector3("wander_target", wander_target);

        // Randomize wander interval (3-6 seconds) and speed (10-14)
        let mut rng = rand::rng();
        let wander_interval = rng.random_range(3.0..6.0);
        let wander_speed = rng.random_range(10.0..14.0);
        context.set_float("wander_interval", wander_interval);
        context.set_float("speed", wander_speed);
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("idle_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("idle_timer", timer);

        let wander_timer = context.get_float("wander_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("wander_timer", wander_timer);

        // Generate new wander point at randomized intervals
        let wander_interval = context.get_float("wander_interval").unwrap_or(4.0);
        if wander_timer > wander_interval {
            // Get current position and generate a distant wander target
            let current_pos = context
                .get_vector3("current_position")
                .unwrap_or(cgmath::Vector3::new(0.0, 1.0, 0.0));
            let wander_target = generate_distant_wander_target(current_pos, 15.0);
            context.set_vector3("wander_target", wander_target);
            context.set_float("wander_timer", 0.0);

            // Randomize next interval and speed for variety
            let mut rng = rand::rng();
            let new_interval = rng.random_range(3.0..6.0);
            let new_speed = rng.random_range(10.0..14.0);
            context.set_float("wander_interval", new_interval);
            context.set_float("speed", new_speed);
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
        context.set_float("speed", 18.0);
        context.set_vector3("color", [0.8, 0.4, 0.1].into()); // Orange
        info!("NPC entered Attack state");
    }

    fn on_update(&mut self, _context: &mut StateContext, _dt: Duration) {
        // Attack logic handled by sub-states
    }

    fn on_exit(&mut self, _context: &mut StateContext) {
        info!("NPC exiting Attack state");
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
        context.set_float("speed", 18.0);
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
        context.set_float("speed", 4.0);
        context.set_vector3("color", [1.0, 0.1, 0.1].into()); // Bright red
        info!("NPC striking!");
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
        context.set_float("speed", 16.0);
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
        context.set_float("speed", 14.0);
        context.set_vector3("color", [0.8, 0.8, 0.2].into()); // Yellow
        info!("NPC entered Defend state");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("defend_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("defend_timer", timer);
    }

    fn on_exit(&mut self, _context: &mut StateContext) {
        info!("NPC exiting Defend state");
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
        context.set_float("speed", 28.0);
        context.set_vector3("color", [0.8, 0.2, 0.8].into()); // Magenta
        info!("NPC fleeing!");
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
