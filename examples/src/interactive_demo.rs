use cgmath::{InnerSpace, Rotation3, Vector3, Zero};
use gears_app::prelude::*;
use log::{LevelFilter, info};
use rand::Rng;
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

impl IntelligentAI {
    pub fn new() -> Self {
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
        // Keep base color for idle state
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
struct AttackApproachState;

impl State<CharacterState> for AttackApproachState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("speed", 12.0);
        context.set_vector3("color", [0.8, 0.4, 0.1].into()); // Orange
        info!("AI entering Attack Approach sub-state");
    }

    fn on_update(&mut self, _context: &mut StateContext, _dt: Duration) {
        // Movement handled by pathfinding
    }

    fn check_transitions(&self, context: &StateContext) -> Option<CharacterState> {
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);

        if enemy_distance < 5.0 {
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
struct AttackRetreatState;

impl State<CharacterState> for AttackRetreatState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("retreat_timer", 0.0);
        context.set_float("speed", 10.0);
        context.set_vector3("color", [0.6, 0.2, 0.2].into()); // Dark red
        info!("AI retreating after strike");
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
struct DefendState;

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

        if health < 30.0 {
            Some(CharacterState::Escape)
        } else if health > 70.0 && enemy_distance < 10.0 {
            Some(CharacterState::Attack)
        } else if enemy_distance > 20.0 {
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
        context.set_vector3("color", [0.8, 0.2, 0.8].into()); // Magenta
        info!("AI fleeing!");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("escape_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("escape_timer", timer);
    }

    fn on_exit(&mut self, _context: &mut StateContext) {
        info!("AI stopped fleeing");
    }

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

// ================================
// MAIN DEMO
// ================================

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

    let mut app = GearsApp::new(Config::default().with_crosshair_enabled(true));

    // Define color palette of available models
    let colors = ["222e50", "007991", "439a86", "bcd8c1", "e9d985"];

    // Custom UI window
    let (w1_frame_tx, w1_frame_rx) = mpsc::channel::<Dt>();

    app.add_window(Box::new(move |ui| {
        egui::Window::new("Interactive Demo")
            .default_open(true)
            .max_width(300.0)
            .max_height(750.0)
            .resizable(true)
            .anchor(egui::Align2::RIGHT_TOP, [0.0, 0.0])
            .show(ui, |ui| {
                if let Ok(dt) = w1_frame_rx.try_recv() {
                    ui.label(format!("Frame time: {:.2} ms", dt.as_secs_f32() * 1000.0));
                    ui.label(format!("FPS: {:.0}", 1.0 / dt.as_secs_f32()));
                }

                ui.separator();
                ui.heading("Interactive Demo");
                ui.label("8 AI entities with FSM + A* Pathfinding");
                ui.label("20-35 random obstacles each startup!");
                ui.label("The player can shoot the enemies to reduce their health.");

                ui.separator();
                ui.label("Color-Coded AI States:");

                let draw_color = |ui: &mut egui::Ui, color: [f32; 3], label: &str| {
                    ui.horizontal(|ui| {
                        let color_rect =
                            egui::Rect::from_min_size(ui.cursor().min, egui::Vec2::new(12.0, 12.0));
                        let egui_color = egui::Color32::from_rgb(
                            (color[0] * 255.0) as u8,
                            (color[1] * 255.0) as u8,
                            (color[2] * 255.0) as u8,
                        );
                        ui.painter().rect_filled(color_rect, 2.0, egui_color);
                        ui.allocate_space(egui::Vec2::new(15.0, 12.0));
                        ui.label(label);
                    });
                };

                draw_color(ui, [0.2, 0.2, 0.8], "Base - Idle (Wandering)");
                draw_color(ui, [0.8, 0.4, 0.1], "Orange - Attack Approach");
                draw_color(ui, [1.0, 0.1, 0.1], "Red - Attack Strike");
                draw_color(ui, [0.6, 0.2, 0.2], "Dark Red - Attack Retreat");
                draw_color(ui, [0.8, 0.8, 0.2], "Yellow - Defend");
                draw_color(ui, [0.8, 0.2, 0.8], "Magenta - Escape");

                ui.separator();
                ui.label("Controls:");
                ui.label("• WASD - Move player");
                ui.label("• Mouse - Look around");
                ui.label("• Space - Jump");
                ui.label("• Left Click - Shoot");
                ui.label("• Alt - Toggle cursor grab");
                ui.label("• F1 - Toggle debug wireframes");
                ui.label("• Esc - Pause");

                ui.separator();
                ui.label("AI Behaviors:");
                ui.label("• Idle: Random wandering");
                ui.label("• Attack: Pursue and strike player");
                ui.label("• Defend: Maintain safe distance");
                ui.label("• Escape: Flee opposite direction (HP < 30)");
                ui.label("• AI use A* pathfinding to navigate");
                ui.label("• Obstacles dynamically block paths");
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
            min: Vector3::new(-100.0, -0.1, -100.0),
            max: Vector3::new(100.0, 0.1, 100.0),
        }),
        Pos3::new(Vector3::new(0.0, -1.0, 0.0)),
        ModelSource::Obj("models/plane/plane.obj"),
    );

    // Wall 1
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Wall 1"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -50.0),
            max: cgmath::Vector3::new(1.0, 2.0, 50.0),
        }),
        Pos3::new(Vector3::new(-51.0, 1.0, 0.0)),
        ModelSource::Obj("models/wall/wall.obj")
    );

    // Wall 2
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Wall 2"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -50.0),
            max: cgmath::Vector3::new(1.0, 2.0, 50.0),
        }),
        Pos3::new_with_rot(
            Vector3::new(0.0, 1.0, -51.0),
            Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0)), // Rotate wall 90 degrees around Y-axis
        ),
        ModelSource::Obj("models/wall/wall.obj")
    );

    // Wall 3
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Wall 3"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -50.0),
            max: cgmath::Vector3::new(1.0, 2.0, 50.0),
        }),
        Pos3::new(Vector3::new(51.0, 1.0, 0.0)),
        ModelSource::Obj("models/wall/wall.obj")
    );

    // Wall 4
    new_entity!(
        app,
        RigidBodyMarker,
        ObstacleMarker,
        Name("Wall 4"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -2.0, -50.0),
            max: cgmath::Vector3::new(1.0, 2.0, 50.0),
        }),
        Pos3::new_with_rot(
            Vector3::new(0.0, 1.0, 51.0),
            Rotation3::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(90.0)), // Rotate wall 90 degrees around Y-axis
        ),
        ModelSource::Obj("models/wall/wall.obj")
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
        Weapon::new(15.0),
    );

    // Generate random obstacles
    let mut rng = rand::rng();
    let num_obstacles = rng.random_range(20..35);

    for i in 0..num_obstacles {
        // Generate random position within a reasonable range
        let x: f32 = rng.random_range(-40.0..40.0);
        let z: f32 = rng.random_range(-40.0..40.0);

        // Avoid spawning too close to center where player starts
        if x.abs() < 5.0 && z.abs() < 5.0 {
            continue;
        }

        new_entity!(
            app,
            RigidBodyMarker,
            ObstacleMarker,
            Name(Box::leak(format!("Obstacle_{}", i + 1).into_boxed_str())),
            Pos3::new(Vector3::new(x, 1.0, z)),
            RigidBody::new_static(AABBCollisionBox {
                min: Vector3::new(-1.0, -1.0, -1.0),
                max: Vector3::new(1.0, 1.0, 1.0),
            }),
            ModelSource::Obj("models/cube/cube.obj"),
        );
    }

    // Create 8 AI entities with colors from the palette
    let positions = vec![
        Vector3::new(-30.0, 1.5, -30.0),
        Vector3::new(30.0, 1.5, -30.0),
        Vector3::new(-30.0, 1.5, 30.0),
        Vector3::new(30.0, 1.5, 30.0),
        Vector3::new(0.0, 1.5, 35.0),
        Vector3::new(35.0, 1.5, 0.0),
        Vector3::new(-35.0, 1.5, 0.0),
        Vector3::new(0.0, 1.5, -35.0),
    ];

    for (i, pos) in positions.into_iter().enumerate() {
        let color_name = colors[i % colors.len()];
        let mut intelligent_ai = IntelligentAI::new();
        intelligent_ai.target_entity = Some(player);

        let pathfinding = PathfindingComponent::new(Vector3::new(0.0, 1.0, 0.0), 25.0, 2.0);
        let model_path = format!("models/capsule/{}/capsule.obj", color_name);

        new_entity!(
            app,
            IntelligentAIMarker,
            PathfindingFollower,
            RigidBodyMarker,
            StaticModelMarker,
            Name(Box::leak(format!("AI_{}", i + 1).into_boxed_str())),
            Pos3::new(pos),
            RigidBody::new(
                1.5,
                Vector3::zero(),
                Vector3::new(0.0, -10.0, 0.0),
                AABBCollisionBox {
                    min: Vector3::new(-1.0, -2.0, -1.0),
                    max: Vector3::new(1.0, 2.0, 1.0),
                }
            ),
            ModelSource::Obj(Box::leak(model_path.into_boxed_str())),
            intelligent_ai,
            pathfinding,
            Health::new(100.0, 100.0),
            Weapon::new(5.0),
            LightMarker,
            Light::PointColoured {
                radius: 8.0,
                intensity: 4.0,
                color: [0.5, 0.5, 0.5],
            },
        );
    }

    // AI Update System
    async_system!(
        app,
        "intelligent_ai_update",
        (w1_frame_tx, player),
        |world, dt| {
            w1_frame_tx
                .send(dt)
                .map_err(|_| SystemError::Other("Failed to send dt".into()))?;

            // Get player position
            let player_pos = if let Some(player_pos3) = world.get_component::<Pos3>(player) {
                match player_pos3.try_read() {
                    Ok(pos_guard) => pos_guard.pos,
                    Err(_) => return Ok(()),
                }
            } else {
                Vector3::new(0.0, 0.0, 0.0)
            };

            // Process each AI entity
            let ai_entities = world.get_entities_with_component::<IntelligentAIMarker>();

            for &entity in ai_entities.iter() {
                let query = ComponentQuery::new()
                    .write::<IntelligentAI>(vec![entity])
                    .read::<Pos3>(vec![entity])
                    .write::<PathfindingComponent>(vec![entity])
                    .read::<Health>(vec![entity])
                    .write::<Light>(vec![entity]);

                if let Some(resources) = world.acquire_query(query) {
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

                            // Update pathfinding behavior based on state
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

                            // Update light color based on FSM state
                            if let Some(color_vec) = ai.fsm.context().get_vector3("color") {
                                if let Light::PointColoured { color, .. } = &mut *light {
                                    color[0] = color_vec.x;
                                    color[1] = color_vec.y;
                                    color[2] = color_vec.z;
                                }
                            }

                            // Update pathfinding target based on behavior
                            let target_pos = match ai.pathfinding_behavior {
                                PathfindingBehavior::Pursue => player_pos,
                                PathfindingBehavior::Maintain => {
                                    let direction_away = (current_pos.pos - player_pos).normalize();
                                    let safe_distance = 10.0;
                                    player_pos + direction_away * safe_distance
                                }
                                PathfindingBehavior::Flee => {
                                    // Calculate direction away from player
                                    let direction_away = (current_pos.pos - player_pos).normalize();
                                    let flee_distance = 40.0;

                                    // Target a position far from the player in the opposite direction
                                    let flee_target =
                                        current_pos.pos + direction_away * flee_distance;

                                    // Clamp to bounds to keep within the play area
                                    Vector3::new(
                                        flee_target.x.clamp(-45.0, 45.0),
                                        current_pos.pos.y,
                                        flee_target.z.clamp(-45.0, 45.0),
                                    )
                                }
                                PathfindingBehavior::Wander => {
                                    let wander_radius = 20.0;
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
                }
            }

            Ok(())
        }
    );

    // Pathfinding System
    async_system!(app, "pathfinding_update", |world, dt| {
        // Collect all entities marked as obstacles
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

        // Process each pathfinding follower
        let follower_entities = world.get_entities_with_component::<PathfindingFollower>();

        for &entity in follower_entities.iter() {
            // Check if we need a new path
            let needs_new_path = {
                let query = ComponentQuery::new()
                    .write::<PathfindingComponent>(vec![entity])
                    .read::<Pos3>(vec![entity]);

                if let Some(resources) = world.acquire_query(query) {
                    if let (Some(pathfinding_comp), Some(pos3_comp)) = (
                        resources.get::<PathfindingComponent>(entity),
                        resources.get::<Pos3>(entity),
                    ) {
                        if let (Ok(mut pathfinding), Ok(current_pos)) =
                            (pathfinding_comp.write(), pos3_comp.read())
                        {
                            pathfinding.update(dt.as_secs_f32());
                            pathfinding.needs_pathfinding(current_pos.pos)
                                && pathfinding.should_recalculate_path()
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            // Calculate new path if needed
            if needs_new_path {
                let query = ComponentQuery::new()
                    .write::<PathfindingComponent>(vec![entity])
                    .read::<Pos3>(vec![entity]);

                if let Some(resources) = world.acquire_query(query) {
                    if let (Some(pathfinding_comp), Some(pos3_comp)) = (
                        resources.get::<PathfindingComponent>(entity),
                        resources.get::<Pos3>(entity),
                    ) {
                        if let (Ok(current_pos), Ok(mut pathfinding)) =
                            (pos3_comp.read(), pathfinding_comp.write())
                        {
                            let mut astar = AStar::new(2.0, DistanceHeuristic::Manhattan);
                            astar.build_grid_from_entities(
                                obstacles.iter().map(|(pos, cb)| (pos, cb)),
                            );

                            if let Some(path) = astar.find_path(current_pos.pos, pathfinding.target)
                            {
                                pathfinding.set_path(path);
                            }
                        }
                    }
                }
            }

            // Move along path
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
                        let should_advance = if let Some(waypoint) = pathfinding.current_waypoint()
                        {
                            let mut direction = waypoint - pos3.pos;
                            direction.y = 0.0;

                            if direction.magnitude() > pathfinding.waypoint_threshold {
                                let normalized_dir = direction.normalize();
                                let target_acceleration = normalized_dir * pathfinding.speed * 10.0;

                                rigidbody.acceleration.x = target_acceleration.x;
                                rigidbody.acceleration.z = target_acceleration.z;
                                rigidbody.velocity.x *= 0.85;
                                rigidbody.velocity.z *= 0.85;
                                false
                            } else {
                                rigidbody.acceleration.x = 0.0;
                                rigidbody.acceleration.z = 0.0;
                                rigidbody.velocity.x *= 0.5;
                                rigidbody.velocity.z *= 0.5;
                                true
                            }
                        } else {
                            rigidbody.acceleration.x = 0.0;
                            rigidbody.acceleration.z = 0.0;
                            rigidbody.velocity.x *= 0.8;
                            rigidbody.velocity.z *= 0.8;
                            false
                        };

                        if should_advance {
                            pathfinding.advance_waypoint();
                        }
                    }
                }
            }
        }

        Ok(())
    });

    // Intent processing system for shooting
    let intent_receiver = app.clone_intent_receiver();
    let last_shot = Arc::new(Mutex::new(Instant::now() - Duration::from_secs(10)));
    let cooldown = Duration::from_millis(200);

    async_system!(
        app,
        "process_shooting_intents",
        (player, intent_receiver, last_shot, cooldown),
        |world, _dt| {
            for intent in intent_receiver.iter() {
                if let Intent::Shoot { entity: _ } = intent {
                    let now = Instant::now();
                    let mut last_time = last_shot.lock().unwrap();
                    if now.duration_since(*last_time) < cooldown {
                        continue;
                    }
                    *last_time = now;
                    drop(last_time);

                    // Get all AI entities to check for hits
                    let ai_entities = world.get_entities_with_component::<IntelligentAIMarker>();

                    // Get player shooting info
                    if let (
                        Some(player_pos_comp),
                        Some(player_view_comp),
                        Some(player_weapon_comp),
                    ) = (
                        world.get_component::<Pos3>(player),
                        world.get_component::<ViewController>(player),
                        world.get_component::<Weapon>(player),
                    ) {
                        if let (Ok(player_pos), Ok(player_view), Ok(weapon)) = (
                            player_pos_comp.read(),
                            player_view_comp.read(),
                            player_weapon_comp.read(),
                        ) {
                            // Check each AI for hits
                            for &ai_entity in ai_entities.iter() {
                                let query = ComponentQuery::new()
                                    .read::<Pos3>(vec![ai_entity])
                                    .read::<RigidBody<AABBCollisionBox>>(vec![ai_entity])
                                    .write::<Health>(vec![ai_entity])
                                    .read::<Name>(vec![ai_entity]);

                                if let Some(resources) = world.acquire_query(query) {
                                    if let (
                                        Some(ai_pos_comp),
                                        Some(ai_body_comp),
                                        Some(ai_health_comp),
                                        Some(ai_name_comp),
                                    ) = (
                                        resources.get::<Pos3>(ai_entity),
                                        resources.get::<RigidBody<AABBCollisionBox>>(ai_entity),
                                        resources.get::<Health>(ai_entity),
                                        resources.get::<Name>(ai_entity),
                                    ) {
                                        if let (
                                            Ok(ai_pos),
                                            Ok(ai_body),
                                            Ok(mut ai_health),
                                            Ok(ai_name),
                                        ) = (
                                            ai_pos_comp.read(),
                                            ai_body_comp.read(),
                                            ai_health_comp.write(),
                                            ai_name_comp.read(),
                                        ) {
                                            let hit = weapon.shoot(
                                                &player_pos,
                                                &player_view,
                                                &ai_pos,
                                                &ai_body,
                                                &mut ai_health,
                                            );

                                            if hit {
                                                info!(
                                                    "HIT {}! HP: {:.1}/{:.1}",
                                                    ai_name.0,
                                                    ai_health.get_health(),
                                                    ai_health.get_max_health()
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Ok(())
        }
    );

    // Run the application
    app.run()
}
