use app::GearsApp;
use cgmath::{One, Rotation3};
use ecs::traits::EntityBuilder;
use gears::{new_entity, prelude::*};
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
        components::transform::Pos3::new(cgmath::Vector3::new(30.0, 20.0, 30.0,)),
        components::Camera::FPS {
            look_at: cgmath::Point3::new(0.0, 0.0, 0.0),
            speed: 10.0,
            sensitivity: 0.5,
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
    let physics_body_1 = new_entity!(
        app,
        components::Name("Physics Body 2"),
        components::physics::PhysicsBody {
            position: cgmath::Vector3::new(40.0, 0.0, 0.0),
            rotation: cgmath::Quaternion::one(),
            mass: 1.0,
            velocity: cgmath::Vector3::new(0.0, 0.0, 0.0),
            acceleration: cgmath::Vector3::new(-15.0, 0.0, 0.0), // * Constant acceleration
            collision_box: components::physics::CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        },
        components::Model::Dynamic {
            obj_path: "res/models/sphere/sphere.obj"
        }
    );

    // Physics Body 2
    let physics_body_2 = new_entity!(
        app,
        components::Name("Physics Body 3"),
        components::physics::PhysicsBody {
            position: cgmath::Vector3::new(-50.0, 0.0, 0.0),
            rotation: cgmath::Quaternion::one(),
            mass: 1.0,
            velocity: cgmath::Vector3::new(100.0, 0.0, 0.0),
            acceleration: cgmath::Vector3::new(0.0, 0.0, 0.0),
            collision_box: components::physics::CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        },
        components::Model::Dynamic {
            obj_path: "res/models/sphere/sphere.obj"
        }
    );

    let cube = new_entity!(
        app,
        components::Name("Cube"),
        components::transform::Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0)),
        components::physics::PhysicsBody {
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Quaternion::one(),
            mass: 1.0,
            velocity: cgmath::Vector3::new(0.0, 0.0, 0.0),
            acceleration: cgmath::Vector3::new(0.0, 0.0, 0.0),
            collision_box: components::physics::CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        },
        components::Model::Dynamic {
            obj_path: "res/models/cube/cube.obj"
        },
    );

    new_entity!(
        app,
        components::Name("Cube"),
        components::transform::Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0)),
        components::physics::PhysicsBody {
            position: cgmath::Vector3::new(0.0, 0.0, 20.0),
            rotation: cgmath::Quaternion::one(),
            mass: 10000000.0,
            velocity: cgmath::Vector3::new(0.0, 0.0, 0.0),
            acceleration: cgmath::Vector3::new(0.0, 0.0, 0.0),
            collision_box: components::physics::CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        },
        components::Model::Dynamic {
            obj_path: "res/models/cube/cube.obj"
        },
    );

    new_entity!(
        app,
        components::Name("Physics Body 2"),
        components::physics::PhysicsBody {
            position: cgmath::Vector3::new(0.0, 20.0, 20.0),
            rotation: cgmath::Quaternion::one(),
            mass: 0.1,
            velocity: cgmath::Vector3::new(0.0, 0.0, 0.0),
            acceleration: cgmath::Vector3::new(0.0, -10.0, 0.0),
            collision_box: components::physics::CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        },
        components::Model::Dynamic {
            obj_path: "res/models/sphere/sphere.obj"
        }
    );

    // Run the application
    app.run().await
}
