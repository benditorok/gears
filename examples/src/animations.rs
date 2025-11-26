use cgmath::{Quaternion, Rotation3, Vector3};
use egui::Align2;
use gears_app::prelude::*;
use log::LevelFilter;
use std::sync::{Arc, Mutex, mpsc};

#[tokio::main]
async fn main() -> EngineResult<()> {
    // Initialize the logger
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .filter_module("wgpu_core::device::resource", log::LevelFilter::Warn)
        .init();

    let mut app = GearsApp::default();

    // ! Entities
    // Add FPS camera positioned to view all models
    new_entity!(
        app,
        CameraMarker,
        Name("FPS Camera"),
        Pos3::new(cgmath::Vector3::new(0.0, 3.0, 8.0,)),
        ViewController::new_look_at(
            cgmath::Point3::new(0.0, 3.0, 8.0),
            cgmath::Point3::new(0.0, 1.0, 0.0),
            0.5,
            0.0,
        ),
        MovementController::default(),
    );

    // Use the entity builder
    app.new_entity() // Add ambient light
        .add_component(LightMarker)
        .add_component(Name("Ambient Light"))
        .add_component(Light::Ambient { intensity: 0.1 })
        .add_component(Pos3::new(cgmath::Vector3::new(0.0, 50.0, 0.0)))
        .new_entity() // Add directional light
        .add_component(LightMarker)
        .add_component(Name("Directional Light"))
        .add_component(Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.3,
        })
        .add_component(Pos3::new(cgmath::Vector3::new(30.0, 30.0, 30.0)))
        .new_entity() // Add a green light
        .add_component(LightMarker)
        .add_component(Name("White Light"))
        .add_component(Light::PointColoured {
            radius: 10.0,
            color: [0.6, 0.6, 0.8],
            intensity: 0.4,
        })
        .add_component(Pos3::new(cgmath::Vector3::new(-4.0, 4.0, 4.0)))
        .build();

    // Add the animated cube
    let animated_cube = new_entity!(
        app,
        StaticModelMarker,
        Name("Animated Cube"),
        ModelSource::Gltf("gltf/cube/AnimatedCube.gltf"),
        Pos3::new(cgmath::Vector3::new(-3.0, 0.0, 0.0)),
        AnimationQueue::default(),
    );

    new_entity!(
        app,
        StaticModelMarker,
        Name("Helmet"),
        ModelSource::Gltf("gltf/helmet/DamagedHelmet.gltf"),
        Pos3::new(cgmath::Vector3::new(3.0, 0.0, 0.0)),
        AnimationQueue::default(),
    );

    new_entity!(
        app,
        StaticModelMarker,
        Name("Mannequin"),
        ModelSource::Gltf("gltf/sk_mannequinn/scene.gltf"),
        Pos3::new(cgmath::Vector3::new(-3.0, 0.0, 8.0)),
        Scale::Uniform(0.02)
    );

    // ! Custom windows
    // Information about the renderer
    let (w1_frame_tx, w1_frame_rx) = mpsc::channel::<Dt>();
    app.add_window(Box::new(move |ui| {
        egui::Window::new("Animation System Demo")
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
                ui.heading("Animation System Demo");

                ui.separator();
                ui.label("Controls:");
                ui.label("WASD - Move player");
                ui.label("Mouse - Look around");
                ui.label("Space - Fly up");
                ui.label("Shift - Fly down");
                ui.label("Alt - Keep the cursor within the window's bounds.");
                ui.label("Esc - Pause");
                ui.label("F1 - Toggle debug mode");
            });
    }));

    // Update entity states
    // Use accumulated game time that respects pause state
    async_system!(app, "update", (w1_frame_tx), |_world, dt| {
        // Send frame time for UI
        let _ = w1_frame_tx.send(dt);

        Ok(())
    });

    // Run gltf animations
    // Track accumulated time for GLTF animations
    let model_accumulated_time = Arc::new(Mutex::new(0.0f32));
    async_system!(
        app,
        "gltf_animations",
        (model_accumulated_time),
        |world, dt| {
            // Update accumulated time only when not paused
            let elapsed_time = {
                let mut time = model_accumulated_time.lock().unwrap();
                *time += dt.as_secs_f32();
                *time
            };

            // Animate the cube with GLTF animation every 5 seconds
            if (elapsed_time as u64) % 5 == 0 && dt.as_secs_f32() > 0.0 {
                if let Some(animation_queue) = world.get_component::<AnimationQueue>(animated_cube)
                {
                    let mut queue = animation_queue.write().unwrap();
                    if !queue.has_queued_animations() {
                        queue.push("animation_AnimatedCube".to_string());
                        queue.set_transition_duration(0.5);
                        queue.set_auto_transition(true);
                        log::info!("Started cube GLTF animation");
                    }
                }
            }

            Ok(())
        }
    );

    // Run the application
    app.run()
}
