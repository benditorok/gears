use cgmath::{One, Rotation3};
use gears::prelude::*;
use log::LevelFilter;
use std::f32::consts::PI;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the logger
    let mut env_builder = env_logger::Builder::new();
    env_builder.filter_level(LevelFilter::Info);
    env_builder.filter_module("wgpu_core::device::resource", log::LevelFilter::Warn);
    env_builder.init();

    let mut app = GearsApp::default();

    // Add FPS camera
    new_entity!(
        app,
        components::Name("FPS Camera"),
        components::transforms::Pos3::new(cgmath::Vector3::new(20.0, 10.0, 20.0,)),
        components::Camera::Dynamic {
            look_at: cgmath::Point3::new(0.0, 0.0, 0.0),
            speed: 10.0,
            sensitivity: 0.5,
            keycodes: components::MovementKeycodes::default(),
        }
    );

    // Add ambient light
    new_entity!(
        app,
        components::Name("Ambient Light"),
        components::lights::Light::Ambient { intensity: 0.05 },
        components::transforms::Pos3::new(cgmath::Vector3::new(0.0, 50.0, 0.0))
    );

    // Add directional light
    new_entity!(
        app,
        components::Name("Directional Light"),
        components::lights::Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.3,
        },
        components::transforms::Pos3::new(cgmath::Vector3::new(30.0, 30.0, 30.0,))
    );

    // * Add moving red light
    let red_light = new_entity!(
        app,
        components::Name("Red Light"),
        components::lights::Light::PointColoured {
            radius: 10.0,
            color: [0.8, 0.0, 0.0],
            intensity: 1.0,
        },
        components::transforms::Pos3::new(cgmath::Vector3::new(15.0, 5.0, 0.0))
    );

    // * Add moving blue light
    let blue_light = new_entity!(
        app,
        components::Name("Blue Light"),
        components::lights::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.0, 0.8],
            intensity: 1.0,
        },
        components::transforms::Pos3::new(cgmath::Vector3::new(-15.0, 5.0, 0.0))
    );

    // Red light
    new_entity!(
        app,
        components::Name("R"),
        components::lights::Light::PointColoured {
            radius: 10.0,
            color: [1.0, 0.0, 0.0],
            intensity: 1.0,
        },
        components::transforms::Pos3::new(cgmath::Vector3::new(0.0, 5.0, -20.0))
    );

    // Green light
    new_entity!(
        app,
        components::Name("G"),
        components::lights::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 1.0, 0.0],
            intensity: 1.0,
        },
        components::transforms::Pos3::new(cgmath::Vector3::new(0.0, 5.0, -30.0))
    );

    // Blue light
    new_entity!(
        app,
        components::Name("B"),
        components::lights::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.0, 1.0],
            intensity: 1.0,
        },
        components::transforms::Pos3::new(cgmath::Vector3::new(0.0, 5.0, -40.0))
    );

    // * If you do not need the IDs of the entities you can chain them together
    app.new_entity() // Cube 1
        .add_component(components::Name("Cube1"))
        .add_component(components::models::ModelSource::Obj("models/cube/cube.obj"))
        .add_component(components::models::StaticModel {
            position: cgmath::Vector3::new(10.0, 0.0, 10.0),
            rotation: cgmath::Quaternion::one(),
        })
        .new_entity() // Cube 2
        .add_component(components::Name("Cube2"))
        .add_component(components::models::ModelSource::Obj("models/cube/cube.obj"))
        .add_component(components::models::StaticModel {
            position: cgmath::Vector3::new(10.0, 0.0, -10.0),
            rotation: cgmath::Quaternion::one(),
        })
        .new_entity() // Cube 3
        .add_component(components::Name("Cube3"))
        .add_component(components::models::ModelSource::Obj("models/cube/cube.obj"))
        .add_component(components::models::StaticModel {
            position: cgmath::Vector3::new(-10.0, 0.0, -10.0),
            rotation: cgmath::Quaternion::one(),
        })
        .new_entity() // Cube 4
        .add_component(components::Name("Cube4"))
        .add_component(components::models::ModelSource::Obj("models/cube/cube.obj"))
        .add_component(components::models::StaticModel {
            position: cgmath::Vector3::new(-10.0, 0.0, 10.0),
            rotation: cgmath::Quaternion::one(),
        })
        .build();

    // Center sphere
    new_entity!(
        app,
        components::Name("Sphere1"),
        components::models::ModelSource::Obj("models/sphere/sphere.obj"),
        components::models::StaticModel {
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Quaternion::one(),
        },
        components::transforms::Flip::Vertical
    );

    // Plane
    new_entity!(
        app,
        components::Name("Plane"),
        components::models::ModelSource::Obj("models/plane/plane.obj"),
        components::models::StaticModel {
            position: cgmath::Vector3::new(0.0, -3.0, 0.0),
            rotation: cgmath::Quaternion::one(),
        },
    );

    // Add 5 spheres in a circle
    let mut moving_spheres: [ecs::Entity; 5] = [ecs::Entity::new(0); 5];
    for (i, sphere) in moving_spheres.iter_mut().enumerate() {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 5.0;
        let x = angle.cos() * 10.0;
        let z = angle.sin() * 10.0;

        let name = format!("Sphere_circle{}", i);

        let sphere_entity = new_entity!(
            app,
            components::Name(Box::leak(name.into_boxed_str())),
            components::models::ModelSource::Obj("models/sphere/sphere.obj"),
            components::models::StaticModel {
                position: cgmath::Vector3::new(x, 0.0, z),
                rotation: cgmath::Quaternion::one(),
            },
        );

        *sphere = sphere_entity;
    }

    // Update loop
    app.update_loop(move |ecs, dt| {
        // ! Here we are inside a loop, so this has to lock on all iterations.
        let ecs = ecs.lock().unwrap();
        let circle_speed = 8.0f32;
        let light_speed_multiplier = 3.0f32;

        // Move the spheres in a circle considering accumulated time
        for sphere in moving_spheres.iter() {
            if let Some(static_model) =
                ecs.get_component_from_entity::<components::models::StaticModel>(*sphere)
            {
                let mut wlock_static_model = static_model.write().unwrap();

                let position = wlock_static_model.position;

                wlock_static_model.position = cgmath::Quaternion::from_axis_angle(
                    (0.0, 1.0, 0.0).into(),
                    cgmath::Deg(PI * dt.as_secs_f32() * circle_speed),
                ) * position;
            }
        }
        // Move the red and blue lights in a circle considering accumulated time
        if let Some(pos) = ecs.get_component_from_entity::<components::transforms::Pos3>(red_light)
        {
            let mut pos3 = pos.write().unwrap();

            pos3.pos = cgmath::Quaternion::from_axis_angle(
                (0.0, 1.0, 0.0).into(),
                cgmath::Deg(PI * dt.as_secs_f32() * circle_speed * light_speed_multiplier),
            ) * pos3.pos;
        }

        if let Some(pos) = ecs.get_component_from_entity::<components::transforms::Pos3>(blue_light)
        {
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
