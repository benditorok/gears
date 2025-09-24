// Simple Query System Example
// This demonstrates how to use the query system to prevent deadlocks

use cgmath::Vector3;
use gears_app::prelude::*;
use log::{LevelFilter, info};
use std::time::Duration;

// Example components
#[derive(Component, Debug)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

impl Position {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Component, Debug)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

impl Velocity {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Component, Debug)]
struct Health {
    current: f32,
    max: f32,
}

impl Health {
    fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    fn is_alive(&self) -> bool {
        self.current > 0.0
    }
}

#[derive(Component, Debug)]
struct MovingObject;

#[derive(Component, Debug)]
struct LivingEntity;

#[tokio::main]
async fn main() -> EngineResult<()> {
    // Initialize logger
    let mut env_builder = env_logger::Builder::new();
    env_builder.filter_level(LevelFilter::Info);
    env_builder.init();

    let mut app = GearsApp::default();

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
    );

    // Create some entities with components
    let entity1 = new_entity!(
        app,
        MovingObject,
        LivingEntity,
        Position::new(0.0, 0.0, 0.0),
        Velocity::new(1.0, 0.0, 0.0),
        Health::new(100.0)
    );

    let entity2 = new_entity!(
        app,
        MovingObject,
        LivingEntity,
        Position::new(10.0, 0.0, 0.0),
        Velocity::new(-1.0, 0.0, 0.0),
        Health::new(80.0)
    );

    let entity3 = new_entity!(
        app,
        MovingObject,
        Position::new(5.0, 5.0, 0.0),
        Velocity::new(0.0, -1.0, 0.0)
    );

    // System 1: Movement System (reads velocity, writes position)
    async_system!(app, "movement_system", move |world, dt| {
        let moving_entities = world.get_entities_with_component::<MovingObject>();

        // Build query for all moving entities
        let query = ComponentQuery::new()
            .read::<Velocity>(moving_entities.clone())
            .write::<Position>(moving_entities.clone());

        // Try to acquire resources with timeout
        if let Some(resources) = world.try_acquire_query(query, Duration::from_millis(5)) {
            info!(
                "Movement system executing for {} entities",
                moving_entities.len()
            );

            // Process each entity
            for &entity in &moving_entities {
                if let (Some(velocity_comp), Some(position_comp)) = (
                    resources.get::<Velocity>(entity),
                    resources.get::<Position>(entity), // world.get_component::<Velocity>(entity),
                                                       // world.get_component::<Position>(entity),
                ) {
                    if let (Ok(velocity), Ok(mut position)) =
                        (velocity_comp.try_read(), position_comp.try_write())
                    {
                        // Update position based on velocity
                        let dt_secs = dt.as_secs_f32();
                        position.x += velocity.x * dt_secs;
                        position.y += velocity.y * dt_secs;
                        position.z += velocity.z * dt_secs;

                        info!(
                            "Entity {} moved to ({:.1}, {:.1}, {:.1})",
                            *entity, position.x, position.y, position.z
                        );
                    }
                }
            }
        } else {
            info!("Movement system deferred - resources locked");
        }

        Ok(())
    });

    // System 2: Health System (reads position, writes health)
    async_system!(app, "health_system", move |world, dt| {
        let living_entities = world.get_entities_with_component::<LivingEntity>();

        // Build query for living entities
        let query = ComponentQuery::new()
            .read::<Position>(living_entities.clone())
            .write::<Health>(living_entities.clone());

        if let Some(_resources) = world.try_acquire_query(query, Duration::from_millis(5)) {
            info!(
                "Health system executing for {} entities",
                living_entities.len()
            );

            for &entity in &living_entities {
                if let (Some(position_comp), Some(health_comp)) = (
                    world.get_component::<Position>(entity),
                    world.get_component::<Health>(entity),
                ) {
                    if let (Ok(position), Ok(mut health)) =
                        (position_comp.try_read(), health_comp.try_write())
                    {
                        // Damage entities that are out of bounds
                        if position.x.abs() > 20.0 || position.y.abs() > 20.0 {
                            health.damage(10.0 * dt.as_secs_f32());
                            info!(
                                "Entity {} took damage! Health: {:.1}/{:.1}",
                                *entity, health.current, health.max
                            );
                        }

                        if !health.is_alive() {
                            info!("Entity {} died!", *entity);
                        }
                    }
                }
            }
        } else {
            info!("Health system deferred - resources locked");
        }

        Ok(())
    });

    // System 3: Collision System (reads position and velocity, writes velocity)
    async_system!(app, "collision_system", move |world, dt| {
        let moving_entities = world.get_entities_with_component::<MovingObject>();

        // This system needs both read and write access to velocity, and read access to position
        let query = ComponentQuery::new()
            .read::<Position>(moving_entities.clone())
            .write::<Velocity>(moving_entities.clone());

        if let Some(_resources) = world.try_acquire_query(query, Duration::from_millis(5)) {
            info!(
                "Collision system executing for {} entities",
                moving_entities.len()
            );

            // Simple collision detection and response
            for &entity in &moving_entities {
                if let (Some(position_comp), Some(velocity_comp)) = (
                    world.get_component::<Position>(entity),
                    world.get_component::<Velocity>(entity),
                ) {
                    if let (Ok(position), Ok(mut velocity)) =
                        (position_comp.try_read(), velocity_comp.try_write())
                    {
                        // Bounce off boundaries
                        let boundary = 15.0;
                        let mut bounced = false;

                        if position.x > boundary || position.x < -boundary {
                            velocity.x = -velocity.x;
                            bounced = true;
                        }
                        if position.y > boundary || position.y < -boundary {
                            velocity.y = -velocity.y;
                            bounced = true;
                        }

                        if bounced {
                            info!(
                                "Entity {} bounced! New velocity: ({:.1}, {:.1}, {:.1})",
                                *entity, velocity.x, velocity.y, velocity.z
                            );
                        }
                    }
                }
            }
        } else {
            info!("Collision system deferred - resources locked");
        }

        Ok(())
    });

    // Add a simple UI to show system status
    app.add_window(Box::new(|ui| {
        egui::Window::new("Query System Demo")
            .default_open(true)
            .show(ui, |ui| {
                ui.heading("Query System Example");
                ui.separator();

                ui.label("This example demonstrates:");
                ui.label("• Movement System: Updates position based on velocity");
                ui.label("• Health System: Damages entities out of bounds");
                ui.label("• Collision System: Bounces entities off boundaries");
                ui.separator();

                ui.label("The Query System prevents deadlocks by:");
                ui.label("• Declaring resource needs upfront");
                ui.label("• Atomic resource acquisition");
                ui.label("• Graceful degradation when resources unavailable");
                ui.label("• Priority-based conflict resolution");
                ui.separator();

                ui.label("Watch the console for system execution logs!");
            });
    }));

    info!("Starting Query System Demo");
    info!("Created entities: {}, {}, {}", *entity1, *entity2, *entity3);

    app.run().await
}
