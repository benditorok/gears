// Intelligent AI System - Combines A* Pathfinding with Hierarchical FSM Behavioral States
// This system demonstrates sophisticated AI enemies that use A* pathfinding for navigation
// while exhibiting complex behavioral states with physics-based movement.

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

                ui.separator();
                ui.label("AI Behavior:");
                ui.label("- Health > 60% & Close → Attack");
                ui.label("- Health ≤ 60% & Very Close → Defend");
                ui.label("- Health < 30% → Escape");
                ui.label("- Far from player → Idle/Wander");
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
        PathfindingTarget, // Mark player as pathfinding target for A* system
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
            ObstacleMarker, // Mark as obstacle for A* pathfinding
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

    // Create pathfinding component that will use A* algorithm
    let pathfinding = PathfindingComponent::new(
        Vector3::new(0.0, 1.0, 0.0), // Initial target
        25.0,                        // Movement speed
        2.0,                         // Grid cell size
    );

    let ai_enemy = new_entity!(
        app,
        IntelligentAIMarker,
        PathfindingFollower, // Mark as pathfinding follower for A* system
        RigidBodyMarker,     // Mark as physics object for proper rendering
        StaticModelMarker,   // Mark for rendering
        Name("Intelligent AI Enemy"),
        Pos3::new(Vector3::new(15.0, 1.5, 0.0)),
        RigidBody::new(
            1.5,                           // mass
            Vector3::zero(),               // velocity
            Vector3::new(0.0, -20.0, 0.0), // acceleration (gravity)
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

    // Intelligent AI Update System - Integrates FSM with A* Pathfinding and Physics
    async_system!(
        app,
        "intelligent_ai_update",
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
                player_pos3.read().unwrap().pos
            } else {
                Vector3::new(0.0, 0.0, 0.0)
            };

            // Update intelligent AI entities
            let entities_with_ai = world.get_entities_with_component::<IntelligentAIMarker>();
            for &entity in entities_with_ai.iter() {
                if let (
                    Some(ai_component),
                    Some(pos3_component),
                    Some(pathfinding_component),
                    Some(health_component),
                    Some(light_component),
                ) = (
                    world.get_component::<IntelligentAI>(entity),
                    world.get_component::<Pos3>(entity),
                    world.get_component::<PathfindingComponent>(entity),
                    world.get_component::<Health>(entity),
                    world.get_component::<Light>(entity),
                ) {
                    let mut ai = ai_component.write().unwrap();
                    let current_pos = pos3_component.read().unwrap().pos;
                    let mut pathfinding = pathfinding_component.write().unwrap();
                    let health = health_component.read().unwrap();
                    let mut light = light_component.write().unwrap();

                    // Calculate distance to target
                    let distance_to_player = (player_pos - current_pos).magnitude();

                    // Update FSM context with current game state
                    ai.fsm
                        .context_mut()
                        .set_float("health", health.get_health());
                    ai.fsm
                        .context_mut()
                        .set_float("enemy_distance", distance_to_player);

                    // Update FSM state machine
                    ai.fsm.update(dt);

                    // Get current FSM state and determine pathfinding behavior
                    let current_state = ai.fsm.current_state().unwrap_or(CharacterState::Idle);
                    let current_sub_state = ai.fsm.current_sub_state();

                    // Update pathfinding behavior based on FSM state
                    ai.pathfinding_behavior = match current_state {
                        CharacterState::Attack => match current_sub_state {
                            Some(CharacterState::AttackApproach) => PathfindingBehavior::Pursue,
                            Some(CharacterState::AttackStrike) => PathfindingBehavior::Pursue,
                            Some(CharacterState::AttackRetreat) => PathfindingBehavior::Maintain,
                            _ => PathfindingBehavior::Pursue,
                        },
                        CharacterState::Defend => PathfindingBehavior::Maintain,
                        CharacterState::Escape => PathfindingBehavior::Flee,
                        CharacterState::Idle => PathfindingBehavior::Wander,
                        _ => PathfindingBehavior::Wander,
                    };

                    // Update debug UI information and light color
                    {
                        *w1_ai_state.lock().unwrap() = current_state;
                        *w1_ai_sub_state.lock().unwrap() = current_sub_state;
                        *w1_pathfind_behavior.lock().unwrap() = ai.pathfinding_behavior;

                        if let Some(color_vec) = ai.fsm.context().get_vector3("color") {
                            let mut debug_color = w1_ai_color.lock().unwrap();
                            debug_color[0] = color_vec.x;
                            debug_color[1] = color_vec.y;
                            debug_color[2] = color_vec.z;

                            // Update the entity's light color to match FSM state
                            if let Light::PointColoured { color, .. } = &mut *light {
                                color[0] = color_vec.x;
                                color[1] = color_vec.y;
                                color[2] = color_vec.z;
                            }
                        }
                    }

                    // Update pathfinding target based on FSM behavior
                    let target_pos = match ai.pathfinding_behavior {
                        PathfindingBehavior::Pursue => {
                            // Move towards player using A* pathfinding
                            player_pos
                        }
                        PathfindingBehavior::Maintain => {
                            // Maintain safe distance (8-12 units from player)
                            let direction_away = (current_pos - player_pos).normalize();
                            let safe_distance = 10.0;
                            player_pos + direction_away * safe_distance
                        }
                        PathfindingBehavior::Flee => {
                            // Move away from player using A* to find escape routes
                            let direction_away = (current_pos - player_pos).normalize();
                            let flee_distance = 30.0;
                            current_pos + direction_away * flee_distance
                        }
                        PathfindingBehavior::Wander => {
                            // Random wandering around current position
                            let wander_radius = 15.0;
                            let random_angle = ai.fsm.context().time_in_state.as_secs_f32() * 0.3;
                            Vector3::new(
                                current_pos.x + random_angle.cos() * wander_radius,
                                current_pos.y,
                                current_pos.z + random_angle.sin() * wander_radius,
                            )
                        }
                        PathfindingBehavior::Guard => {
                            // Stay in area (not implemented in this demo)
                            current_pos
                        }
                    };

                    // Update pathfinding component target (A* system will handle path calculation)
                    pathfinding.set_target(target_pos);

                    // Update speed based on FSM context
                    let speed = ai.fsm.context().get_float("speed").unwrap_or(2.0);
                    pathfinding.speed = speed;
                }
            }

            Ok(())
        }
    );

    // Add pathfinding system from pathfinding example (handles A* calculation and movement)
    async_system!(app, "pathfinding_update", move |world, dt| {
        // Get player position (target for pathfinding)
        let player_entities = world.get_entities_with_component::<PathfindingTarget>();
        let player_pos = if let Some(&player_entity) = player_entities.first() {
            if let Some(pos3) = world.get_component::<Pos3>(player_entity) {
                let pos_guard = pos3.read().map_err(|e| {
                    SystemError::ComponentAccess(format!("Failed to read player Pos3: {}", e))
                })?;
                pos_guard.pos
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        // Pre-collect obstacle data once (performance optimization)
        let rigid_body_entities = world.get_entities_with_component::<ObstacleMarker>();
        let obstacles: Vec<(Vector3<f32>, AABBCollisionBox)> = rigid_body_entities
            .iter()
            .filter_map(|&rb_entity| {
                let pos_opt = world.get_component::<Pos3>(rb_entity);
                let rb_opt = world.get_component::<RigidBody<AABBCollisionBox>>(rb_entity);

                if let (Some(pos_comp), Some(rb_comp)) = (pos_opt, rb_opt)
                    && let (Ok(pos_guard), Ok(rb_guard)) = (pos_comp.read(), rb_comp.read())
                {
                    return Some((pos_guard.pos, rb_guard.collision_box.clone()));
                }

                None
            })
            .collect();

        // Update all pathfinding followers
        let follower_entities = world.get_entities_with_component::<PathfindingFollower>();
        let mut entities_needing_paths = Vec::new();

        // First pass: update components and collect entities needing path recalculation
        for &entity in follower_entities.iter() {
            let pathfinding_comp = match world.get_component::<PathfindingComponent>(entity) {
                Some(comp) => comp,
                None => continue,
            };

            let pos3_comp = match world.get_component::<Pos3>(entity) {
                Some(comp) => comp,
                None => continue,
            };

            // Update pathfinding component
            {
                let mut pathfinding = pathfinding_comp.write().map_err(|e| {
                    SystemError::ComponentAccess(format!(
                        "Failed to write PathfindingComponent: {}",
                        e
                    ))
                })?;

                pathfinding.update(dt.as_secs_f32());

                // Check if we need pathfinding and should recalculate
                let current_pos = {
                    let pos3 = pos3_comp.read().map_err(|e| {
                        SystemError::ComponentAccess(format!("Failed to read Pos3: {}", e))
                    })?;
                    pos3.pos
                };

                if pathfinding.needs_pathfinding(current_pos)
                    && pathfinding.should_recalculate_path()
                {
                    entities_needing_paths.push(entity);
                }
            }
        }

        // Second pass: calculate paths for entities that need them (limit to max 1 per frame for performance)
        if let Some(&entity) = entities_needing_paths.first() {
            let pathfinding_comp = world.get_component::<PathfindingComponent>(entity).unwrap();
            let pos3_comp = world.get_component::<Pos3>(entity).unwrap();

            // Get current position and target
            let current_pos = {
                let pos3 = pos3_comp.read().map_err(|e| {
                    SystemError::ComponentAccess(format!("Failed to read Pos3: {}", e))
                })?;
                pos3.pos
            };

            let target_pos = {
                let pathfinding = pathfinding_comp.read().map_err(|e| {
                    SystemError::ComponentAccess(format!(
                        "Failed to read PathfindingComponent: {}",
                        e
                    ))
                })?;
                pathfinding.target
            };

            // Build pathfinding grid from collected obstacles (excluding this entity)
            let mut astar = AStar::new(2.0, DistanceHeuristic::Manhattan);
            astar.build_grid_from_entities(obstacles.iter().map(|(pos, cb)| (pos, cb)));

            if let Some(path) = astar.find_path(current_pos, target_pos) {
                let mut pathfinding = pathfinding_comp.write().map_err(|e| {
                    SystemError::ComponentAccess(format!(
                        "Failed to write PathfindingComponent: {}",
                        e
                    ))
                })?;
                pathfinding.set_path(path);
            }
        }

        // Third pass: move entities along their paths using physics
        for &entity in follower_entities.iter() {
            let pathfinding_comp = match world.get_component::<PathfindingComponent>(entity) {
                Some(comp) => comp,
                None => continue,
            };

            let pos3_comp = match world.get_component::<Pos3>(entity) {
                Some(comp) => comp,
                None => continue,
            };

            let rigidbody_comp = match world.get_component::<RigidBody<AABBCollisionBox>>(entity) {
                Some(comp) => comp,
                None => continue,
            };

            // Move along the current path using physics
            {
                let pathfinding = pathfinding_comp.read().map_err(|e| {
                    SystemError::ComponentAccess(format!(
                        "Failed to read PathfindingComponent: {}",
                        e
                    ))
                })?;

                let pos3 = pos3_comp.read().map_err(|e| {
                    SystemError::ComponentAccess(format!("Failed to read Pos3: {}", e))
                })?;

                let mut rigidbody = rigidbody_comp.write().map_err(|e| {
                    SystemError::ComponentAccess(format!("Failed to write RigidBody: {}", e))
                })?;

                if let Some(waypoint) = pathfinding.current_waypoint() {
                    // Calculate direction to waypoint (only horizontal movement for pathfinding)
                    let mut direction = waypoint - pos3.pos;
                    direction.y = 0.0; // Don't try to move vertically through pathfinding

                    if direction.magnitude() > pathfinding.waypoint_threshold {
                        // Apply horizontal acceleration toward target
                        let normalized_dir = direction.normalize();
                        let target_acceleration = normalized_dir * pathfinding.speed * 12.0; // Multiply for stronger acceleration

                        // Keep gravity (y-component of acceleration) and add horizontal movement
                        rigidbody.acceleration.x = target_acceleration.x;
                        rigidbody.acceleration.z = target_acceleration.z;
                        // Leave rigidbody.acceleration.y unchanged (gravity)

                        // Apply some damping to horizontal velocity to prevent overshooting
                        rigidbody.velocity.x *= 0.85;
                        rigidbody.velocity.z *= 0.85;
                    } else {
                        // Reached waypoint, advance to next
                        // Stop horizontal movement
                        rigidbody.acceleration.x = 0.0;
                        rigidbody.acceleration.z = 0.0;
                        rigidbody.velocity.x *= 0.5; // Brake
                        rigidbody.velocity.z *= 0.5; // Brake

                        drop(pathfinding); // Release read lock
                        let mut pathfinding_mut = pathfinding_comp.write().map_err(|e| {
                            SystemError::ComponentAccess(format!(
                                "Failed to write PathfindingComponent: {}",
                                e
                            ))
                        })?;
                        pathfinding_mut.advance_waypoint();
                    }
                } else {
                    // No waypoint, stop horizontal movement
                    rigidbody.acceleration.x = 0.0;
                    rigidbody.acceleration.z = 0.0;
                    rigidbody.velocity.x *= 0.8; // Gradual stop
                    rigidbody.velocity.z *= 0.8; // Gradual stop
                }
            }
        }

        Ok(())
    });

    // Run the application
    app.run().await
}
