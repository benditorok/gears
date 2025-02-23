use gears_app::prelude::*;
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

    // Add ambient light
    new_entity!(
        app,
        LightMarker,
        Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0)),
        Name("Ambient Light"),
        Light::Ambient { intensity: 0.05 },
    );

    // Add directional light
    new_entity!(
        app,
        LightMarker,
        Name("Directional Light"),
        Pos3::new(cgmath::Vector3::new(30.0, 30.0, 30.0,)),
        Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.3,
        },
    );

    // * START moving objects
    // Physics Body 1
    new_entity!(
        app,
        RigidBodyMarker,
        Name("Moving sphere"),
        Pos3::new(cgmath::Vector3::new(50.0, 0.0, 0.0)),
        RigidBody::new(
            1.0,
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(-15.0, -10.0, 0.0), // * Constant acceleration
            CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        ),
        ModelSource::Obj("models/sphere/sphere.obj"),
    );

    // Physics Body 2
    new_entity!(
        app,
        RigidBodyMarker,
        Name("Moving sphere"),
        Pos3::new(cgmath::Vector3::new(50.0, 0.0, 0.0)),
        RigidBody::new(
            1.0,
            cgmath::Vector3::new(100.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, -10.0, 0.0),
            CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        ),
        ModelSource::Obj("models/sphere/sphere.obj"),
    );

    new_entity!(
        app,
        RigidBodyMarker,
        Name("Cube"),
        Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0)),
        RigidBody::new(
            10.0,
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, -10.0, 0.0),
            CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        ),
        ModelSource::Obj("models/cube/cube.obj"),
    );
    // * END moving objects
    // Bouncing sphere
    new_entity!(
        app,
        RigidBodyMarker,
        Name("Static cube"),
        Pos3::new(cgmath::Vector3::new(5.0, 0.0, 20.0)),
        RigidBody::new_static(CollisionBox {
            min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
            max: cgmath::Vector3::new(1.0, 1.0, 1.0),
        },),
        ModelSource::Obj("models/cube/cube.obj"),
    );

    new_entity!(
        app,
        RigidBodyMarker,
        Name("Falling sphere"),
        Pos3::new(cgmath::Vector3::new(5.0, 20.0, 20.0)),
        RigidBody::new(
            1.0,
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, -5.0, 0.0),
            CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        ),
        ModelSource::Obj("models/sphere/sphere.obj"),
    );

    // Falling sphere bouncing off into the void
    new_entity!(
        app,
        RigidBodyMarker,
        Name("Static cube"),
        Pos3::new(cgmath::Vector3::new(0.0, 0.0, 20.0)),
        RigidBody::new_static(CollisionBox {
            min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
            max: cgmath::Vector3::new(1.0, 1.0, 1.0),
        },),
        ModelSource::Obj("models/cube/cube.obj"),
    );

    new_entity!(
        app,
        RigidBodyMarker,
        Name("Falling sphere"),
        Pos3::new(cgmath::Vector3::new(0.0, 20.0, 20.0)),
        RigidBody::new(
            0.1,
            cgmath::Vector3::new(0.0, 0.0, 1.0),
            cgmath::Vector3::new(0.0, -10.0, 0.0),
            CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        ),
        ModelSource::Obj("models/sphere/sphere.obj"),
    );

    // Plane
    new_entity!(
        app,
        RigidBodyMarker,
        Name("Plane"),
        Pos3::new(cgmath::Vector3::new(0.0, -3.0, 0.0)),
        RigidBody::new_static(CollisionBox {
            min: cgmath::Vector3::new(-50.0, -0.1, -50.0),
            max: cgmath::Vector3::new(50.0, 0.1, 50.0),
        },),
        ModelSource::Obj("models/plane/plane.obj"),
    );

    new_entity!(
        app,
        RigidBodyMarker,
        Name("Falling sphere"),
        Pos3::new(cgmath::Vector3::new(10.0, 20.0, 20.0)),
        RigidBody::new(
            0.1,
            cgmath::Vector3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, -10.0, 0.0),
            CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        ),
        ModelSource::Obj("models/sphere/sphere.obj"),
    );

    // Run the application
    app.run().await
}
