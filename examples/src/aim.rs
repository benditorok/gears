use cgmath::{One, Rotation3};
use ecs::traits::Prefab;
use egui::Align2;
use gears::prelude::*;
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
                ui.end_row();
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
        RigidBody::new_static(CollisionBox {
            min: cgmath::Vector3::new(-50.0, -0.1, -50.0),
            max: cgmath::Vector3::new(50.0, 0.1, 50.0),
        }),
        Pos3::new(cgmath::Vector3::new(0.0, -1.0, 0.0)),
        ModelSource::Obj("res/models/plane/plane.obj"),
    );
    // * ENDREGION

    // * Player
    let mut player_prefab = prefabs::Player::default();
    app.new_entity();
    app.add_component(PlayerMarker);
    app.add_component(player_prefab.pos3.take().unwrap());
    app.add_component(player_prefab.model_source.take().unwrap());
    app.add_component(player_prefab.movement_controller.take().unwrap());
    app.add_component(player_prefab.view_controller.take().unwrap());
    app.add_component(player_prefab.rigidbody.take().unwrap());
    app.build();
    // * Player

    // Add 5 spheres in a circle
    let mut moving_spheres: [ecs::Entity; 5] = [ecs::Entity::new(0); 5];
    for (i, sphere) in moving_spheres.iter_mut().enumerate() {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 5.0;
        let x = angle.cos() * 10.0;
        let z = angle.sin() * 10.0;

        let name = format!("Sphere_circle{}", i);

        let sphere_entity = new_entity!(
            app,
            StaticModelMarker,
            Name(Box::leak(name.into_boxed_str())),
            ModelSource::Obj("res/models/sphere/sphere.obj"),
            Pos3::new(cgmath::Vector3::new(x, 1.0, z)),
        );

        *sphere = sphere_entity;
    }

    // Update loop
    app.update_loop(move |ecs, dt| {
        // Send the frame time to the custom window
        w1_frame_tx.send(dt).unwrap();

        // ! Here we are inside a loop, so this has to lock on all iterations.
        let ecs = ecs.lock().unwrap();
        let circle_speed = 8.0f32;
        let light_speed_multiplier = 3.0f32;

        // Move the spheres in a circle considering accumulated time
        for sphere in moving_spheres.iter() {
            if let Some(pos3) = ecs.get_component_from_entity::<Pos3>(*sphere) {
                let mut wlock_pos3 = pos3.write().unwrap();

                let position = wlock_pos3.pos;

                wlock_pos3.pos = cgmath::Quaternion::from_axis_angle(
                    (0.0, 1.0, 0.0).into(),
                    cgmath::Deg(PI * dt.as_secs_f32() * circle_speed),
                ) * position;
            }
        }
        // Move the red and blue lights in a circle considering accumulated time
        if let Some(pos) = ecs.get_component_from_entity::<Pos3>(red_light) {
            let mut pos3 = pos.write().unwrap();

            pos3.pos = cgmath::Quaternion::from_axis_angle(
                (0.0, 1.0, 0.0).into(),
                cgmath::Deg(PI * dt.as_secs_f32() * circle_speed * light_speed_multiplier),
            ) * pos3.pos;
        }

        if let Some(pos) = ecs.get_component_from_entity::<Pos3>(blue_light) {
            let mut pos3 = pos.write().unwrap();

            pos3.pos = cgmath::Quaternion::from_axis_angle(
                (0.0, 1.0, 0.0).into(),
                cgmath::Deg(PI * dt.as_secs_f32() * circle_speed * light_speed_multiplier),
            ) * pos3.pos;
        }
    })
    .await?;

    // Run the application
    app.run().await
}
