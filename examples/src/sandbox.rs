use app::GearsApp;
use cgmath::Rotation3;
use ecs::traits::EntityBuilder;
use gears::prelude::*;
use std::f32::consts::PI;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = GearsApp::default();

    // Add FPS camera
    app.new_entity()
        .add_component(components::Name("FPS Camera"))
        .add_component(components::Pos3::new(cgmath::Vector3::new(
            20.0, 10.0, 20.0,
        )))
        .add_component(components::Camera::FPS {
            look_at: cgmath::Point3::new(0.0, 0.0, 0.0),
            speed: 10.0,
            sensitivity: 0.5,
        })
        .build();

    // // Add fixed camera
    // app.new_entity()
    //     .add_component(components::Name("Fixed Camera"))
    //     .add_component(components::Pos3::new(20.0, 15.0, 20.0))
    //     .add_component(components::Camera::Fixed {
    //         look_at: components::Pos3::new(0.0, 10.0, 0.0),
    //     })
    //     .build();

    app.new_entity() // Add ambient light
        .add_component(components::Name("Ambient Light"))
        .add_component(components::Light::Ambient { intensity: 0.05 })
        .add_component(components::Pos3::new(cgmath::Vector3::new(0.0, 50.0, 0.0)))
        .new_entity() // Add directional light
        .add_component(components::Name("Directional Light"))
        .add_component(components::Light::Directional {
            direction: [-0.5, -0.5, 0.0],
            intensity: 0.3,
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(
            30.0, 30.0, 30.0,
        )))
        .build();

    // * Add moving red light
    let blue_light = app
        .new_entity()
        .add_component(components::Name("Red Light"))
        .add_component(components::Light::PointColoured {
            radius: 10.0,
            color: [0.8, 0.0, 0.0],
            intensity: 1.0,
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(15.0, 5.0, 0.0)))
        .build();

    // * Add moving blue light
    let red_light = app
        .new_entity()
        .add_component(components::Name("Blue Light"))
        .add_component(components::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.0, 0.8],
            intensity: 1.0,
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(-15.0, 5.0, 0.0)))
        .build();

    // RGB lights
    app.new_entity()
        .add_component(components::Name("R"))
        .add_component(components::Light::PointColoured {
            radius: 10.0,
            color: [1.0, 0.0, 0.0],
            intensity: 1.0,
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(0.0, 5.0, -20.0)))
        .build();

    app.new_entity()
        .add_component(components::Name("G"))
        .add_component(components::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 1.0, 0.0],
            intensity: 1.0,
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(0.0, 5.0, -30.0)))
        .build();

    app.new_entity()
        .add_component(components::Name("B"))
        .add_component(components::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.0, 1.0],
            intensity: 1.0,
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(0.0, 5.0, -40.0)))
        .build();

    // Plane
    app.new_entity()
        .add_component(components::Name("Plane"))
        .add_component(components::Model::Dynamic {
            obj_path: "res/models/plane/plane.obj",
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(0.0, -3.0, 0.0)))
        // .add_component(components::Scale::NonUniform {
        //     x: 2.0,
        //     y: 2.0,
        //     z: 1.0,
        // })
        // .add_component(components::Flip::Horizontal)
        .build();

    // Center sphere
    app.new_entity()
        .add_component(components::Name("Sphere1"))
        .add_component(components::Model::Dynamic {
            obj_path: "res/models/sphere/sphere.obj",
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(0.0, 0.0, 0.0)))
        .add_component(components::Flip::Vertical)
        // .add_component(components::Collider::new(
        //     cgmath::Point3::new(-5.0, -5.0, -5.0),
        //     cgmath::Point3::new(5.0, 5.0, 5.0),
        // ))
        .build();

    // * If you do not need the IDs of the entities you can chain them together
    app.new_entity() // Cube 1
        .add_component(components::Name("Cube1"))
        .add_component(components::Model::Dynamic {
            obj_path: "res/models/cube/cube.obj",
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(10.0, 0.0, 10.0)))
        .new_entity() // Cube 2
        .add_component(components::Name("Cube2"))
        .add_component(components::Model::Dynamic {
            obj_path: "res/models/cube/cube.obj",
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(
            10.0, 0.0, -10.0,
        )))
        .new_entity() // Cube 3
        .add_component(components::Name("Cube3"))
        .add_component(components::Model::Dynamic {
            obj_path: "res/models/cube/cube.obj",
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(
            -10.0, 0.0, -10.0,
        )))
        .new_entity() // Cube 4
        .add_component(components::Name("Cube4"))
        .add_component(components::Model::Dynamic {
            obj_path: "res/models/cube/cube.obj",
        })
        .add_component(components::Pos3::new(cgmath::Vector3::new(
            -10.0, 0.0, 10.0,
        )))
        .build();

    // Add 5 spheres in a circle
    let mut moving_spheres: [ecs::Entity; 5] = [ecs::Entity { 0: 0 }; 5];
    for i in 0..5 {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 5.0;
        let x = angle.cos() * 10.0;
        let z = angle.sin() * 10.0;

        let name = format!("Sphere_circle{}", i);

        let sphere = app
            .new_entity()
            .add_component(components::Name(Box::leak(name.into_boxed_str())))
            .add_component(components::Model::Dynamic {
                obj_path: "res/models/sphere/sphere.obj",
            })
            .add_component(components::Pos3::new(cgmath::Vector3::new(x, 0.0, z)))
            .build();

        moving_spheres[i] = sphere;
    }

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
