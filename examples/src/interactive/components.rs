use crate::behaviour::*;
use gears_app::prelude::*;
use std::time::{Duration, Instant};

#[derive(Component, Debug)]
pub(super) struct Npc {
    pub fsm: FiniteStateMachine<CharacterState>,
    pub pathfinding_behavior: PathfindingBehavior,
    pub target_entity: Option<Entity>,
    #[allow(unused)]
    pub last_behavior_change: Instant,
    #[allow(unused)]
    pub behavior_change_interval: Duration,
}

impl Npc {
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
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub(super) struct IntelligentAIMarker;
