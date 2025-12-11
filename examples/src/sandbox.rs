use cgmath::Rotation3;
use egui::Align2;
use gears_app::prelude::*;
use log::LevelFilter;
use std::f32::consts::PI;
use std::sync::mpsc;

#[tokio::main]
async fn main() -> EngineResult<()> {
    // Initialize the logger
    let mut env_builder = env_logger::Builder::new();
    env_builder.filter_level(LevelFilter::Info);
    env_builder.filter_module("wgpu_core::device::resource", log::LevelFilter::Warn);
    env_builder.init();

    let mut app = GearsApp::default();

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
        Pos3::new(cgmath::Vector3::new(30.0, 30.0, 30.0,))
    );

    // Add moving red light
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

    // Add moving blue light
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

    // Static red light
    new_entity!(
        app,
        LightMarker,
        Name("R"),
        Pos3::new(cgmath::Vector3::new(40.0, 5.0, 0.0)),
        Light::PointColoured {
            radius: 10.0,
            color: [1.0, 0.0, 0.0],
            intensity: 1.0,
        },
    );

    // Static green light
    new_entity!(
        app,
        LightMarker,
        Name("G"),
        Pos3::new(cgmath::Vector3::new(30.0, 5.0, 0.0)),
        Light::PointColoured {
            radius: 10.0,
            color: [0.0, 1.0, 0.0],
            intensity: 1.0,
        },
    );

    // Static blue light
    new_entity!(
        app,
        LightMarker,
        Name("B"),
        Pos3::new(cgmath::Vector3::new(20.0, 5.0, 0.0)),
        Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.0, 1.0],
            intensity: 1.0,
        },
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

    // Player with a camera
    let player = PlayerPrefab::from_prefab(&mut app, PlayerPrefab::default());

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

    // Add a sphere that can be pushed around (dynamic rigidbody)
    new_entity!(
        app,
        RigidBodyMarker,
        Name("Pushable Sphere"),
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
    );

    // Add static cubes
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
        Pos3::new(cgmath::Vector3::new(20.0, 0.0, -20.0)),
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
    new_entity!(
        app,
        RigidBodyMarker,
        Name("Static cube"),
        Pos3::new(cgmath::Vector3::new(-20.0, 0.0, 20.0)),
        RigidBody::new_static(AABBCollisionBox {
            min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
            max: cgmath::Vector3::new(1.0, 1.0, 1.0),
        },),
        ModelSource::Obj("models/cube/cube.obj"),
    );

    // Custom window to get informations about the renderer
    let (w1_frame_tx, w1_frame_rx) = mpsc::channel::<Dt>();
    app.add_window(Box::new(move |ui| {
        egui::Window::new("Renderer info")
            .default_open(true)
            .max_width(200.0)
            .max_height(600.0)
            .default_width(200.0)
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
                ui.label("F1 - Toggle debug mode");
            });
    }));

    // Create a system to handle the movement of spheres and lights
    async_system!(
        app,
        "handle_entity_movements",
        (w1_frame_tx),
        |world, dt| {
            w1_frame_tx
                .send(dt)
                .map_err(|_| SystemError::Other("Failed to send dt.".into()))?;

            // Move the spheres in a circle
            let circle_speed = 8.0f32;
            let light_speed_multiplier = 3.0f32;
            for sphere in moving_spheres.iter() {
                if let Some(pos3) = world.get_component::<Pos3>(*sphere) {
                    let mut wlock_pos3 = pos3.write().unwrap();
                    wlock_pos3.pos = cgmath::Quaternion::from_axis_angle(
                        (0.0, 1.0, 0.0).into(),
                        cgmath::Deg(PI * dt.as_secs_f32() * circle_speed),
                    ) * wlock_pos3.pos;
                }
            }

            // Handle the movement of lights
            if let Some(pos3) = world.get_component::<Pos3>(red_light) {
                let mut wlock_pos3 = pos3.write().unwrap();

                wlock_pos3.pos = cgmath::Quaternion::from_axis_angle(
                    (0.0, 1.0, 0.0).into(),
                    cgmath::Deg(PI * dt.as_secs_f32() * circle_speed * light_speed_multiplier),
                ) * wlock_pos3.pos;
            }
            if let Some(pos3) = world.get_component::<Pos3>(blue_light) {
                let mut wlock_pos3 = pos3.write().unwrap();

                wlock_pos3.pos = cgmath::Quaternion::from_axis_angle(
                    (0.0, 1.0, 0.0).into(),
                    cgmath::Deg(PI * dt.as_secs_f32() * circle_speed * light_speed_multiplier),
                ) * wlock_pos3.pos;
            }

            Ok(())
        }
    );

    // Run the application
    app.run()
}
