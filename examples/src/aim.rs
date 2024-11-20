use cgmath::One;
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
    // Add FPS camera
    new_entity!(
        app,
        components::Name("FPS Camera"),
        components::transforms::Pos3::new(cgmath::Vector3::new(30.0, 20.0, 30.0,)),
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
        components::transforms::Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0))
    );

    // Add directional light
    new_entity!(
        app,
        components::Name("Directional Light"),
        components::lights::Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.4,
        },
        components::transforms::Pos3::new(cgmath::Vector3::new(30.0, 30.0, 30.0,))
    );

    // Plane
    new_entity!(
        app,
        components::Name("Plane"),
        components::Marker::RigidBody,
        components::physics::RigidBody::new_static(components::physics::CollisionBox {
            min: cgmath::Vector3::new(-50.0, -0.1, -50.0),
            max: cgmath::Vector3::new(50.0, 0.1, 50.0),
        }),
        components::transforms::Pos3::new(cgmath::Vector3::new(0.0, -1.0, 0.0)),
        components::models::ModelSource::Obj("res/models/plane/plane.obj"),
    );
    // * ENDREGION

    // * Player
    let player_prefab = components::prefabs::Player::default();
    let player_components = player_prefab.unpack_prefab();
    let player = new_entity!(app, player_components);

    // Run the application
    app.run().await
}
