use cgmath::Vector3;
use egui::Align2;
use gears_app::prelude::*;
use log::LevelFilter;
use std::sync::mpsc;

#[tokio::main]
async fn main() -> EngineResult<()> {
    // Initialize the logger
    let mut env_builder = env_logger::Builder::new();
    env_builder.filter_level(LevelFilter::Info);
    env_builder.filter_module("wgpu_core::device::resource", log::LevelFilter::Warn);
    env_builder.init();

    let mut app = GearsApp::default();

    // Custom window for physics info
    let (w1_frame_tx, w1_frame_rx) = mpsc::channel::<Dt>();
    app.add_window(Box::new(move |ui| {
        egui::Window::new("Physics Demo")
            .default_open(true)
            .max_width(300.0)
            .max_height(400.0)
            .resizable(true)
            .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .show(ui, |ui| {
                if let Ok(dt) = w1_frame_rx.try_recv() {
                    ui.label(format!("Frame time: {:.2} ms", dt.as_secs_f32() * 1000.0));
                    ui.label(format!("FPS: {:.0}", 1.0 / dt.as_secs_f32()));
                }

                ui.separator();
                ui.label("Physics Simulation Demo");
                ui.label("Watch objects fall and interact!");
                ui.separator();
                ui.label("Controls:");
                ui.label("WASD - Move camera");
                ui.label("Mouse - Look around");
                ui.label("Space - Move up");
                ui.label("Shift - Move down");
                ui.label("Alt - Toggle cursor lock");
                ui.label("F1 - Toggle collider wireframes");
            });
    }));

    // Add ambient light
    new_entity!(
        app,
        LightMarker,
        Pos3::new(Vector3::new(0.0, 0.0, 0.0)),
        Name("Ambient Light"),
        Light::Ambient { intensity: 0.15 },
    );

    // Add directional light (sun)
    new_entity!(
        app,
        LightMarker,
        Name("Sun"),
        Pos3::new(Vector3::new(30.0, 40.0, 30.0)),
        Light::Directional {
            direction: [-0.5, -0.8, -0.3],
            intensity: 0.7,
        },
    );

    // Ground plane
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

    // Falling crates - create a grid of crates at different heights
    let crate_grid = vec![
        // Row 1
        (-20.0, 15.0, -20.0),
        (-20.0, 18.0, -10.0),
        (-20.0, 21.0, 0.0),
        (-20.0, 24.0, 10.0),
        (-20.0, 27.0, 20.0),
        // Row 2
        (-10.0, 12.0, -20.0),
        (-10.0, 16.0, -10.0),
        (-10.0, 20.0, 0.0),
        (-10.0, 24.0, 10.0),
        (-10.0, 28.0, 20.0),
        // Row 3 (center)
        (0.0, 10.0, -20.0),
        (0.0, 15.0, -10.0),
        (0.0, 20.0, 0.0),
        (0.0, 25.0, 10.0),
        (0.0, 30.0, 20.0),
        // Row 4
        (10.0, 14.0, -20.0),
        (10.0, 17.0, -10.0),
        (10.0, 21.0, 0.0),
        (10.0, 25.0, 10.0),
        (10.0, 29.0, 20.0),
        // Row 5
        (20.0, 13.0, -20.0),
        (20.0, 19.0, -10.0),
        (20.0, 22.0, 0.0),
        (20.0, 26.0, 10.0),
        (20.0, 31.0, 20.0),
    ];

    for (x, y, z) in crate_grid {
        new_entity!(
            app,
            RigidBodyMarker,
            Name("Crate"),
            RigidBody::new(
                60.0,
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                AABBCollisionBox {
                    min: Vector3::new(-1.5, -1.5, -1.5),
                    max: Vector3::new(1.5, 1.5, 1.5),
                },
            ),
            Pos3::new(Vector3::new(x, y, z)),
            ModelSource::Gltf("gltf/low_poly_wooden_box/scene.gltf"),
        );
    }

    // Falling spheres - scattered at various heights
    let sphere_drops = vec![
        (-25.0, 35.0, -15.0, 2.0),
        (-18.0, 40.0, 5.0, 1.5),
        (-12.0, 28.0, -25.0, 3.0),
        (-5.0, 45.0, 15.0, 1.0),
        (0.0, 50.0, -5.0, 2.5),
        (5.0, 32.0, -18.0, 1.8),
        (12.0, 38.0, 8.0, 2.2),
        (18.0, 42.0, -12.0, 1.3),
        (25.0, 36.0, 18.0, 2.8),
        (-22.0, 48.0, 22.0, 1.6),
        (22.0, 44.0, -22.0, 1.4),
        (-8.0, 52.0, -8.0, 1.2),
        (8.0, 34.0, 8.0, 2.4),
        (15.0, 46.0, -2.0, 1.7),
        (-15.0, 30.0, 12.0, 2.1),
    ];

    for (x, y, z, mass) in sphere_drops {
        new_entity!(
            app,
            RigidBodyMarker,
            Name("Sphere"),
            Pos3::new(Vector3::new(x, y, z)),
            RigidBody::new(
                mass,
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, -10.0, 0.0),
                AABBCollisionBox {
                    min: Vector3::new(-1.0, -1.0, -1.0),
                    max: Vector3::new(1.0, 1.0, 1.0),
                },
            ),
            ModelSource::Obj("models/sphere/sphere.obj"),
        );
    }

    // Some projectile spheres with initial velocity
    let projectiles = vec![
        (-40.0, 10.0, 0.0, 35.0, 8.0, 0.0),
        (40.0, 12.0, 0.0, -30.0, 5.0, 5.0),
        (0.0, 15.0, -40.0, 0.0, 6.0, 32.0),
        (0.0, 18.0, 40.0, 5.0, 4.0, -28.0),
        (-30.0, 14.0, -30.0, 25.0, 7.0, 25.0),
        (30.0, 16.0, 30.0, -22.0, 6.0, -22.0),
    ];

    for (px, py, pz, vx, vy, vz) in projectiles {
        new_entity!(
            app,
            RigidBodyMarker,
            Name("Projectile"),
            Pos3::new(Vector3::new(px, py, pz)),
            RigidBody::new(
                1.5,
                Vector3::new(vx, vy, vz),
                Vector3::new(0.0, -10.0, 0.0),
                AABBCollisionBox {
                    min: Vector3::new(-1.0, -1.0, -1.0),
                    max: Vector3::new(1.0, 1.0, 1.0),
                },
            ),
            ModelSource::Obj("models/sphere/sphere.obj"),
        );
    }

    // Add more crates falling from extreme heights
    let high_crates = vec![
        (5.0, 60.0, 5.0),
        (-5.0, 65.0, -5.0),
        (3.0, 70.0, -3.0),
        (-3.0, 75.0, 3.0),
        (0.0, 80.0, 0.0),
    ];

    for (x, y, z) in high_crates {
        new_entity!(
            app,
            RigidBodyMarker,
            Name("High Crate"),
            RigidBody::new(
                55.0,
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                AABBCollisionBox {
                    min: Vector3::new(-1.5, -1.5, -1.5),
                    max: Vector3::new(1.5, 1.5, 1.5),
                },
            ),
            Pos3::new(Vector3::new(x, y, z)),
            ModelSource::Gltf("gltf/low_poly_wooden_box/scene.gltf"),
        );
    }

    // Add FPS camera positioned to view the action
    new_entity!(
        app,
        CameraMarker,
        Name("FPS Camera"),
        Pos3::new(Vector3::new(0.0, 25.0, 70.0)),
        ViewController::new_look_at(
            cgmath::Point3::new(0.0, 25.0, 70.0),
            cgmath::Point3::new(0.0, 10.0, 0.0),
            0.8,
            0.0,
        ),
        MovementController::default(),
    );

    // System to send frame data to UI
    async_system!(app, "frame_stats", (w1_frame_tx), |_world, dt| {
        w1_frame_tx
            .send(dt)
            .map_err(|_| SystemError::Other("Failed to send dt".into()))?;
        Ok(())
    });

    // Run the application
    app.run()
}
