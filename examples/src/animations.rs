use egui::Align2;
use gears_app::{prelude::*, systems};
use log::LevelFilter;
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

    let (w1_frame_tx, w1_frame_rx) = mpsc::channel::<Dt>();

    // ! Entities
    // Add FPS camera
    new_entity!(
        app,
        CameraMarker,
        Name("FPS Camera"),
        Pos3::new(cgmath::Vector3::new(30.0, 20.0, 30.0,)),
        ViewController::new_look_at(
            cgmath::Point3::new(30.0, 20.0, 30.0),
            cgmath::Point3::new(0.0, 0.0, 0.0),
            0.8,
            0.0,
        ),
        MovementController::default(),
    );

    // Use the entity builder
    app.new_entity() // Add ambient light
        .add_component(LightMarker)
        .add_component(Name("Ambient Light"))
        .add_component(Light::Ambient { intensity: 0.05 })
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

    let animated_cube = new_entity!(
        app,
        StaticModelMarker,
        Name("test"),
        ModelSource::Gltf("gltf/cube/AnimatedCube.gltf"),
        Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0)),
        AnimationQueue::default(),
    );

    new_entity!(
        app,
        StaticModelMarker,
        Name("test"),
        ModelSource::Gltf("gltf/helmet/DamagedHelmet.gltf"),
        Pos3::new(cgmath::Vector3::new(0.0, 5.0, 0.0)),
    );

    new_entity!(
        app,
        StaticModelMarker,
        Name("Sphere1"),
        ModelSource::Obj("models/sphere/sphere.obj"),
        Pos3::new(cgmath::Vector3::new(0.0, 0.0, 5.0)),
    );

    // ! Custom windows
    // Information about the renderer
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
                ui.end_row();
            });
    }));

    let time_started = std::time::Instant::now();

    let update_sys = systems::async_system("update", move |sa| {
        Box::pin({
            let w1_frame_tx = w1_frame_tx.clone();

            async move {
                let (world, dt) = match sa {
                    SystemAccessors::External { world, dt } => (world, dt),
                    _ => return Ok(()),
                };

                if time_started.elapsed().as_secs() % 3 == 0 {
                    let animation_queue = world
                        .get_component::<AnimationQueue>(animated_cube)
                        .unwrap();

                    animation_queue
                        .write()
                        .unwrap()
                        .push("animation_AnimatedCube");
                }

                Ok(())
            }
        })
    });

    app.add_async_system(update_sys);

    // Run the application
    app.run().await
}
