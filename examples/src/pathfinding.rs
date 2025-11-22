use cgmath::{InnerSpace, Vector3, Zero};
use egui::Align2;
use gears_app::prelude::*;
use log::{LevelFilter, info};
use std::sync::mpsc;

#[tokio::main]
async fn main() -> EngineResult<()> {
    // Initialize the logger
    let mut env_builder = env_logger::Builder::new();
    env_builder.filter_level(LevelFilter::Debug);
    env_builder.filter_module("wgpu_core::device::resource", log::LevelFilter::Warn);
    env_builder.init();

    let mut app = GearsApp::default();

    // Custom window for pathfinding info
    let (w1_frame_tx, w1_frame_rx) = mpsc::channel::<Dt>();
    app.add_window(Box::new(move |ui| {
        egui::Window::new("Pathfinding Demo")
            .default_open(true)
            .max_width(300.0)
            .max_height(600.0)
            .resizable(true)
            .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .show(ui, |ui| {
                if let Ok(dt) = w1_frame_rx.try_recv() {
                    ui.label(format!("Frame time: {:.2} ms", dt.as_secs_f32() * 1000.0));
                    ui.label(format!("FPS: {:.0}", 1.0 / dt.as_secs_f32()));
                }

                ui.separator();
                ui.label("A* Pathfinding Demo");
                ui.label("Red spheres use A* to track the player");
                ui.label("They navigate around obstacles");
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

    // Add ambient light
    new_entity!(
        app,
        LightMarker,
        Name("Ambient Light"),
        Light::Ambient { intensity: 0.05 },
        Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0))
    );

    // Add directional light
    new_entity!(
        app,
        LightMarker,
        Name("Directional Light"),
        Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.4,
        },
        Pos3::new(cgmath::Vector3::new(30.0, 30.0, 30.0))
    );

    // Plane (ground)
    new_entity!(
        app,
        RigidBodyMarker,
        Name("Ground Plane"),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-50.0, -0.1, -50.0),
            max: cgmath::Vector3::new(50.0, 0.1, 50.0),
        }),
        Pos3::new(cgmath::Vector3::new(0.0, -1.0, 0.0)),
        ModelSource::Obj("models/plane/plane.obj"),
    );

    // Player
    let mut player_prefab = Player::default();
    let _player = new_entity!(
        app,
        PlayerMarker,
        PathfindingTarget, // Mark player as a pathfinding target
        player_prefab.pos3.take().unwrap(),
        player_prefab.model_source.take().unwrap(),
        player_prefab.movement_controller.take().unwrap(),
        player_prefab.view_controller.take().unwrap(),
        player_prefab.rigidbody.take().unwrap(),
        Health::default(),
        Weapon::new(20.0),
    );

    // Create some obstacles
    let obstacles = vec![
        (cgmath::Vector3::new(10.0, 1.0, 5.0), "Obstacle 1"),
        (cgmath::Vector3::new(-8.0, 1.0, -3.0), "Obstacle 2"),
        (cgmath::Vector3::new(5.0, 1.0, -10.0), "Obstacle 3"),
        (cgmath::Vector3::new(-12.0, 1.0, 8.0), "Obstacle 4"),
        (cgmath::Vector3::new(0.0, 1.0, 12.0), "Obstacle 5"),
        (cgmath::Vector3::new(15.0, 1.0, -5.0), "Obstacle 6"),
    ];

    for (pos, name) in obstacles {
        new_entity!(
            app,
            RigidBodyMarker,
            ObstacleMarker,
            Name(name),
            Pos3::new(pos),
            RigidBody::new_static(AABBCollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            }),
            ModelSource::Obj("models/cube/cube.obj"),
        );
    }

    // Create enemy entities that will track the player using A* pathfinding
    let enemy_positions = vec![
        cgmath::Vector3::new(-20.0, 1.0, -20.0),
        cgmath::Vector3::new(20.0, 1.0, -20.0),
        cgmath::Vector3::new(-20.0, 1.0, 20.0),
        cgmath::Vector3::new(20.0, 1.0, 20.0),
    ];

    let mut enemies = Vec::new();
    for (i, pos) in enemy_positions.into_iter().enumerate() {
        let enemy_name = format!("Enemy_{}", i + 1);
        let pathfinding = PathfindingComponent::new(
            cgmath::Vector3::new(0.0, 1.0, 0.0), // Initial target (will be updated to player position)
            20.0,                                // Movement speed
            2.0,                                 // Grid cell size
        );

        let enemy = new_entity!(
            app,
            EnemyMarker,
            PathfindingFollower,
            RigidBodyMarker,
            Name(Box::leak(enemy_name.into_boxed_str())),
            Pos3::new(pos),
            RigidBody::new(
                1.5,
                Vector3::zero(),
                Vector3::new(0.0, -10.0, 0.0),
                AABBCollisionBox {
                    min: cgmath::Vector3::new(-1.0, -2.0, -1.0),
                    max: cgmath::Vector3::new(1.0, 2.0, 1.0),
                }
            ),
            ModelSource::Obj("models/capsule/capsule.obj"),
            pathfinding,
        );
        enemies.push(enemy);
    }

    async_system!(app, "pathfinding_update", (w1_frame_tx), |world, dt| {
        w1_frame_tx
            .send(dt)
            .map_err(|_| SystemError::Other("Failed to send dt".into()))?;

        // Get player position first
        let player_entities = world.get_entities_with_component::<PathfindingTarget>();
        let player_pos = if let Some(&player_entity) = player_entities.first() {
            let query = ComponentQuery::new().read::<Pos3>(vec![player_entity]);
            if let Some(resources) = world.acquire_query(query) {
                if let Some(pos3) = resources.get::<Pos3>(player_entity) {
                    pos3.read()
                        .map_err(|e| {
                            SystemError::ComponentAccess(format!(
                                "Failed to read player Pos3: {}",
                                e
                            ))
                        })?
                        .pos
                } else {
                    return Ok(());
                }
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        // Collect obstacle data
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
                    let pos_guard = pos_comp.read().map_err(|e| {
                        SystemError::ComponentAccess(format!("Failed to read obstacle Pos3: {}", e))
                    })?;
                    let rb_guard = rb_comp.read().map_err(|e| {
                        SystemError::ComponentAccess(format!(
                            "Failed to read obstacle RigidBody: {}",
                            e
                        ))
                    })?;

                    obstacles.push((pos_guard.pos, rb_guard.collision_box.clone()));
                }
            }
        }

        info!("Found {} obstacles for pathfinding", obstacles.len());

        // Process each follower entity individually
        let follower_entities = world.get_entities_with_component::<PathfindingFollower>();

        for &entity in follower_entities.iter() {
            // First, update pathfinding and check if we need a new path
            let needs_new_path = {
                let query = ComponentQuery::new()
                    .write::<PathfindingComponent>(vec![entity])
                    .read::<Pos3>(vec![entity]);

                if let Some(resources) = world.acquire_query(query) {
                    if let (Some(pathfinding_comp), Some(pos3_comp)) = (
                        resources.get::<PathfindingComponent>(entity),
                        resources.get::<Pos3>(entity),
                    ) {
                        let mut pathfinding = pathfinding_comp.write().map_err(|e| {
                            SystemError::ComponentAccess(format!(
                                "Failed to write PathfindingComponent: {}",
                                e
                            ))
                        })?;
                        let current_pos = pos3_comp
                            .read()
                            .map_err(|e| {
                                SystemError::ComponentAccess(format!("Failed to read Pos3: {}", e))
                            })?
                            .pos;

                        pathfinding.update(dt.as_secs_f32());
                        pathfinding.set_target(player_pos);

                        pathfinding.needs_pathfinding(current_pos)
                            && pathfinding.should_recalculate_path()
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            // If we need a new path, calculate it
            if needs_new_path {
                let query = ComponentQuery::new()
                    .write::<PathfindingComponent>(vec![entity])
                    .read::<Pos3>(vec![entity]);

                if let Some(resources) = world.acquire_query(query) {
                    if let (Some(pathfinding_comp), Some(pos3_comp)) = (
                        resources.get::<PathfindingComponent>(entity),
                        resources.get::<Pos3>(entity),
                    ) {
                        let current_pos = pos3_comp
                            .read()
                            .map_err(|e| {
                                SystemError::ComponentAccess(format!("Failed to read Pos3: {}", e))
                            })?
                            .pos;

                        // Build pathfinding grid
                        let mut astar = AStar::new(2.0, DistanceHeuristic::Manhattan);
                        astar.build_grid_from_entities(obstacles.iter().map(|(pos, cb)| (pos, cb)));

                        if let Some(path) = astar.find_path(current_pos, player_pos) {
                            info!(
                                "Path found with {} waypoints for entity {:?}",
                                path.len(),
                                entity
                            );
                            let mut pathfinding = pathfinding_comp.write().map_err(|e| {
                                SystemError::ComponentAccess(format!(
                                    "Failed to write PathfindingComponent: {}",
                                    e
                                ))
                            })?;
                            pathfinding.set_path(path);
                        } else {
                            info!("No path found for entity {:?}", entity);
                        }
                    }
                }
            }

            // Move the entity along its path
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
                    let mut pathfinding = pathfinding_comp.write().map_err(|e| {
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

                    let should_advance_waypoint =
                        if let Some(waypoint) = pathfinding.current_waypoint() {
                            let mut direction = waypoint - pos3.pos;
                            direction.y = 0.0; // Only horizontal movement

                            if direction.magnitude() > pathfinding.waypoint_threshold {
                                // Move toward waypoint
                                let normalized_dir = direction.normalize();
                                let target_acceleration = normalized_dir * pathfinding.speed * 8.0;

                                rigidbody.acceleration.x = target_acceleration.x;
                                rigidbody.acceleration.z = target_acceleration.z;
                                rigidbody.velocity.x *= 0.85;
                                rigidbody.velocity.z *= 0.85;

                                info!("Entity {:?} moving toward waypoint {:?}", entity, waypoint);
                                false // Don't advance waypoint
                            } else {
                                // Reached waypoint - stop movement and mark for advancement
                                rigidbody.acceleration.x = 0.0;
                                rigidbody.acceleration.z = 0.0;
                                rigidbody.velocity.x *= 0.5;
                                rigidbody.velocity.z *= 0.5;
                                true // Advance waypoint
                            }
                        } else {
                            // No waypoint, stop movement
                            rigidbody.acceleration.x = 0.0;
                            rigidbody.acceleration.z = 0.0;
                            rigidbody.velocity.x *= 0.8;
                            rigidbody.velocity.z *= 0.8;
                            false // No waypoint to advance
                        };

                    // If we need to advance waypoint, do it with a separate query
                    if should_advance_waypoint {
                        pathfinding.advance_waypoint();
                        info!("Advanced waypoint for entity {:?}", entity);
                    }
                }
            }
        }

        Ok(())
    });

    // Run the application
    app.run()
}
