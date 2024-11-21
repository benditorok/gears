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
    /*
       pub pos3: Option<Pos3>,
       pub model_source: Option<ModelSource>,
       pub movement_controller: Option<MovementController>,
       pub view_controller: Option<ViewController>,
       pub rigidbody: Option<RigidBody>,
    */
    app.add_component(components::misc::PlayerMarker);
    app.add_component(player_prefab.pos3.take().unwrap());
    app.add_component(player_prefab.model_source.take().unwrap());
    app.add_component(player_prefab.movement_controller.take().unwrap());
    app.add_component(player_prefab.view_controller.take().unwrap());
    app.add_component(player_prefab.rigidbody.take().unwrap());

    app.build();

    // Run the application
    app.run().await
}
