use cgmath::InnerSpace;
use egui::Align2;
use gears_app::systems::SystemError;
use gears_app::{prelude::*, systems};
use gears_macro::Component;
use log::{LevelFilter, info};
use std::f32::consts::PI;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Custom state enum for character FSM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterState {
    // Main states
    Idle,
    Attack,
    Defend,
    Escape,

    // Sub-states for each main state
    IdleWander,
    IdleWatch,

    AttackApproach,
    AttackStrike,
    AttackRetreat,

    DefendBlock,
    DefendCounter,

    EscapeFlee,
    EscapeHide,
}

impl StateIdentifier for CharacterState {
    fn as_str(&self) -> &'static str {
        match self {
            CharacterState::Idle => "idle",
            CharacterState::Attack => "attack",
            CharacterState::Defend => "defend",
            CharacterState::Escape => "escape",
            CharacterState::IdleWander => "idle_wander",
            CharacterState::IdleWatch => "idle_watch",
            CharacterState::AttackApproach => "attack_approach",
            CharacterState::AttackStrike => "attack_strike",
            CharacterState::AttackRetreat => "attack_retreat",
            CharacterState::DefendBlock => "defend_block",
            CharacterState::DefendCounter => "defend_counter",
            CharacterState::EscapeFlee => "escape_flee",
            CharacterState::EscapeHide => "escape_hide",
        }
    }
}

impl std::fmt::Display for CharacterState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl CharacterState {
    /// Check if this state is a main state (not a sub-state)
    pub fn is_main_state(&self) -> bool {
        matches!(
            self,
            CharacterState::Idle
                | CharacterState::Attack
                | CharacterState::Defend
                | CharacterState::Escape
        )
    }

    /// Check if this state is a sub-state of the given parent
    pub fn is_sub_state_of(&self, parent: CharacterState) -> bool {
        match parent {
            CharacterState::Idle => {
                matches!(self, CharacterState::IdleWander | CharacterState::IdleWatch)
            }
            CharacterState::Attack => matches!(
                self,
                CharacterState::AttackApproach
                    | CharacterState::AttackStrike
                    | CharacterState::AttackRetreat
            ),
            CharacterState::Defend => matches!(
                self,
                CharacterState::DefendBlock | CharacterState::DefendCounter
            ),
            CharacterState::Escape => matches!(
                self,
                CharacterState::EscapeFlee | CharacterState::EscapeHide
            ),
            _ => false,
        }
    }
}

// Character marker for entities with FSM
#[derive(Component, Debug, Clone, Copy)]
pub struct CharacterMarker;

// Simple state implementations
#[derive(Debug)]
struct IdleState;

impl State<CharacterState> for IdleState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("idle_timer", 0.0);
        context.set_float("speed", 2.0);
        info!("Character entered {} state", CharacterState::Idle);
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("idle_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("idle_timer", timer);

        // Update character color to blue (calm)
        context.set_vector3("color", [0.2, 0.2, 0.8].into());
    }

    fn check_transitions(&self, context: &StateContext) -> Option<CharacterState> {
        let health = context.get_float("health").unwrap_or(100.0);
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);
        let timer = context.get_float("idle_timer").unwrap_or(0.0);

        if health < 30.0 {
            Some(CharacterState::Escape)
        } else if enemy_distance < 15.0 && health > 60.0 {
            Some(CharacterState::Attack)
        } else if enemy_distance < 8.0 && health <= 60.0 {
            Some(CharacterState::Defend)
        } else if timer > 5.0 {
            // Return to wandering after being idle for too long
            None // Stay in idle but could transition to sub-states
        } else {
            None
        }
    }
}

// Sub-state implementations for Attack
#[derive(Debug)]
struct AttackApproachState;

impl State<CharacterState> for AttackApproachState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("approach_timer", 0.0);
        context.set_float("speed", 6.0);
        info!(
            "Character entered {} > {} sub-state",
            CharacterState::Attack,
            CharacterState::AttackApproach
        );
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("approach_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("approach_timer", timer);

        // Update character color to orange (approaching)
        context.set_vector3("color", [0.8, 0.4, 0.1].into());
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
        context.set_float("speed", 2.0);
        info!(
            "Character entered {} > {} sub-state",
            CharacterState::Attack,
            CharacterState::AttackStrike
        );
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("strike_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("strike_timer", timer);

        // Update character color to bright red (striking)
        context.set_vector3("color", [1.0, 0.1, 0.1].into());
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
        context.set_float("speed", 4.0);
        info!(
            "Character entered {} > {} sub-state",
            CharacterState::Attack,
            CharacterState::AttackRetreat
        );
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("retreat_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("retreat_timer", timer);

        // Update character color to dark red (retreating)
        context.set_vector3("color", [0.6, 0.2, 0.2].into());
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

// Main attack state that contains sub-states
#[derive(Debug)]
struct AttackState;

impl State<CharacterState> for AttackState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("attack_timer", 0.0);
        context.set_bool("attacking", true);
        info!("Character entered {} state", CharacterState::Attack);
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("attack_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("attack_timer", timer);
    }

    fn on_exit(&mut self, context: &mut StateContext) {
        context.set_bool("attacking", false);
        info!("Character exited {} state", CharacterState::Attack);
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
struct DefendState;

impl State<CharacterState> for DefendState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("defend_timer", 0.0);
        context.set_float("speed", 1.0);
        context.set_bool("defending", true);
        info!("Character entered {} state", CharacterState::Defend);
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("defend_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("defend_timer", timer);

        // Update character color to yellow (defensive)
        context.set_vector3("color", [0.8, 0.8, 0.2].into());
    }

    fn on_exit(&mut self, context: &mut StateContext) {
        context.set_bool("defending", false);
        info!("Character exited {} state", CharacterState::Defend);
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
        context.set_float("speed", 12.0);
        context.set_bool("escaping", true);
        info!("Character entered {} state", CharacterState::Escape);
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("escape_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("escape_timer", timer);

        // Update character color to purple (panicked)
        context.set_vector3("color", [0.8, 0.2, 0.8].into());
    }

    fn on_exit(&mut self, context: &mut StateContext) {
        context.set_bool("escaping", false);
        info!("Character exited {} state", CharacterState::Escape);
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
async fn main() -> anyhow::Result<()> {
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

    // Custom windows for FSM info
    let (w1_frame_tx, w1_frame_rx) = mpsc::channel::<Dt>();
    let w1_character_state = Arc::new(Mutex::new(CharacterState::Idle));
    let w1_character_sub_state = Arc::new(Mutex::new(Option::None));
    let w1_character_color = Arc::new(Mutex::new([0.2f32, 0.2f32, 0.8f32]));
    let character_state = w1_character_state.clone();
    let character_sub_state = w1_character_sub_state.clone();
    let character_color = w1_character_color.clone();
    app.add_window(Box::new(move |ui| {
        egui::Window::new("FSM Character Demo")
            .default_open(true)
            .max_width(400.0)
            .max_height(600.0)
            .default_width(350.0)
            .resizable(true)
            .anchor(Align2::RIGHT_TOP, [0.0, 0.0])
            .show(ui, |ui| {
                if let Ok(dt) = w1_frame_rx.try_recv() {
                    ui.label(format!("Frame time: {:.2} ms", dt.as_secs_f32() * 1000.0));
                    ui.label(format!("FPS: {:.0}", 1.0 / dt.as_secs_f32()));
                }

                ui.separator();
                ui.heading("Hierarchical FSM Demo");

                ui.separator();
                ui.label("Character States:");

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

                draw_state_with_color(ui, [0.2, 0.2, 0.8], "- IDLE - Calm, wandering");
                draw_state_with_color(ui, [0.8, 0.4, 0.1], "- ATTACK - Aggressive, pursuing");
                draw_state_with_color(ui, [0.8, 0.4, 0.1], "  - APPROACH - Moving towards target");
                draw_state_with_color(ui, [1.0, 0.1, 0.1], "  - STRIKE - Attacking target");
                draw_state_with_color(ui, [0.6, 0.2, 0.2], "  - RETREAT - Backing away");
                draw_state_with_color(ui, [0.8, 0.8, 0.2], "- DEFEND - Defensive, blocking");
                draw_state_with_color(ui, [0.8, 0.2, 0.8], "- ESCAPE - Panicked, fleeing");

                ui.separator();
                // Display current state color
                let color_rgb = character_color.lock().unwrap();
                let egui_color = egui::Color32::from_rgb(
                    (color_rgb[0] * 255.0) as u8,
                    (color_rgb[1] * 255.0) as u8,
                    (color_rgb[2] * 255.0) as u8,
                );
                ui.horizontal(|ui| {
                    ui.heading("Current State Information");
                    let color_rect =
                        egui::Rect::from_min_size(ui.cursor().min, egui::Vec2::new(20.0, 20.0));
                    ui.painter().rect_filled(color_rect, 2.0, egui_color);
                    ui.allocate_space(egui::Vec2::new(25.0, 20.0));
                });
                ui.label(format!("State: {:?}", character_state.lock().unwrap()));
                if let Some(sub_state) = character_sub_state.lock().unwrap().as_ref() {
                    ui.label(format!("Sub-State: {:?}", sub_state));
                }

                ui.separator();
                ui.label("State transitions based on:");
                ui.label("- Health level");
                ui.label("- Distance to enemy (player)");
                ui.label("- Time in current state");

                ui.separator();
                ui.label("Controls:");
                ui.label("WASD - Move player");
                ui.label("Mouse - Look around");
                ui.label("Space - Jump");
                ui.label("Alt - Keep the cursor within the window's bounds.");
                ui.label("Esc - Pause");
            });
    }));

    // Add ambient light
    new_entity!(
        app,
        LightMarker,
        Name("Ambient Light"),
        Light::Ambient { intensity: 0.1 },
        Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0))
    );

    // Add directional light
    new_entity!(
        app,
        LightMarker,
        Name("Directional Light"),
        Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.6,
        },
        Pos3::new(cgmath::Vector3::new(30.0, 30.0, 30.0))
    );

    // Plane
    new_entity!(
        app,
        RigidBodyMarker,
        Name("Plane"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-50.0, -0.1, -50.0),
            max: cgmath::Vector3::new(50.0, 0.1, 50.0),
        }),
        Pos3::new(cgmath::Vector3::new(0.0, -1.0, 0.0)),
        ModelSource::Obj("models/plane/plane.obj"),
    );

    // Player
    let mut player_prefab = Player::default();
    let player = new_entity!(
        app,
        PlayerMarker,
        player_prefab.pos3.take().unwrap(),
        player_prefab.model_source.take().unwrap(),
        player_prefab.movement_controller.take().unwrap(),
        player_prefab.view_controller.take().unwrap(),
        player_prefab.rigidbody.take().unwrap(),
        Health::default(),
    );

    // Character with FSM
    let mut character_fsm = FiniteStateMachine::<CharacterState>::new();

    // Add main states to the FSM
    character_fsm.add_state(CharacterState::Idle, Box::new(IdleState));
    character_fsm.add_state(CharacterState::Attack, Box::new(AttackState));
    character_fsm.add_state(CharacterState::Defend, Box::new(DefendState));
    character_fsm.add_state(CharacterState::Escape, Box::new(EscapeState));

    // Add sub-states for Attack state
    character_fsm.add_sub_state(
        CharacterState::Attack,
        CharacterState::AttackApproach,
        Box::new(AttackApproachState),
    );
    character_fsm.add_sub_state(
        CharacterState::Attack,
        CharacterState::AttackStrike,
        Box::new(AttackStrikeState),
    );
    character_fsm.add_sub_state(
        CharacterState::Attack,
        CharacterState::AttackRetreat,
        Box::new(AttackRetreatState),
    );

    // Set initial sub-state for Attack
    character_fsm.set_initial_sub_state(CharacterState::Attack, CharacterState::AttackApproach);

    // Set initial state
    character_fsm.set_initial_state(CharacterState::Idle);

    // Initialize character context
    character_fsm.context_mut().set_float("health", 100.0);
    character_fsm.context_mut().set_float("max_health", 100.0);
    character_fsm
        .context_mut()
        .set_float("enemy_distance", 100.0);
    character_fsm
        .context_mut()
        .set_vector3("color", [0.2, 0.2, 0.8].into());

    let character = new_entity!(
        app,
        CharacterMarker,
        StaticModelMarker,
        Name("FSM Character"),
        Pos3::new(cgmath::Vector3::new(5.0, 1.0, 0.0)),
        ModelSource::Obj("models/sphere/sphere.obj"),
        character_fsm,
        Health::new(100.0, 100.0),
    );

    // Add some obstacles
    for i in 0..4 {
        let angle = i as f32 * PI * 0.5;
        let x = angle.cos() * 25.0;
        let z = angle.sin() * 25.0;

        new_entity!(
            app,
            RigidBodyMarker,
            Name("Obstacle"),
            Pos3::new(cgmath::Vector3::new(x, 1.0, z)),
            RigidBody::new_static(AABBCollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            }),
            ModelSource::Obj("models/cube/cube.obj"),
        );
    }

    // FSM Update System
    let fsm_update_sys = systems::async_system("fsm_update", move |sa| {
        Box::pin({
            let w1_frame_tx = w1_frame_tx.clone();

            let fsm_debug_charater_state = w1_character_state.clone();
            let fsm_debug_character_sub_state = w1_character_sub_state.clone();
            let fsm_debug_character_color = w1_character_color.clone();
            async move {
                let (world, dt) = match sa {
                    SystemAccessors::External { world, dt } => (world, dt),
                    _ => return Ok(()),
                };

                w1_frame_tx
                    .send(*dt)
                    .map_err(|_| SystemError::Other("Failed to send dt".into()))?;

                // Get player position
                let player_pos = if let Some(player_pos3) = world.get_component::<Pos3>(player) {
                    player_pos3.read().unwrap().pos
                } else {
                    cgmath::Vector3::new(0.0, 0.0, 0.0)
                };

                // Update character FSM
                if let Some(character_pos3) = world.get_component::<Pos3>(character) {
                    if let Some(character_fsm) =
                        world.get_component::<FiniteStateMachine<CharacterState>>(character)
                    {
                        if let Some(character_health) = world.get_component::<Health>(character) {
                            let char_pos = character_pos3.read().unwrap().pos;
                            let distance = (player_pos - char_pos).magnitude();

                            // Update FSM
                            let mut fsm = character_fsm.write().unwrap();
                            let health = character_health.read().unwrap();

                            // Update context with current game state
                            fsm.context_mut().set_float("health", health.get_health());
                            fsm.context_mut().set_float("enemy_distance", distance);

                            // Update FSM
                            fsm.update(*dt);

                            // Apply FSM-driven behavior
                            let speed = fsm.context().get_float("speed").unwrap_or(2.0);
                            let current_state = fsm.current_state().unwrap_or(CharacterState::Idle);
                            let current_sub_state = fsm.current_sub_state();
                            // Update debug information in the information window
                            {
                                // Update main state for debugging
                                let mut debug_character_state =
                                    fsm_debug_charater_state.lock().unwrap();
                                *debug_character_state = current_state;

                                // Update sub state for debugging
                                if let Some(sub_state) = current_sub_state {
                                    let mut debug_sub_state =
                                        fsm_debug_character_sub_state.lock().unwrap();
                                    *debug_sub_state = Some(sub_state);
                                } else {
                                    let mut debug_sub_state =
                                        fsm_debug_character_sub_state.lock().unwrap();
                                    *debug_sub_state = None;
                                }

                                // Update color for debugging
                                if let Some(color_vec) = fsm.context().get_vector3("color") {
                                    let mut debug_color = fsm_debug_character_color.lock().unwrap();
                                    debug_color[0] = color_vec.x;
                                    debug_color[1] = color_vec.y;
                                    debug_color[2] = color_vec.z;
                                }
                            }
                            // Move character based on state
                            let mut pos_guard = character_pos3.write().unwrap();
                            match current_state {
                                CharacterState::Attack => {
                                    // Different movement based on sub-state
                                    match current_sub_state {
                                        Some(CharacterState::AttackApproach) => {
                                            // Move towards player
                                            let direction =
                                                (player_pos - pos_guard.pos).normalize();
                                            pos_guard.pos += direction * speed * dt.as_secs_f32();
                                        }
                                        Some(CharacterState::AttackStrike) => {
                                            // Stay close and strike (minimal movement)
                                            let direction =
                                                (player_pos - pos_guard.pos).normalize();
                                            pos_guard.pos +=
                                                direction * speed * 0.2 * dt.as_secs_f32();
                                        }
                                        Some(CharacterState::AttackRetreat) => {
                                            // Move away briefly
                                            let direction =
                                                (pos_guard.pos - player_pos).normalize();
                                            pos_guard.pos +=
                                                direction * speed * 0.5 * dt.as_secs_f32();
                                        }
                                        _ => {
                                            // Default attack behavior
                                            let direction =
                                                (player_pos - pos_guard.pos).normalize();
                                            pos_guard.pos += direction * speed * dt.as_secs_f32();
                                        }
                                    }
                                }
                                CharacterState::Escape => {
                                    // Move away from player
                                    let direction = (pos_guard.pos - player_pos).normalize();
                                    pos_guard.pos += direction * speed * dt.as_secs_f32();
                                }
                                CharacterState::Defend => {
                                    // Maintain distance, slight backing away
                                    if distance < 8.0 {
                                        let direction = (pos_guard.pos - player_pos).normalize();
                                        pos_guard.pos += direction * speed * 0.5 * dt.as_secs_f32();
                                    }
                                }
                                CharacterState::Idle => {
                                    // Random wandering (simplified)
                                    let wander_time = fsm.context().time_in_state.as_secs_f32();
                                    let wander_x = (wander_time * 0.7).sin() * 0.5;
                                    let wander_z = (wander_time * 0.5).cos() * 0.5;
                                    pos_guard.pos.x += wander_x * dt.as_secs_f32();
                                    pos_guard.pos.z += wander_z * dt.as_secs_f32();
                                }
                                _ => {}
                            }

                            // Keep character on the ground and within bounds
                            pos_guard.pos.y = 1.0;
                            pos_guard.pos.x = pos_guard.pos.x.clamp(-40.0, 40.0);
                            pos_guard.pos.z = pos_guard.pos.z.clamp(-40.0, 40.0);
                        }
                    }
                }

                Ok(())
            }
        })
    });

    app.add_async_system(fsm_update_sys);

    // Run the application
    app.run().await
}
