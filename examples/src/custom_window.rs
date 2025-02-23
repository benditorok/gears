use cgmath::{Euler, Quaternion, Rad, Rotation3};
use egui::Align2;
use gears_app::prelude::*;
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
    // Add fixed camera
    new_entity!(
        app,
        CameraMarker,
        Name("Fixed Camera"),
        Pos3::new(cgmath::Vector3::new(3.0, 2.0, 3.0)),
        ViewController::new_look_at(
            cgmath::Point3::new(3.0, 2.0, 3.0),
            cgmath::Point3::new(0.0, 0.0, 0.0),
            0.0,
            0.0
        )
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
        .add_component(Name("Green Light"))
        .add_component(Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.8, 0.0],
            intensity: 0.6,
        })
        .add_component(Pos3::new(cgmath::Vector3::new(-4.0, 4.0, 4.0)))
        .build();

    // Add a sphere and get the Entity for reference
    let sphere_entity = new_entity!(
        app,
        StaticModelMarker,
        Name("Sphere1"),
        ModelSource::Obj("models/sphere/sphere.obj"),
        Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0)),
    );

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
                if let Some(sphere) = cw_ecs.get_component::<Pos3>(sphere_entity) {
                    let mut wlock_sphere = sphere.write().unwrap();
                    ui.label("Position");
                    ui.add(egui::Slider::new(&mut wlock_sphere.pos.x, -10.0..=10.0));
                    ui.add(egui::Slider::new(&mut wlock_sphere.pos.y, -10.0..=10.0));
                    ui.add(egui::Slider::new(&mut wlock_sphere.pos.z, -10.0..=10.0));
                    ui.label("Rotation");
                    let euler = Euler::from(wlock_sphere.rot);
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

                    wlock_sphere.rot = Quaternion::from(Euler {
                        x: Rad(pitch),
                        y: Rad(yaw),
                        z: Rad(roll),
                    });
                }
                ui.end_row();
            });
    }));

    // Use the update loop to spin the sphere
    app.update_loop(move |world, dt| {
        // Send the frame time to the custom window
        w1_frame_tx.send(dt).unwrap();

        let spin_speed = 0.5f32;

        if let Some(static_model) = world.get_component::<Pos3>(sphere_entity) {
            let mut wlock_static_model = static_model.write().unwrap();

            let rotation = wlock_static_model.rot;
            wlock_static_model.rot =
                Quaternion::from_angle_y(cgmath::Rad(dt.as_secs_f32() * spin_speed)) * rotation;
        }
    })
    .await?;

    app.run().await
}
