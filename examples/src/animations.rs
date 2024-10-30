use cgmath::{Euler, One, Quaternion, Rad, Rotation3};
use core::panic;
use egui::Align2;
use gears::prelude::*;
use log::LevelFilter;
use std::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the logger
    let mut env_builder = env_logger::Builder::new();
    env_builder.filter_level(LevelFilter::Info);
    env_builder.filter_module("wgpu_core::device::resource", log::LevelFilter::Warn);
    env_builder.init();

    let mut app = GearsApp::default();

    // ! Entities
    // Add FPS camera
    new_entity!(
        app,
        components::Name("FPS Camera"),
        components::transform::Pos3::new(cgmath::Vector3::new(30.0, 20.0, 30.0,)),
        components::Camera::FPS {
            look_at: cgmath::Point3::new(0.0, 0.0, 0.0),
            speed: 10.0,
            sensitivity: 0.5,
            keycodes: components::CameraKeycodes::default(),
        }
    );

    // Use the entity builder
    app.new_entity() // Add ambient light
        .add_component(components::Name("Ambient Light"))
        .add_component(components::light::Light::Ambient { intensity: 0.05 })
        .add_component(components::transform::Pos3::new(cgmath::Vector3::new(
            0.0, 50.0, 0.0,
        )))
        .new_entity() // Add directional light
        .add_component(components::Name("Directional Light"))
        .add_component(components::light::Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.3,
        })
        .add_component(components::transform::Pos3::new(cgmath::Vector3::new(
            30.0, 30.0, 30.0,
        )))
        .new_entity() // Add a green light
        .add_component(components::Name("Green Light"))
        .add_component(components::light::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.8, 0.0],
            intensity: 0.6,
        })
        .add_component(components::transform::Pos3::new(cgmath::Vector3::new(
            -4.0, 4.0, 4.0,
        )))
        .build();

    let animated_cube = new_entity!(
        app,
        components::Name("test"),
        components::model::ModelSource::Gltf("res/gltf/cube/AnimatedCube.gltf"),
        components::model::StaticModel {
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::one(),
        },
    );

    let animated_cube = new_entity!(
        app,
        components::Name("test"),
        components::model::ModelSource::Gltf("res/gltf/helmet/DamagedHelmet.gltf"),
        components::model::StaticModel {
            position: cgmath::Vector3::new(0.0, 5.0, 0.0),
            rotation: Quaternion::from_angle_x(cgmath::Rad(90.0)),
        },
    );

    // Add a sphere and get the Entity for reference
    let sphere_entity = new_entity!(
        app,
        components::Name("Sphere1"),
        components::model::ModelSource::Obj("res/models/sphere/sphere.obj"),
        components::model::StaticModel {
            position: cgmath::Vector3::new(0.0, 0.0, 5.0),
            rotation: Quaternion::one(),
        },
    );

    // let avo = new_entity!(
    //     app,
    //     components::Name("avo"),
    //     components::model::ModelSource::Gltf("res/animated/avocado/Avocado.gltf"),
    //     components::model::StaticModel {
    //         position: cgmath::Vector3::new(0.0, 15.0, 0.0),
    //         rotation: Quaternion::one(),
    //     },
    // );

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
                ui.end_row();
            });
    }));

    // Use the update loop to spin the sphere
    app.update_loop(move |ecs, dt| {
        // Send the frame time to the custom window
        w1_frame_tx.send(dt).unwrap();
    })
    .await?;

    app.run().await
}
