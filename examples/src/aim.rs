use cgmath::Rotation3;
use egui::Align2;

use gears_app::prelude::*;
use gears_app::systems::SystemError;
use log::LevelFilter;
use std::f32::consts::PI;
use std::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("{}", info);
        println!("Press Enter to close...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
    }));

    // TODO collect the entities in a single init fn so we can use an ENUM marker with its variants
    // TODO Model locations should be relative to the exe in a released build

    // Initialize the logger
    let mut env_builder = env_logger::Builder::new();
    env_builder.filter_level(LevelFilter::Info);
    env_builder.filter_module("wgpu_core::device::resource", log::LevelFilter::Warn);
    env_builder.init();

    let mut app = GearsApp::default();

    // ! Custom windows
    // Informations about the renderer
    let (w1_frame_tx, w1_frame_rx) = mpsc::channel::<Dt>();
    app.add_window(Box::new(move |ui| {
        egui::Window::new("Renderer info")
            .default_open(true)
            .max_width(1000.0)
            .max_height(800.0)
            .default_width(800.0)
            .resizable(true)
            .anchor(Align2::RIGHT_TOP, [0.0, 0.0])
            .show(ui, |ui| {
                if let Ok(dt) = w1_frame_rx.try_recv() {
                    ui.label(format!("Frame time: {:.2} ms", dt.as_secs_f32() * 1000.0));
                    ui.label(format!("FPS: {:.0}", 1.0 / dt.as_secs_f32()));
                }

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
        Pos3::new(cgmath::Vector3::new(30.0, 30.0, 30.0,))
    );

    // * Add moving red light
    let red_light = new_entity!(
        app,
        LightMarker,
        Name("Red Light"),
        Light::PointColoured {
            radius: 10.0,
            color: [0.8, 0.0, 0.0],
            intensity: 1.0,
        },
        Pos3::new(cgmath::Vector3::new(15.0, 5.0, 0.0))
    );

    // * Add moving blue light
    let blue_light = new_entity!(
        app,
        LightMarker,
        Name("Blue Light"),
        Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.0, 0.8],
            intensity: 1.0,
        },
        Pos3::new(cgmath::Vector3::new(-15.0, 5.0, 0.0))
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
    // * ENDREGION

    // * Player
    let mut player_prefab = Player::default();
    let player = new_entity!(
        app,
        PlayerMarker,
        player_prefab.pos3.take().unwrap(),
        player_prefab.model_source.take().unwrap(),
        player_prefab.movement_controller.take().unwrap(),
        player_prefab.view_controller.take().unwrap(),
        player_prefab.rigidbody.take().unwrap(),
        // * + HEALTH
        Health::default(),
        // * + WEAPON
        Weapon::new(20.0),
    );
    // * Player

    // Add 5 spheres in a circle
    let mut moving_spheres: [Entity; 5] = [Entity::new(0); 5];
    for (i, sphere) in moving_spheres.iter_mut().enumerate() {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 5.0;
        let x = angle.cos() * 10.0;
        let z = angle.sin() * 10.0;

        let name = format!("Sphere_circle{}", i);

        let sphere_entity = new_entity!(
            app,
            RigidBodyMarker,
            Name(Box::leak(name.into_boxed_str())),
            Pos3::new(cgmath::Vector3::new(x, 1.0, z)),
            RigidBody::new_static(AABBCollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },),
            ModelSource::Obj("models/sphere/sphere.obj"),
        );

        *sphere = sphere_entity;
    }

    new_entity!(
        app,
        RigidBodyMarker,
        Name("Static cube"),
        Pos3::new(cgmath::Vector3::new(0.0, 0.0, 20.0)),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
            max: cgmath::Vector3::new(1.0, 1.0, 1.0),
        },),
        ModelSource::Obj("models/cube/cube.obj"),
    );

    new_entity!(
        app,
        RigidBodyMarker,
        Name("Static cube"),
        Pos3::new(cgmath::Vector3::new(20.0, 0.0, 20.0)),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
            max: cgmath::Vector3::new(1.0, 1.0, 1.0),
        },),
        ModelSource::Obj("models/cube/cube.obj"),
    );

    new_entity!(
        app,
        RigidBodyMarker,
        Name("Static cube"),
        Pos3::new(cgmath::Vector3::new(0.0, 0.0, -20.0)),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
            max: cgmath::Vector3::new(1.0, 1.0, 1.0),
        },),
        ModelSource::Obj("models/cube/cube.obj"),
    );

    new_entity!(
        app,
        RigidBodyMarker,
        Name("Static cube"),
        Pos3::new(cgmath::Vector3::new(-20.0, 0.0, -20.0)),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
            max: cgmath::Vector3::new(1.0, 1.0, 1.0),
        },),
        ModelSource::Obj("models/cube/cube.obj"),
    );

    // // Merc
    // new_entity!(
    //     app,
    //     RigidBodyMarker,
    //     Name("Merc"),
    //     Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0)),
    //     RigidBody::new_static(CollisionBox {
    //         min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
    //         max: cgmath::Vector3::new(1.0, 1.0, 1.0),
    //     },),
    //     ModelSource::Gltf("gltf/merc/scene.gltf"),
    // );

    // Target
    let target = new_entity!(
        app,
        TargetMarker,
        RigidBodyMarker,
        Name("Target"),
        Pos3::new(cgmath::Vector3::new(30.0, 3.0, 0.0)),
        RigidBody::new(
            20.0,
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            AABBCollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        ),
        ModelSource::Obj("models/sphere/sphere.obj"),
        // * + HEALTH
        Health::default(),
    );

    // Systems can now be written with simple async closures
    // When capturing variables in move closures, use the closure capture pattern
    async_system!(app, "update", {
        // let w1_frame_tx = w1_frame_tx.clone(); // Clone Sender for use in closure
        // let moving_spheres = moving_spheres; // Copy array of entities
        // let red_light = red_light; // Copy entity
        // let blue_light = blue_light; // Copy entity

        move |sa| {
            std::boxed::Box::pin({
                let value = w1_frame_tx.clone();
                async move {
                    value
                        .send(sa.dt)
                        .map_err(|_| SystemError::Other("Failed to send dt".into()))?;

                    let circle_speed = 8.0f32;
                    let light_speed_multiplier = 3.0f32;

                    // Move the spheres in a circle considering accumulated time
                    for sphere in moving_spheres.iter() {
                        if let Some(pos3) = sa.world.get_component::<Pos3>(*sphere) {
                            let mut wlock_pos3 = pos3.write().unwrap();
                            wlock_pos3.pos = cgmath::Quaternion::from_axis_angle(
                                (0.0, 1.0, 0.0).into(),
                                cgmath::Deg(PI * sa.dt.as_secs_f32() * circle_speed),
                            ) * wlock_pos3.pos;
                        }
                    }

                    // Handle lights movement
                    if let Some(pos3) = sa.world.get_component::<Pos3>(red_light) {
                        let mut wlock_pos3 = pos3.write().unwrap();

                        wlock_pos3.pos = cgmath::Quaternion::from_axis_angle(
                            (0.0, 1.0, 0.0).into(),
                            cgmath::Deg(
                                PI * sa.dt.as_secs_f32() * circle_speed * light_speed_multiplier,
                            ),
                        ) * wlock_pos3.pos;
                    }

                    if let Some(pos3) = sa.world.get_component::<Pos3>(blue_light) {
                        let mut wlock_pos3 = pos3.write().unwrap();

                        wlock_pos3.pos = cgmath::Quaternion::from_axis_angle(
                            (0.0, 1.0, 0.0).into(),
                            cgmath::Deg(
                                PI * sa.dt.as_secs_f32() * circle_speed * light_speed_multiplier,
                            ),
                        ) * wlock_pos3.pos;
                    }

                    Ok(())
                }
            })
        }
    });

    // Run the application
    app.run().await
}
