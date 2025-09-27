// Intelligent AI System - Fixed version using Query System to prevent deadlocks
// This system demonstrates sophisticated AI enemies that use A* pathfinding for navigation
// while exhibiting complex behavioral states with physics-based movement.
// The key difference is using the query system to prevent resource starvation.

use cgmath::{InnerSpace, Vector3, Zero};
use gears_app::prelude::*;
use log::{LevelFilter, info};
use std::f32::consts::PI;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

// ================================
// BEHAVIORAL FSM SYSTEM
// ================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterState {
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
pub enum PathfindingBehavior {
    Pursue,   // Move towards target using A*
    Maintain, // Keep safe distance from target
    Flee,     // Move away from target
    Wander,   // Random exploration
    Guard,    // Stay in area
}

// ================================
// INTEGRATED COMPONENTS
// ================================

#[derive(Component, Debug)]
pub struct IntelligentAI {
    pub fsm: FiniteStateMachine<CharacterState>,
    pub pathfinding_behavior: PathfindingBehavior,
    pub target_entity: Option<Entity>,
    pub last_behavior_change: Instant,
    pub behavior_change_interval: Duration,
}

impl Default for IntelligentAI {
    fn default() -> Self {
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
            .set_vector3("color", [0.2, 0.2, 0.8].into());

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
pub struct IntelligentAIMarker;

// ================================
// FSM STATE IMPLEMENTATIONS
// ================================

#[derive(Debug)]
struct IdleState;

impl State<CharacterState> for IdleState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("idle_timer", 0.0);
        context.set_float("speed", 8.0);
        context.set_vector3("color", [0.2, 0.2, 0.8].into()); // Blue
        info!("AI entered Idle state");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("idle_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("idle_timer", timer);
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
struct AttackState;

impl State<CharacterState> for AttackState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("attack_timer", 0.0);
        context.set_bool("attacking", true);
        info!("AI entered Attack state");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("attack_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("attack_timer", timer);
    }

    fn on_exit(&mut self, context: &mut StateContext) {
        context.set_bool("attacking", false);
    }

    fn check_transitions(&self, context: &StateContext) -> Option<CharacterState> {
        let health = context.get_float("health").unwrap_or(100.0);
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);
        let timer = context.get_float("attack_timer").unwrap_or(0.0);

        if health < 30.0 {
            Some(CharacterState::Escape)
        } else if enemy_distance > 20.0 || timer > 8.0 {
            Some(CharacterState::Idle)
        } else if health < 60.0 && enemy_distance < 5.0 {
            Some(CharacterState::Defend)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct AttackApproachState;

impl State<CharacterState> for AttackApproachState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("approach_timer", 0.0);
        context.set_float("speed", 15.0);
        context.set_vector3("color", [0.8, 0.4, 0.1].into()); // Orange
        info!("AI entered Attack > Approach sub-state");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("approach_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("approach_timer", timer);
    }

    fn check_transitions(&self, context: &StateContext) -> Option<CharacterState> {
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);
        let timer = context.get_float("approach_timer").unwrap_or(0.0);

        if enemy_distance < 3.0 || timer > 2.0 {
            Some(CharacterState::AttackStrike)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct AttackStrikeState;

impl State<CharacterState> for AttackStrikeState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("strike_timer", 0.0);
        context.set_float("speed", 5.0);
        context.set_vector3("color", [1.0, 0.1, 0.1].into()); // Bright Red
        info!("AI entered Attack > Strike sub-state");
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
struct AttackRetreatState;

impl State<CharacterState> for AttackRetreatState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("retreat_timer", 0.0);
        context.set_float("speed", 10.0);
        context.set_vector3("color", [0.6, 0.2, 0.2].into()); // Dark Red
        info!("AI entered Attack > Retreat sub-state");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("retreat_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("retreat_timer", timer);
    }

    fn check_transitions(&self, context: &StateContext) -> Option<CharacterState> {
        let timer = context.get_float("retreat_timer").unwrap_or(0.0);
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);

        if timer > 1.5 && enemy_distance > 5.0 {
            Some(CharacterState::AttackApproach)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct DefendState;

impl State<CharacterState> for DefendState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("defend_timer", 0.0);
        context.set_float("speed", 6.0);
        context.set_bool("defending", true);
        context.set_vector3("color", [0.8, 0.8, 0.2].into()); // Yellow
        info!("AI entered Defend state");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("defend_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("defend_timer", timer);
    }

    fn on_exit(&mut self, context: &mut StateContext) {
        context.set_bool("defending", false);
    }

    fn check_transitions(&self, context: &StateContext) -> Option<CharacterState> {
        let health = context.get_float("health").unwrap_or(100.0);
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);
        let timer = context.get_float("defend_timer").unwrap_or(0.0);

        if health < 30.0 {
            Some(CharacterState::Escape)
        } else if health > 80.0 && enemy_distance < 10.0 {
            Some(CharacterState::Attack)
        } else if enemy_distance > 15.0 || timer > 4.0 {
            Some(CharacterState::Idle)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct EscapeState;

impl State<CharacterState> for EscapeState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("escape_timer", 0.0);
        context.set_float("speed", 20.0);
        context.set_bool("escaping", true);
        context.set_vector3("color", [0.8, 0.2, 0.8].into()); // Purple
        info!("AI entered Escape state");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("escape_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("escape_timer", timer);
    }

    fn on_exit(&mut self, context: &mut StateContext) {
        context.set_bool("escaping", false);
    }

    fn check_transitions(&self, context: &StateContext) -> Option<CharacterState> {
        let health = context.get_float("health").unwrap_or(100.0);
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);
        let timer = context.get_float("escape_timer").unwrap_or(0.0);

        if health > 60.0 && enemy_distance > 25.0 {
            Some(CharacterState::Idle)
        } else if health > 40.0 && enemy_distance > 15.0 && timer > 2.0 {
            Some(CharacterState::Defend)
        } else {
            None
        }
    }
}

#[tokio::main]
async fn main() -> EngineResult<()> {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("{}", info);
        println!("Press Enter to close...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
    }));

    // Initialize the logger
    let mut env_builder = env_logger::Builder::new();
    env_builder.filter_level(LevelFilter::Info);
    env_builder.filter_module("wgpu_core::device::resource", log::LevelFilter::Warn);
    env_builder.init();

    let mut app = GearsApp::default();

    // Custom UI window for displaying AI information
    let (w1_frame_tx, w1_frame_rx) = mpsc::channel::<Dt>();
    let w1_ai_state = Arc::new(Mutex::new(CharacterState::Idle));
    let w1_ai_sub_state = Arc::new(Mutex::new(Option::None));
    let w1_ai_color = Arc::new(Mutex::new([0.2f32, 0.2f32, 0.8f32]));
    let w1_pathfind_behavior = Arc::new(Mutex::new(PathfindingBehavior::Wander));

    let ai_state = w1_ai_state.clone();
    let ai_sub_state = w1_ai_sub_state.clone();
    let ai_color = w1_ai_color.clone();
    let pathfind_behavior = w1_pathfind_behavior.clone();

    app.add_window(Box::new(move |ui| {
        egui::Window::new("Intelligent AI Demo")
            .default_open(true)
            .max_width(450.0)
            .max_height(700.0)
            .default_width(400.0)
            .resizable(true)
            .anchor(egui::Align2::RIGHT_TOP, [0.0, 0.0])
            .show(ui, |ui| {
                if let Ok(dt) = w1_frame_rx.try_recv() {
                    ui.label(format!("Frame time: {:.2} ms", dt.as_secs_f32() * 1000.0));
                    ui.label(format!("FPS: {:.0}", 1.0 / dt.as_secs_f32()));
                }

                ui.separator();
                ui.heading("Intelligent AI: FSM + A* Pathfinding");

                ui.separator();
                ui.label("Visual State Indicators:");

                // Helper function to draw color rectangle and label
                let draw_state_with_color = |ui: &mut egui::Ui, color: [f32; 3], text: &str| {
                    ui.horizontal(|ui| {
                        let egui_color = egui::Color32::from_rgb(
                            (color[0] * 255.0) as u8,
                            (color[1] * 255.0) as u8,
                            (color[2] * 255.0) as u8,
                        );
                        let color_rect =
                            egui::Rect::from_min_size(ui.cursor().min, egui::Vec2::new(12.0, 12.0));
                        ui.painter().rect_filled(color_rect, 2.0, egui_color);
                        ui.allocate_space(egui::Vec2::new(15.0, 12.0));
                        ui.label(text);
                    });
                };

                draw_state_with_color(ui, [0.2, 0.2, 0.8], "- IDLE - Random wandering");
                draw_state_with_color(ui, [0.8, 0.4, 0.1], "- ATTACK - Pursuing target");
                draw_state_with_color(ui, [0.8, 0.4, 0.1], "  - APPROACH - Moving towards");
                draw_state_with_color(ui, [1.0, 0.1, 0.1], "  - STRIKE - Attacking");
                draw_state_with_color(ui, [0.6, 0.2, 0.2], "  - RETREAT - Backing away");
                draw_state_with_color(ui, [0.8, 0.8, 0.2], "- DEFEND - Maintaining distance");
                draw_state_with_color(ui, [0.8, 0.2, 0.8], "- ESCAPE - Fleeing target");

                ui.separator();

                // Current state display
                let color_rgb = ai_color.lock().unwrap();
                let egui_color = egui::Color32::from_rgb(
                    (color_rgb[0] * 255.0) as u8,
                    (color_rgb[1] * 255.0) as u8,
                    (color_rgb[2] * 255.0) as u8,
                );
                ui.horizontal(|ui| {
                    ui.heading("Current AI Status");
                    let color_rect =
                        egui::Rect::from_min_size(ui.cursor().min, egui::Vec2::new(20.0, 20.0));
                    ui.painter().rect_filled(color_rect, 2.0, egui_color);
                    ui.allocate_space(egui::Vec2::new(25.0, 20.0));
                });

                ui.label(format!("FSM State: {:?}", ai_state.lock().unwrap()));
                if let Some(sub_state) = ai_sub_state.lock().unwrap().as_ref() {
                    ui.label(format!("Sub-State: {:?}", sub_state));
                }
                ui.label(format!(
                    "Pathfinding: {:?}",
                    pathfind_behavior.lock().unwrap()
                ));

                ui.separator();
                ui.label("System Integration:");
                ui.label("- FSM states determine pathfinding behavior");
                ui.label("- A* algorithm for intelligent navigation");
                ui.label("- Physics-based movement with momentum");
                ui.label("- Obstacle avoidance and collision detection");
                ui.label("- Health and distance-based state transitions");

                ui.separator();
                ui.label("Controls:");
                ui.label("WASD - Move player");
                ui.label("Mouse - Look around");
                ui.label("Space - Jump");
                ui.label("Alt - Keep the cursor within the window's bounds.");
                ui.label("Esc - Pause");
                ui.label("F1 - Toggle debug mode");
            });
    }));

    // Add lighting
    new_entity!(
        app,
        LightMarker,
        Name("Ambient Light"),
        Light::Ambient { intensity: 0.1 },
        Pos3::new(Vector3::new(0.0, 0.0, 0.0))
    );

    new_entity!(
        app,
        LightMarker,
        Name("Directional Light"),
        Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.6,
        },
        Pos3::new(Vector3::new(30.0, 30.0, 30.0))
    );

    // Create ground plane
    new_entity!(
        app,
        RigidBodyMarker,
        Name("Ground Plane"),
        RigidBody::new_static(AABBCollisionBox {
            min: Vector3::new(-50.0, -0.1, -50.0),
            max: Vector3::new(50.0, 0.1, 50.0),
        }),
        Pos3::new(Vector3::new(0.0, -1.0, 0.0)),
        ModelSource::Obj("models/plane/plane.obj"),
    );

    // Create player
    let mut player_prefab = Player::default();
    let player = new_entity!(
        app,
        PlayerMarker,
        PathfindingTarget,
        player_prefab.pos3.take().unwrap(),
        player_prefab.model_source.take().unwrap(),
        player_prefab.movement_controller.take().unwrap(),
        player_prefab.view_controller.take().unwrap(),
        player_prefab.rigidbody.take().unwrap(),
        Health::default(),
    );

    // Create obstacles for pathfinding to navigate around
    for i in 0..8 {
        let angle = i as f32 * PI * 0.25;
        let x = angle.cos() * 20.0;
        let z = angle.sin() * 20.0;

        new_entity!(
            app,
            RigidBodyMarker,
            ObstacleMarker,
            Name("Obstacle"),
            Pos3::new(Vector3::new(x, 1.0, z)),
            RigidBody::new_static(AABBCollisionBox {
                min: Vector3::new(-1.0, -1.0, -1.0),
                max: Vector3::new(1.0, 1.0, 1.0),
            }),
            ModelSource::Obj("models/cube/cube.obj"),
        );
    }

    // Create intelligent AI enemy with integrated FSM + A* Pathfinding
    let mut intelligent_ai = IntelligentAI::default();
    intelligent_ai.target_entity = Some(player);

    let pathfinding = PathfindingComponent::new(Vector3::new(0.0, 1.0, 0.0), 25.0, 2.0);

    let ai_enemy = new_entity!(
        app,
        IntelligentAIMarker,
        PathfindingFollower,
        RigidBodyMarker,
        StaticModelMarker,
        Name("Intelligent AI Enemy"),
        Pos3::new(Vector3::new(15.0, 1.5, 0.0)),
        RigidBody::new(
            1.5,
            Vector3::zero(),
            Vector3::new(0.0, -20.0, 0.0),
            AABBCollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            }
        ),
        ModelSource::Obj("models/sphere/sphere.obj"),
        intelligent_ai,
        pathfinding,
        Health::new(100.0, 100.0),
        LightMarker,
        Light::PointColoured {
            radius: 10.0,
            intensity: 6.0,
            color: [0.2, 0.2, 0.8],
        },
    );

    // Intelligent AI Update System using Query System
    async_system!(
        app,
        "intelligent_ai_update_query",
        (
            w1_frame_tx,
            w1_ai_state,
            w1_ai_sub_state,
            w1_ai_color,
            w1_pathfind_behavior
        ),
        |world, dt| {
            w1_frame_tx
                .send(dt)
                .map_err(|_| SystemError::Other("Failed to send dt".into()))?;

            // Get player position for AI targeting
            let player_pos = if let Some(player_pos3) = world.get_component::<Pos3>(player) {
                match player_pos3.try_read() {
                    Ok(pos_guard) => pos_guard.pos,
                    Err(_) => {
                        // Skip this frame if player position is locked
                        info!("Skipping AI update - player position locked");
                        return Ok(());
                    }
                }
            } else {
                Vector3::new(0.0, 0.0, 0.0)
            };

            // Get AI entities
            let ai_entities = world.get_entities_with_component::<IntelligentAIMarker>();

            for &entity in ai_entities.iter() {
                // Build query for this AI entity - we need read/write access to multiple components
                let query = ComponentQuery::new()
                    .write::<IntelligentAI>(vec![entity])
                    .read::<Pos3>(vec![entity])
                    .write::<PathfindingComponent>(vec![entity])
                    .read::<Health>(vec![entity])
                    .write::<Light>(vec![entity]);

                // Try to acquire resources with a short timeout
                if let Some(resources) = world.acquire_query(query) {
                    // We have exclusive access to all needed components
                    if let (
                        Some(ai_component),
                        Some(pos3_component),
                        Some(pathfinding_component),
                        Some(health_component),
                        Some(light_component),
                    ) = (
                        resources.get::<IntelligentAI>(entity),
                        resources.get::<Pos3>(entity),
                        resources.get::<PathfindingComponent>(entity),
                        resources.get::<Health>(entity),
                        resources.get::<Light>(entity),
                    ) {
                        // Now we can safely work with all components
                        if let (
                            Ok(mut ai),
                            Ok(current_pos),
                            Ok(mut pathfinding),
                            Ok(health),
                            Ok(mut light),
                        ) = (
                            ai_component.write(),
                            pos3_component.read(),
                            pathfinding_component.write(),
                            health_component.read(),
                            light_component.write(),
                        ) {
                            let distance_to_player = (player_pos - current_pos.pos).magnitude();

                            // Update FSM context
                            ai.fsm
                                .context_mut()
                                .set_float("health", health.get_health());
                            ai.fsm
                                .context_mut()
                                .set_float("enemy_distance", distance_to_player);

                            // Update FSM
                            ai.fsm.update(dt);

                            // Get current states
                            let current_state =
                                ai.fsm.current_state().unwrap_or(CharacterState::Idle);
                            let current_sub_state = ai.fsm.current_sub_state();

                            // Update pathfinding behavior
                            ai.pathfinding_behavior = match current_state {
                                CharacterState::Attack => match current_sub_state {
                                    Some(CharacterState::AttackApproach) => {
                                        PathfindingBehavior::Pursue
                                    }
                                    Some(CharacterState::AttackStrike) => {
                                        PathfindingBehavior::Pursue
                                    }
                                    Some(CharacterState::AttackRetreat) => {
                                        PathfindingBehavior::Maintain
                                    }
                                    _ => PathfindingBehavior::Pursue,
                                },
                                CharacterState::Defend => PathfindingBehavior::Maintain,
                                CharacterState::Escape => PathfindingBehavior::Flee,
                                CharacterState::Idle => PathfindingBehavior::Wander,
                                _ => PathfindingBehavior::Wander,
                            };

                            // Update debug UI
                            {
                                *w1_ai_state.lock().unwrap() = current_state;
                                *w1_ai_sub_state.lock().unwrap() = current_sub_state;
                                *w1_pathfind_behavior.lock().unwrap() = ai.pathfinding_behavior;

                                if let Some(color_vec) = ai.fsm.context().get_vector3("color") {
                                    let mut debug_color = w1_ai_color.lock().unwrap();
                                    debug_color[0] = color_vec.x;
                                    debug_color[1] = color_vec.y;
                                    debug_color[2] = color_vec.z;

                                    if let Light::PointColoured { color, .. } = &mut *light {
                                        color[0] = color_vec.x;
                                        color[1] = color_vec.y;
                                        color[2] = color_vec.z;
                                    }
                                }
                            }

                            // Update pathfinding target
                            let target_pos = match ai.pathfinding_behavior {
                                PathfindingBehavior::Pursue => player_pos,
                                PathfindingBehavior::Maintain => {
                                    let direction_away = (current_pos.pos - player_pos).normalize();
                                    let safe_distance = 10.0;
                                    player_pos + direction_away * safe_distance
                                }
                                PathfindingBehavior::Flee => {
                                    let direction_away = (current_pos.pos - player_pos).normalize();
                                    let flee_distance = 30.0;
                                    current_pos.pos + direction_away * flee_distance
                                }
                                PathfindingBehavior::Wander => {
                                    let wander_radius = 15.0;
                                    let random_angle =
                                        ai.fsm.context().time_in_state.as_secs_f32() * 0.3;
                                    Vector3::new(
                                        current_pos.pos.x + random_angle.cos() * wander_radius,
                                        current_pos.pos.y,
                                        current_pos.pos.z + random_angle.sin() * wander_radius,
                                    )
                                }
                                PathfindingBehavior::Guard => current_pos.pos,
                            };

                            pathfinding.set_target(target_pos);
                            let speed = ai.fsm.context().get_float("speed").unwrap_or(2.0);
                            pathfinding.speed = speed;
                        }
                    }
                } else {
                    info!("Missing components for entity {}", *entity);
                }
            }

            Ok(())
        }
    );

    // Pathfinding System using Query System
    async_system!(app, "pathfinding_update_query", move |world, dt| {
        // Get player position
        let player_entities = world.get_entities_with_component::<PathfindingTarget>();
        let player_pos = if let Some(&player_entity) = player_entities.first() {
            if let Some(pos3) = world.get_component::<Pos3>(player_entity) {
                pos3.read().unwrap().pos
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        // Pre-collect obstacle data
        let obstacle_entities = world.get_entities_with_component::<ObstacleMarker>();
        let mut obstacles = Vec::new();

        for &obstacle_entity in obstacle_entities.iter() {
            let query = ComponentQuery::new()
                .read::<Pos3>(vec![obstacle_entity])
                .read::<RigidBody<AABBCollisionBox>>(vec![obstacle_entity]);

            if let Some(resources) = world.acquire_query(query) {
                if let (Some(pos_comp), Some(rb_comp)) = (
                    resources.get::<Pos3>(obstacle_entity),
                    resources.get::<RigidBody<AABBCollisionBox>>(obstacle_entity),
                ) {
                    if let (Ok(pos_guard), Ok(rb_guard)) = (pos_comp.read(), rb_comp.read()) {
                        obstacles.push((pos_guard.pos, rb_guard.collision_box.clone()));
                    }
                }
            }
        }

        // Update pathfinding followers
        let follower_entities = world.get_entities_with_component::<PathfindingFollower>();

        if let Some(&entity) = follower_entities.first() {
            let query = ComponentQuery::new()
                .write::<PathfindingComponent>(vec![entity])
                .read::<Pos3>(vec![entity])
                .write::<RigidBody<AABBCollisionBox>>(vec![entity]);

            if let Some(resources) = world.acquire_query(query) {
                if let (Some(pathfinding_comp), Some(pos3_comp), Some(rigidbody_comp)) = (
                    resources.get::<PathfindingComponent>(entity),
                    resources.get::<Pos3>(entity),
                    resources.get::<RigidBody<AABBCollisionBox>>(entity),
                ) {
                    if let (Ok(mut pathfinding), Ok(pos3), Ok(mut rigidbody)) = (
                        pathfinding_comp.write(),
                        pos3_comp.read(),
                        rigidbody_comp.write(),
                    ) {
                        pathfinding.update(dt.as_secs_f32());
                        let current_pos = pos3.pos;

                        // Check if we need to recalculate path
                        if pathfinding.needs_pathfinding(current_pos)
                            && pathfinding.should_recalculate_path()
                        {
                            let mut astar = AStar::new(2.0, DistanceHeuristic::Manhattan);
                            astar.build_grid_from_entities(
                                obstacles.iter().map(|(pos, cb)| (pos, cb)),
                            );

                            if let Some(path) = astar.find_path(current_pos, pathfinding.target) {
                                pathfinding.set_path(path);
                            }
                        }

                        // Move along path
                        if let Some(waypoint) = pathfinding.current_waypoint() {
                            let mut direction = waypoint - pos3.pos;
                            direction.y = 0.0;

                            if direction.magnitude() > pathfinding.waypoint_threshold {
                                let normalized_dir = direction.normalize();
                                let target_acceleration = normalized_dir * pathfinding.speed * 12.0;

                                rigidbody.acceleration.x = target_acceleration.x;
                                rigidbody.acceleration.z = target_acceleration.z;
                                rigidbody.velocity.x *= 0.85;
                                rigidbody.velocity.z *= 0.85;
                            } else {
                                rigidbody.acceleration.x = 0.0;
                                rigidbody.acceleration.z = 0.0;
                                rigidbody.velocity.x *= 0.5;
                                rigidbody.velocity.z *= 0.5;
                                pathfinding.advance_waypoint();
                            }
                        } else {
                            rigidbody.acceleration.x = 0.0;
                            rigidbody.acceleration.z = 0.0;
                            rigidbody.velocity.x *= 0.8;
                            rigidbody.velocity.z *= 0.8;
                        }
                    }
                }
            } else {
                info!("Missing components for entity {}", *entity);
            }
        }

        Ok(())
    });

    app.run().await
}
