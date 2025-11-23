use crate::behaviour::*;
use gears_app::prelude::*;
use std::time::{Duration, Instant};

#[derive(Component, Debug)]
pub(super) struct IntelligentAI {
    pub fsm: FiniteStateMachine<CharacterState>,
    pub pathfinding_behavior: PathfindingBehavior,
    pub target_entity: Option<Entity>,
    pub last_behavior_change: Instant,
    pub behavior_change_interval: Duration,
    pub last_position: cgmath::Vector3<f32>,
    pub stuck_timer: f32,
    pub stuck_threshold: f32,
}

impl IntelligentAI {
    pub(super) fn new() -> Self {
        let mut fsm = FiniteStateMachine::<CharacterState>::new();

        // Add states
        fsm.add_state(CharacterState::Idle, Box::new(IdleState));
        fsm.add_state(CharacterState::Attack, Box::new(AttackState));
        fsm.add_state(CharacterState::Defend, Box::new(DefendState));
        fsm.add_state(CharacterState::Escape, Box::new(EscapeState));

        // Add sub-states for Attack
        fsm.add_sub_state(
            CharacterState::Attack,
            CharacterState::AttackApproach,
            Box::new(AttackApproachState),
        );
        fsm.add_sub_state(
            CharacterState::Attack,
            CharacterState::AttackStrike,
            Box::new(AttackStrikeState),
        );
        fsm.add_sub_state(
            CharacterState::Attack,
            CharacterState::AttackRetreat,
            Box::new(AttackRetreatState),
        );

        fsm.set_initial_sub_state(CharacterState::Attack, CharacterState::AttackApproach);
        fsm.set_initial_state(CharacterState::Idle);

        // Initialize context
        fsm.context_mut().set_float("health", 100.0);
        fsm.context_mut().set_float("max_health", 100.0);
        fsm.context_mut().set_float("enemy_distance", 100.0);
        fsm.context_mut()
            .set_vector3("color", [0.5, 0.5, 0.5].into());

        Self {
            fsm,
            pathfinding_behavior: PathfindingBehavior::Wander,
            target_entity: None,
            last_behavior_change: Instant::now(),
            behavior_change_interval: Duration::from_millis(500),
            last_position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            stuck_timer: 0.0,
            stuck_threshold: 0.5, // If we haven't moved 0.5 units in 2 seconds, we're stuck
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub(super) struct IntelligentAIMarker;
