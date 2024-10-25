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
        components::Pos3::new(cgmath::Vector3::new(20.0, 10.0, 20.0,)),
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
        components::Light::Ambient { intensity: 0.05 },
        components::Pos3::new(cgmath::Vector3::new(0.0, 50.0, 0.0))
    );

    // Add directional light
    new_entity!(
        app,
        components::Name("Directional Light"),
        components::Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.3,
        },
        components::Pos3::new(cgmath::Vector3::new(30.0, 30.0, 30.0,))
    );

    // * Add moving red light
    let red_light = new_entity!(
        app,
        components::Name("Red Light"),
        components::Light::PointColoured {
            radius: 10.0,
            color: [0.8, 0.0, 0.0],
            intensity: 1.0,
        },
        components::Pos3::new(cgmath::Vector3::new(15.0, 5.0, 0.0))
    );

    // * Add moving blue light
    let blue_light = new_entity!(
        app,
        components::Name("Blue Light"),
        components::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.0, 0.8],
            intensity: 1.0,
        },
        components::Pos3::new(cgmath::Vector3::new(-15.0, 5.0, 0.0))
    );

    // Red light
    new_entity!(
        app,
        components::Name("R"),
        components::Light::PointColoured {
            radius: 10.0,
            color: [1.0, 0.0, 0.0],
            intensity: 1.0,
        },
        components::Pos3::new(cgmath::Vector3::new(0.0, 5.0, -20.0))
    );

    // Green light
    new_entity!(
        app,
        components::Name("G"),
        components::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 1.0, 0.0],
            intensity: 1.0,
        },
        components::Pos3::new(cgmath::Vector3::new(0.0, 5.0, -30.0))
    );

    // Blue light
    new_entity!(
        app,
        components::Name("B"),
        components::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.0, 1.0],
            intensity: 1.0,
        },
        components::Pos3::new(cgmath::Vector3::new(0.0, 5.0, -40.0))
    );

    // * If you do not need the IDs of the entities you can chain them together
    app.new_entity() // Cube 1
        .add_component(components::Name("Cube1"))
        .add_component(components::Model::Static {
            obj_path: "res/models/cube/cube.obj",
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(10.0, 0.0, 10.0)))
        .new_entity() // Cube 2
        .add_component(components::Name("Cube2"))
        .add_component(components::Model::Static {
            obj_path: "res/models/cube/cube.obj",
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(
            10.0, 0.0, -10.0,
        )))
        .new_entity() // Cube 3
        .add_component(components::Name("Cube3"))
        .add_component(components::Model::Static {
            obj_path: "res/models/cube/cube.obj",
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(
            -10.0, 0.0, -10.0,
        )))
        .new_entity() // Cube 4
        .add_component(components::Name("Cube4"))
        .add_component(components::Model::Static {
            obj_path: "res/models/cube/cube.obj",
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(
            -10.0, 0.0, 10.0,
        )))
        .build();

    // Center sphere
    new_entity!(
        app,
        components::Name("Sphere1"),
        components::Model::Static {
            obj_path: "res/models/sphere/sphere.obj",
        },
        components::Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0)),
        components::Flip::Vertical
    );

    // Plane
    new_entity!(
        app,
        components::Name("Plane"),
        components::Model::Dynamic {
            obj_path: "res/models/plane/plane.obj",
        },
        components::Pos3::new(cgmath::Vector3::new(0.0, -3.0, 0.0)),
    );

    // Add 5 spheres in a circle
    let mut moving_spheres: [ecs::Entity; 5] = [ecs::Entity(0); 5];
    for (i, sphere) in moving_spheres.iter_mut().enumerate() {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 5.0;
        let x = angle.cos() * 10.0;
        let z = angle.sin() * 10.0;

        let name = format!("Sphere_circle{}", i);

        let sphere_entity = new_entity!(
            app,
            components::Name(Box::leak(name.into_boxed_str())),
            components::Model::Dynamic {
                obj_path: "res/models/sphere/sphere.obj",
            },
            components::Pos3::new(cgmath::Vector3::new(x, 0.0, z))
        );

        *sphere = sphere_entity;
    }

    // Physics Body 1
    let physics_body_1 = new_entity!(
        app,
        components::Name("Physics Body 2"),
        components::PhysicsBody {
            position: cgmath::Vector3::new(6.0, 0.0, -20.0),
            rotation: cgmath::Quaternion::one(),
            mass: 1.0,
            velocity: cgmath::Vector3::new(-10.0, 0.0, 0.0),
            acceleration: cgmath::Vector3::new(0.0, 0.0, 0.0),
            collision_box: components::CollisionBox {
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
        components::PhysicsBody {
            position: cgmath::Vector3::new(-5.0, 0.0, -20.0),
            rotation: cgmath::Quaternion::one(),
            mass: 1.0,
            velocity: cgmath::Vector3::new(10.0, 0.0, 0.0),
            acceleration: cgmath::Vector3::new(0.0, 0.0, 0.0),
            collision_box: components::CollisionBox {
                min: cgmath::Vector3::new(-1.0, -1.0, -1.0),
                max: cgmath::Vector3::new(1.0, 1.0, 1.0),
            },
        },
        components::Model::Dynamic {
            obj_path: "res/models/sphere/sphere.obj"
        }
    );

    // Update loop
    app.update_loop(move |ecs, dt| {
        // ! Here we are inside a loop, so this has to lock on all iterations.
        let ecs = ecs.lock().unwrap();
        let circle_speed = 8.0f32;
        let light_speed_multiplier = 3.0f32;

        // Move the spheres in a circle considering accumulated time
        for sphere in moving_spheres.iter() {
            if let Some(pos) = ecs.get_component_from_entity::<components::Pos3>(*sphere) {
                let mut pos3 = pos.write().unwrap();

                pos3.pos = cgmath::Quaternion::from_axis_angle(
                    (0.0, 1.0, 0.0).into(),
                    cgmath::Deg(PI * dt.as_secs_f32() * circle_speed),
                ) * pos3.pos;
            }
        }
        // Move the red and blue lights in a circle considering accumulated time
        if let Some(pos) = ecs.get_component_from_entity::<components::Pos3>(red_light) {
            let mut pos3 = pos.write().unwrap();

            pos3.pos = cgmath::Quaternion::from_axis_angle(
                (0.0, 1.0, 0.0).into(),
                cgmath::Deg(PI * dt.as_secs_f32() * circle_speed * light_speed_multiplier),
            ) * pos3.pos;
        }

        if let Some(pos) = ecs.get_component_from_entity::<components::Pos3>(blue_light) {
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
