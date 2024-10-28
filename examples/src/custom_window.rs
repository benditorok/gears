use std::sync::mpsc;

use cgmath::{Euler, One, Quaternion, Rad, Rotation3};
use egui::{Align, Align2};
use gears::{core::Dt, new_entity, prelude::*};
use log::{warn, LevelFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the logger
    let mut env_builder = env_logger::Builder::new();
    env_builder.filter_level(LevelFilter::Info);
    env_builder.filter_module("wgpu_core::device::resource", log::LevelFilter::Warn);
    env_builder.init();

    let mut app = GearsApp::default();

    // ! Entities
    // Add fixed camera
    new_entity!(
        app,
        components::Name("Fixed Camera"),
        components::transform::Pos3::new(cgmath::Vector3::new(3.0, 2.0, 3.0)),
        components::Camera::Fixed {
            look_at: cgmath::Point3::new(0.0, 0.0, 0.0),
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

    // Add a sphere and get the Entity for reference
    let sphere_entity = new_entity!(
        app,
        components::Name("Sphere1"),
        components::model::ModelSource {
            obj_path: "res/models/sphere/sphere.obj",
        },
        components::model::StaticModel {
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::one(),
        },
    );

    // ! Custom windows
    let (dt_tx, dt_rx) = tokio::sync::mpsc::channel::<Dt>(10);

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

    // Move the object around
    let cw_ecs = app.get_ecs();
    app.add_window(Box::new(move |ui| {
        egui::Window::new("Sphere")
            .default_open(true)
            .max_width(1000.0)
            .max_height(800.0)
            .default_width(800.0)
            .resizable(true)
            .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .show(ui, |ui| {
                if let Some(sphere) = cw_ecs
                    .lock()
                    .unwrap()
                    .get_component_from_entity::<components::model::StaticModel>(sphere_entity)
                {
                    let mut wlock_sphere = sphere.write().unwrap();
                    ui.label("Position");
                    ui.add(egui::Slider::new(
                        &mut wlock_sphere.position.x,
                        -10.0..=10.0,
                    ));
                    ui.add(egui::Slider::new(
                        &mut wlock_sphere.position.y,
                        -10.0..=10.0,
                    ));
                    ui.add(egui::Slider::new(
                        &mut wlock_sphere.position.z,
                        -10.0..=10.0,
                    ));
                    ui.label("Rotation");
                    let euler = Euler::from(wlock_sphere.rotation);
                    let mut pitch = euler.x.0;
                    let mut yaw = euler.y.0;
                    let mut roll = euler.z.0;

                    ui.add(
                        egui::Slider::new(&mut pitch, -std::f32::consts::PI..=std::f32::consts::PI)
                            .text("Pitch"),
                    );
                    ui.add(
                        egui::Slider::new(&mut yaw, -std::f32::consts::PI..=std::f32::consts::PI)
                            .text("Yaw"),
                    );
                    ui.add(
                        egui::Slider::new(&mut roll, -std::f32::consts::PI..=std::f32::consts::PI)
                            .text("Roll"),
                    );

                    wlock_sphere.rotation = Quaternion::from(Euler {
                        x: Rad(pitch),
                        y: Rad(yaw),
                        z: Rad(roll),
                    });
                }
                ui.end_row();
            });
    }));

    // Use the update loop to spin the sphere
    app.update_loop(move |ecs, dt| {
        // Send the frame time to the custom window
        w1_frame_tx.send(dt).unwrap();

        let ecs = ecs.lock().unwrap();
        let spin_speed = 0.5f32;

        if let Some(static_model) =
            ecs.get_component_from_entity::<components::model::StaticModel>(sphere_entity)
        {
            let mut wlock_static_model = static_model.write().unwrap();

            let rotation = wlock_static_model.rotation;
            wlock_static_model.rotation =
                Quaternion::from_angle_y(cgmath::Rad(dt.as_secs_f32() * spin_speed)) * rotation;
        }
    })
    .await?;

    app.run().await
}
