use cgmath::InnerSpace;
use egui::Align2;
use gears_app::systems::SystemError;
use gears_app::{prelude::*, systems};
use gears_macro::Component;
use log::{LevelFilter, info};
use std::sync::mpsc;

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

    // Custom window for pathfinding info
    let (w1_frame_tx, w1_frame_rx) = mpsc::channel::<Dt>();
    app.add_window(Box::new(move |ui| {
        egui::Window::new("Pathfinding Demo")
            .default_open(true)
            .max_width(400.0)
            .max_height(600.0)
            .default_width(300.0)
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
                ui.label("Esc - Pause");
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
                1.5,                                   // Mass of the enemy
                cgmath::Vector3::new(0.0, 0.0, 0.0),   // Initial velocity
                cgmath::Vector3::new(0.0, -10.0, 0.0), // Initial acceleration (gravity)
                AABBCollisionBox {
                    min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                    max: cgmath::Vector3::new(1.0, 1.0, 1.0),
                },
            ),
            ModelSource::Obj("models/sphere/sphere.obj"),
            pathfinding,
        );
        enemies.push(enemy);
    }

    // Pathfinding system
    let pathfinding_sys = systems::async_system("pathfinding_update", move |sa| {
        let frame_tx = w1_frame_tx.clone();
        Box::pin(async move {
            let (world, dt) = match sa {
                SystemAccessors::External { world, dt } => (world, dt),
                _ => return Ok(()),
            };

            frame_tx
                .send(*dt)
                .map_err(|_| SystemError::Other("Failed to send dt".into()))?;

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
            let obstacles: Vec<(cgmath::Vector3<f32>, AABBCollisionBox)> = rigid_body_entities
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

            info!("Found {} obstacles for pathfinding", obstacles.len());

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
                    pathfinding.set_target(player_pos);

                    // Check if we need pathfinding and should recalculate
                    let current_pos = {
                        let pos3 = pos3_comp.read().map_err(|e| {
                            SystemError::ComponentAccess(format!("Failed to read Pos3: {}", e))
                        })?;
                        pos3.pos
                    };

                    info!(
                        "Entity {:?} - needs pathfinding: {}, should recalculate: {}",
                        entity,
                        pathfinding.needs_pathfinding(current_pos),
                        pathfinding.should_recalculate_path()
                    );

                    if pathfinding.needs_pathfinding(current_pos)
                        && pathfinding.should_recalculate_path()
                    {
                        entities_needing_paths.push(entity);
                    }
                }
            }

            info!(
                "Found {} entities needing path calculation",
                entities_needing_paths.len()
            );

            // Second pass: calculate paths for entities that need them (limit to max 1 per frame for performance)
            if let Some(&entity) = entities_needing_paths.first() {
                info!("Calculating path for entity {:?}", entity);
                let pathfinding_comp = world.get_component::<PathfindingComponent>(entity).unwrap();
                let pos3_comp = world.get_component::<Pos3>(entity).unwrap();

                // Get current position
                let current_pos = {
                    let pos3 = pos3_comp.read().map_err(|e| {
                        SystemError::ComponentAccess(format!("Failed to read Pos3: {}", e))
                    })?;
                    pos3.pos
                };

                info!(
                    "Current position: {:?}, Target: {:?}",
                    current_pos, player_pos
                );

                // Build pathfinding grid from collected obstacles (excluding this entity)
                let mut astar = AStar::new(2.0);
                astar.build_grid_from_entities(obstacles.iter().map(|(pos, cb)| (pos, cb)));

                if let Some(path) = astar.find_path(current_pos, player_pos) {
                    info!("Path found with {} waypoints", path.len());
                    let mut pathfinding = pathfinding_comp.write().map_err(|e| {
                        SystemError::ComponentAccess(format!(
                            "Failed to write PathfindingComponent: {}",
                            e
                        ))
                    })?;
                    pathfinding.set_path(path);
                } else {
                    info!("No path found!");
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

                let rigidbody_comp =
                    match world.get_component::<RigidBody<AABBCollisionBox>>(entity) {
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

                        info!(
                            "Entity at {:?} moving toward waypoint {:?}, distance: {:.2}",
                            pos3.pos,
                            waypoint,
                            direction.magnitude()
                        );

                        if direction.magnitude() > pathfinding.waypoint_threshold {
                            // Apply horizontal acceleration toward target
                            let normalized_dir = direction.normalize();
                            let target_acceleration = normalized_dir * pathfinding.speed * 8.0; // Multiply for stronger acceleration

                            // Keep gravity (y-component of acceleration) and add horizontal movement
                            rigidbody.acceleration.x = target_acceleration.x;
                            rigidbody.acceleration.z = target_acceleration.z;
                            // Leave rigidbody.acceleration.y unchanged (gravity)

                            // Apply some damping to horizontal velocity to prevent overshooting
                            rigidbody.velocity.x *= 0.85;
                            rigidbody.velocity.z *= 0.85;

                            info!("Applied acceleration: {:?}", target_acceleration);
                        } else {
                            // Reached waypoint, advance to next
                            info!("Reached waypoint, advancing to next");

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
                        info!("No waypoint available for entity");
                    }
                }
            }

            Ok(())
        })
    });

    app.add_async_system(pathfinding_sys);

    // Run the application
    app.run().await
}
