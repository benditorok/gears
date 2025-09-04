use cgmath::InnerSpace;
use egui::Align2;
use gears_app::systems::SystemError;
use gears_app::{prelude::*, systems};
use gears_macro::Component;
use log::{LevelFilter, info};
use std::f32::consts::PI;
use std::sync::mpsc;
use std::time::Duration;

// Character states
const IDLE: StateId = "idle";
const ATTACK: StateId = "attack";
const DEFEND: StateId = "defend";
const ESCAPE: StateId = "escape";

// Sub-states for each main state
const IDLE_WANDER: StateId = "idle_wander";
const IDLE_WATCH: StateId = "idle_watch";

const ATTACK_APPROACH: StateId = "attack_approach";
const ATTACK_STRIKE: StateId = "attack_strike";
const ATTACK_RETREAT: StateId = "attack_retreat";

const DEFEND_BLOCK: StateId = "defend_block";
const DEFEND_COUNTER: StateId = "defend_counter";

const ESCAPE_FLEE: StateId = "escape_flee";
const ESCAPE_HIDE: StateId = "escape_hide";

// Character marker for entities with FSM
#[derive(Component, Debug, Clone, Copy)]
pub struct CharacterMarker;

// Simple state implementations
#[derive(Debug)]
struct IdleState;

impl State for IdleState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("idle_timer", 0.0);
        context.set_float("speed", 2.0);
        info!("Character entered IDLE state");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("idle_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("idle_timer", timer);

        // Update character color to blue (calm)
        context.set_vector3("color", [0.2, 0.2, 0.8]);
    }

    fn check_transitions(&self, context: &StateContext) -> Option<StateId> {
        let health = context.get_float("health").unwrap_or(100.0);
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);
        let timer = context.get_float("idle_timer").unwrap_or(0.0);

        if health < 30.0 {
            Some(ESCAPE)
        } else if enemy_distance < 15.0 && health > 60.0 {
            Some(ATTACK)
        } else if enemy_distance < 8.0 && health <= 60.0 {
            Some(DEFEND)
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

impl State for AttackApproachState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("approach_timer", 0.0);
        context.set_float("speed", 6.0);
        info!("Character entered ATTACK > APPROACH sub-state");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("approach_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("approach_timer", timer);

        // Update character color to orange (approaching)
        context.set_vector3("color", [0.8, 0.4, 0.1]);
    }

    fn check_transitions(&self, context: &StateContext) -> Option<StateId> {
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);
        let timer = context.get_float("approach_timer").unwrap_or(0.0);

        if enemy_distance < 3.0 || timer > 2.0 {
            Some(ATTACK_STRIKE)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct AttackStrikeState;

impl State for AttackStrikeState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("strike_timer", 0.0);
        context.set_float("speed", 2.0);
        info!("Character entered ATTACK > STRIKE sub-state");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("strike_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("strike_timer", timer);

        // Update character color to bright red (striking)
        context.set_vector3("color", [1.0, 0.1, 0.1]);
    }

    fn check_transitions(&self, context: &StateContext) -> Option<StateId> {
        let timer = context.get_float("strike_timer").unwrap_or(0.0);

        if timer > 1.0 {
            Some(ATTACK_RETREAT)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct AttackRetreatState;

impl State for AttackRetreatState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("retreat_timer", 0.0);
        context.set_float("speed", 4.0);
        info!("Character entered ATTACK > RETREAT sub-state");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("retreat_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("retreat_timer", timer);

        // Update character color to dark red (retreating)
        context.set_vector3("color", [0.6, 0.2, 0.2]);
    }

    fn check_transitions(&self, context: &StateContext) -> Option<StateId> {
        let timer = context.get_float("retreat_timer").unwrap_or(0.0);
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);

        if timer > 1.5 && enemy_distance > 5.0 {
            Some(ATTACK_APPROACH)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct DefendState;

impl State for DefendState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("defend_timer", 0.0);
        context.set_float("speed", 1.0);
        context.set_bool("defending", true);
        info!("Character entered DEFEND state");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("defend_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("defend_timer", timer);

        // Update character color to yellow (defensive)
        context.set_vector3("color", [0.8, 0.8, 0.2]);
    }

    fn on_exit(&mut self, context: &mut StateContext) {
        context.set_bool("defending", false);
        info!("Character exited DEFEND state");
    }

    fn check_transitions(&self, context: &StateContext) -> Option<StateId> {
        let health = context.get_float("health").unwrap_or(100.0);
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);
        let timer = context.get_float("defend_timer").unwrap_or(0.0);

        if health < 30.0 {
            Some(ESCAPE)
        } else if health > 80.0 && enemy_distance < 10.0 {
            Some(ATTACK)
        } else if enemy_distance > 15.0 || timer > 4.0 {
            Some(IDLE)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct EscapeState;

impl State for EscapeState {
    fn on_enter(&mut self, context: &mut StateContext) {
        context.set_float("escape_timer", 0.0);
        context.set_float("speed", 12.0);
        context.set_bool("escaping", true);
        info!("Character entered ESCAPE state");
    }

    fn on_update(&mut self, context: &mut StateContext, dt: Duration) {
        let timer = context.get_float("escape_timer").unwrap_or(0.0) + dt.as_secs_f32();
        context.set_float("escape_timer", timer);

        // Update character color to purple (panicked)
        context.set_vector3("color", [0.8, 0.2, 0.8]);
    }

    fn on_exit(&mut self, context: &mut StateContext) {
        context.set_bool("escaping", false);
        info!("Character exited ESCAPE state");
    }

    fn check_transitions(&self, context: &StateContext) -> Option<StateId> {
        let health = context.get_float("health").unwrap_or(100.0);
        let enemy_distance = context.get_float("enemy_distance").unwrap_or(100.0);
        let timer = context.get_float("escape_timer").unwrap_or(0.0);

        if health > 60.0 && enemy_distance > 25.0 {
            Some(IDLE)
        } else if health > 40.0 && enemy_distance > 15.0 && timer > 2.0 {
            Some(DEFEND)
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
    let character_entity_channel = std::sync::Arc::new(std::sync::Mutex::new(None));
    let character_entity_channel_clone = character_entity_channel.clone();
    let character_entity_ui = character_entity_channel.clone();

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

                // Show current state information
                if let Some(character_entity) = *character_entity_ui.lock().unwrap() {
                    ui.separator();
                    ui.label("Current State Information:");
                    ui.label("Main State: [Will be updated in system]");
                    ui.label("Sub-State: [Will be updated in system]");
                }

                ui.separator();
                ui.label("Character States:");
                ui.label("- IDLE - Calm, wandering");
                ui.label("- ATTACK - Aggressive, pursuing");
                ui.label("  ├ APPROACH - Moving towards target");
                ui.label("  ├ STRIKE - Attacking target");
                ui.label("  └ RETREAT - Backing away");
                ui.label("- DEFEND - Defensive, blocking");
                ui.label("- ESCAPE - Panicked, fleeing");

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
    let mut character_fsm = FiniteStateMachine::new();

    // Create hierarchical attack state with sub-states
    let mut attack_state = HierarchicalState::new()
        .with_enter_callback(|ctx| {
            ctx.set_float("attack_timer", 0.0);
            ctx.set_bool("attacking", true);
            info!("Character entered ATTACK state");
        })
        .with_exit_callback(|ctx| {
            ctx.set_bool("attacking", false);
            info!("Character exited ATTACK state");
        })
        .with_update_callback(|ctx, dt| {
            let timer = ctx.get_float("attack_timer").unwrap_or(0.0) + dt.as_secs_f32();
            ctx.set_float("attack_timer", timer);
        })
        .with_transition_callback(|ctx| {
            let health = ctx.get_float("health").unwrap_or(100.0);
            let enemy_distance = ctx.get_float("enemy_distance").unwrap_or(100.0);
            let timer = ctx.get_float("attack_timer").unwrap_or(0.0);

            if health < 30.0 {
                Some(ESCAPE)
            } else if enemy_distance > 20.0 || timer > 8.0 {
                Some(IDLE)
            } else if health < 60.0 && enemy_distance < 5.0 {
                Some(DEFEND)
            } else {
                None
            }
        });

    // Add sub-states to attack
    attack_state.add_sub_state(ATTACK_APPROACH, Box::new(AttackApproachState));
    attack_state.add_sub_state(ATTACK_STRIKE, Box::new(AttackStrikeState));
    attack_state.add_sub_state(ATTACK_RETREAT, Box::new(AttackRetreatState));
    attack_state.set_initial_sub_state(ATTACK_APPROACH);

    // Add states to the FSM
    character_fsm.add_state(IDLE, Box::new(IdleState));
    character_fsm.add_state(ATTACK, Box::new(attack_state));
    character_fsm.add_state(DEFEND, Box::new(DefendState));
    character_fsm.add_state(ESCAPE, Box::new(EscapeState));

    // Set initial state
    character_fsm.set_initial_state(IDLE);

    // Initialize character context
    character_fsm.context_mut().set_float("health", 100.0);
    character_fsm.context_mut().set_float("max_health", 100.0);
    character_fsm
        .context_mut()
        .set_float("enemy_distance", 100.0);
    character_fsm
        .context_mut()
        .set_vector3("color", [0.2, 0.2, 0.8]);

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

    // Store character entity for UI access
    *character_entity_channel_clone.lock().unwrap() = Some(character);

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
                        world.get_component::<FiniteStateMachine>(character)
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

                            // Debug: Log character state and position every 2 seconds
                            static mut LAST_LOG_TIME: f32 = 0.0;
                            unsafe {
                                LAST_LOG_TIME += dt.as_secs_f32();
                                if LAST_LOG_TIME > 2.0 {
                                    let current_state = fsm.current_state().unwrap_or("unknown");
                                    let current_sub_state = fsm.current_sub_state();
                                    let sub_state_str = match current_sub_state {
                                        Some(sub) => format!(" > {}", sub),
                                        None => String::new(),
                                    };
                                    info!(
                                        "Character: {}{}, position: {:?}, distance: {:.2}, health: {:.1}",
                                        current_state,
                                        sub_state_str,
                                        char_pos,
                                        distance,
                                        health.get_health()
                                    );
                                    LAST_LOG_TIME = 0.0;
                                }
                            }

                            // Apply FSM-driven behavior
                            let speed = fsm.context().get_float("speed").unwrap_or(2.0);
                            let current_state = fsm.current_state().unwrap_or("unknown");
                            let current_sub_state = fsm.current_sub_state();

                            // Move character based on state
                            let mut pos_guard = character_pos3.write().unwrap();
                            match current_state {
                                ATTACK => {
                                    // Different movement based on sub-state
                                    match current_sub_state {
                                        Some(ATTACK_APPROACH) => {
                                            // Move towards player
                                            let direction =
                                                (player_pos - pos_guard.pos).normalize();
                                            pos_guard.pos += direction * speed * dt.as_secs_f32();
                                        }
                                        Some(ATTACK_STRIKE) => {
                                            // Stay close and strike (minimal movement)
                                            let direction =
                                                (player_pos - pos_guard.pos).normalize();
                                            pos_guard.pos +=
                                                direction * speed * 0.2 * dt.as_secs_f32();
                                        }
                                        Some(ATTACK_RETREAT) => {
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
                                ESCAPE => {
                                    // Move away from player
                                    let direction = (pos_guard.pos - player_pos).normalize();
                                    pos_guard.pos += direction * speed * dt.as_secs_f32();
                                }
                                DEFEND => {
                                    // Maintain distance, slight backing away
                                    if distance < 8.0 {
                                        let direction = (pos_guard.pos - player_pos).normalize();
                                        pos_guard.pos += direction * speed * 0.5 * dt.as_secs_f32();
                                    }
                                }
                                IDLE => {
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
