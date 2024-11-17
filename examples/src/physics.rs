use cgmath::One;
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

    // Add ambient light
    new_entity!(
        app,
        components::Name("Ambient Light"),
        components::light::Light::Ambient { intensity: 0.05 },
        components::transform::Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0))
    );

    // Add directional light
    new_entity!(
        app,
        components::Name("Directional Light"),
        components::light::Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.3,
        },
        components::transform::Pos3::new(cgmath::Vector3::new(30.0, 30.0, 30.0,))
    );

    // Physics Body 1
    new_entity!(
        app,
        components::Name("Moving sphere"),
        components::physics::PhysicsBody::new(
            cgmath::Vector3::new(40.0, 0.0, 0.0),
            cgmath::Quaternion::one(),
            1.0,
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(-15.0, 0.0, 0.0), // * Constant acceleration
            components::physics::CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        ),
        components::model::ModelSource::Obj("res/models/sphere/sphere.obj"),
    );

    // Physics Body 2
    new_entity!(
        app,
        components::Name("Moving sphere"),
        components::physics::PhysicsBody::new(
            cgmath::Vector3::new(-50.0, 0.0, 0.0),
            cgmath::Quaternion::one(),
            1.0,
            cgmath::Vector3::new(100.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            components::physics::CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        ),
        components::model::ModelSource::Obj("res/models/sphere/sphere.obj"),
    );

    new_entity!(
        app,
        components::Name("Cube"),
        components::transform::Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0)),
        components::physics::PhysicsBody::new(
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            cgmath::Quaternion::one(),
            1.0,
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            components::physics::CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        ),
        components::model::ModelSource::Obj("res/models/cube/cube.obj"),
    );

    // Bouncing sphere
    new_entity!(
        app,
        components::Name("Static cube"),
        components::physics::PhysicsBody::new_static(
            cgmath::Vector3::new(5.0, 0.0, 20.0),
            cgmath::Quaternion::one(),
            components::physics::CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        ),
        components::model::ModelSource::Obj("res/models/cube/cube.obj"),
    );

    new_entity!(
        app,
        components::Name("Falling sphere"),
        components::physics::PhysicsBody::new(
            cgmath::Vector3::new(5.0, 20.0, 20.0),
            cgmath::Quaternion::one(),
            0.1,
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, -10.0, 0.0),
            components::physics::CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        ),
        components::model::ModelSource::Obj("res/models/sphere/sphere.obj"),
    );

    // Falling sphere bouncing off into the void
    new_entity!(
        app,
        components::Name("Static cube"),
        components::physics::PhysicsBody::new_static(
            cgmath::Vector3::new(0.0, 0.0, 20.0),
            cgmath::Quaternion::one(),
            components::physics::CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        ),
        components::model::ModelSource::Obj("res/models/cube/cube.obj"),
    );

    new_entity!(
        app,
        components::Name("Falling sphere"),
        components::physics::PhysicsBody::new(
            cgmath::Vector3::new(0.0, 20.0, 20.0),
            cgmath::Quaternion::one(),
            0.1,
            cgmath::Vector3::new(0.0, 0.0, 1.0),
            cgmath::Vector3::new(0.0, -10.0, 0.0),
            components::physics::CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        ),
        components::model::ModelSource::Obj("res/models/sphere/sphere.obj"),
    );

    // Plane
    new_entity!(
        app,
        components::Name("Plane"),
        components::physics::PhysicsBody::new_static(
            cgmath::Vector3::new(0.0, -3.0, 0.0),
            cgmath::Quaternion::one(),
            components::physics::CollisionBox {
                min: cgmath::Vector3::new(-50.0, -1.0, -50.0),
                max: cgmath::Vector3::new(50.0, 1.0, 50.0),
            },
        ),
        components::model::ModelSource::Obj("res/models/plane/plane.obj"),
    );

    new_entity!(
        app,
        components::Name("Falling sphere"),
        components::physics::PhysicsBody::new(
            cgmath::Vector3::new(10.0, 20.0, 20.0),
            cgmath::Quaternion::one(),
            0.1,
            cgmath::Vector3::new(0.0, 0.0, 1.0),
            cgmath::Vector3::new(0.0, -10.0, 0.0),
            components::physics::CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        ),
        components::model::ModelSource::Obj("res/models/sphere/sphere.obj"),
    );

    // Run the application
    app.run().await
}
