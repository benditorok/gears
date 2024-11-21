use std::f32::consts::PI;

use cgmath::{One, Rotation3};
use ecs::traits::Prefab;
use gears::prelude::*;
use log::LevelFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the logger
    let mut env_builder = env_logger::Builder::new();
    env_builder.filter_level(LevelFilter::Info);
    env_builder.filter_module("wgpu_core::device::resource", log::LevelFilter::Warn);
    env_builder.init();

    let mut app = GearsApp::default();

    // * REGION setup
    // // Add FPS camera
    // new_entity!(
    //     app,
    //     components::misc::Name("FPS Camera"),
    //     components::transforms::Pos3::new(cgmath::Vector3::new(30.0, 20.0, 30.0,)),
    //     components::controllers::ViewController::default()
    // );

    // Add ambient light
    new_entity!(
        app,
        components::misc::LightMarker,
        components::misc::Name("Ambient Light"),
        components::lights::Light::Ambient { intensity: 0.05 },
        components::transforms::Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0))
    );

    // Add directional light
    new_entity!(
        app,
        components::misc::LightMarker,
        components::misc::Name("Directional Light"),
        components::lights::Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.4,
        },
        components::transforms::Pos3::new(cgmath::Vector3::new(30.0, 30.0, 30.0,))
    );

    // * Add moving red light
    let red_light = new_entity!(
        app,
        components::misc::LightMarker,
        components::misc::Name("Red Light"),
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
        components::misc::LightMarker,
        components::misc::Name("Blue Light"),
        components::lights::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.0, 0.8],
            intensity: 1.0,
        },
        components::transforms::Pos3::new(cgmath::Vector3::new(-15.0, 5.0, 0.0))
    );

    // Plane
    new_entity!(
        app,
        components::misc::RigidBodyMarker,
        components::misc::Name("Plane"),
        components::physics::RigidBody::new_static(components::physics::CollisionBox {
            min: cgmath::Vector3::new(-50.0, -0.1, -50.0),
            max: cgmath::Vector3::new(50.0, 0.1, 50.0),
        }),
        components::transforms::Pos3::new(cgmath::Vector3::new(0.0, -1.0, 0.0)),
        components::models::ModelSource::Obj("res/models/plane/plane.obj"),
    );
    // * ENDREGION

    // * Player
    let mut player_prefab = components::prefabs::Player::default();
    app.new_entity();
    app.add_component(components::misc::PlayerMarker);
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
            components::misc::StaticModelMarker,
            components::misc::Name(Box::leak(name.into_boxed_str())),
            components::models::ModelSource::Obj("res/models/sphere/sphere.obj"),
            components::transforms::Pos3::new(cgmath::Vector3::new(x, 1.0, z)),
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
            if let Some(pos3) =
                ecs.get_component_from_entity::<components::transforms::Pos3>(*sphere)
            {
                let mut wlock_pos3 = pos3.write().unwrap();

                let position = wlock_pos3.pos;

                wlock_pos3.pos = cgmath::Quaternion::from_axis_angle(
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
