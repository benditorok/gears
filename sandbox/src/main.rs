use ecs::utils::EntityBuilder;
use gears::prelude::*;
use log::{self, info};
use std::sync::Arc;
use std::{env, thread};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut ecs = ecs::Manager::default();

    // Add FPS camera
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("FPS Camera"))
        .add_component(components::Pos3::new(20.0, 10.0, 20.0))
        .add_component(components::Camera::FPS {
            look_at: components::Pos3::new(0.0, 0.0, 0.0),
            speed: 10.0,
            sensitivity: 0.5,
        })
        .build();

    // // Add fixed camera
    // EntityBuilder::new_entity(&mut ecs)
    //     .add_component(components::Name("Fixed Camera"))
    //     .add_component(components::Pos3::new(20.0, 15.0, 20.0))
    //     .add_component(components::Camera::Fixed {
    //         look_at: components::Pos3::new(0.0, 10.0, 0.0),
    //     })
    //     .build();

    // Add ambient light
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Ambient Light"))
        .add_component(components::Light::Ambient)
        .add_component(components::Pos3::new(0.0, 50.0, 0.0))
        .build();

    // * Add moving red light
    let blue_light = EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Red Light"))
        .add_component(components::Light::PointColoured {
            radius: 10.0,
            color: [0.8, 0.0, 0.0],
        })
        .add_component(components::Pos3::new(15.0, 5.0, 0.0))
        .build();

    // * Add moving blue light
    let red_light = EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Blue Light"))
        .add_component(components::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.0, 0.8],
        })
        .add_component(components::Pos3::new(-15.0, 5.0, 0.0))
        .build();

    // RGB lights
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("R"))
        .add_component(components::Light::PointColoured {
            radius: 10.0,
            color: [1.0, 0.0, 0.0],
        })
        .add_component(components::Pos3::new(0.0, 5.0, -20.0))
        .build();

    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("G"))
        .add_component(components::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 1.0, 0.0],
        })
        .add_component(components::Pos3::new(0.0, 5.0, -30.0))
        .build();

    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("B"))
        .add_component(components::Light::PointColoured {
            radius: 10.0,
            color: [0.0, 0.0, 1.0],
        })
        .add_component(components::Pos3::new(0.0, 5.0, -40.0))
        .build();

    // Plane
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Plane"))
        .add_component(components::Model::Dynamic {
            obj_path: "res/models/plane/plane.obj",
        })
        .add_component(components::Pos3::new(0.0, -3.0, 0.0))
        // .add_component(components::Scale::NonUniform {
        //     x: 2.0,
        //     y: 2.0,
        //     z: 1.0,
        // })
        // .add_component(components::Flip::Horizontal)
        .build();

    // Center sphere
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Sphere1"))
        .add_component(components::Model::Dynamic {
            obj_path: "res/models/sphere/sphere.obj",
        })
        .add_component(components::Pos3::new(0.0, 0.0, 0.0))
        .add_component(components::Flip::Vertical)
        // .add_component(components::Collider::new(
        //     cgmath::Point3::new(-5.0, -5.0, -5.0),
        //     cgmath::Point3::new(5.0, 5.0, 5.0),
        // ))
        .build();

    // Cube 1
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Cube1"))
        .add_component(components::Model::Dynamic {
            obj_path: "res/models/cube/cube.obj",
        })
        .add_component(components::Pos3::new(10.0, 0.0, 10.0))
        .build();

    // Cube 2
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Cube2"))
        .add_component(components::Model::Dynamic {
            obj_path: "res/models/cube/cube.obj",
        })
        .add_component(components::Pos3::new(10.0, 0.0, -10.0))
        .build();

    // Cube 3
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Cube3"))
        .add_component(components::Model::Dynamic {
            obj_path: "res/models/cube/cube.obj",
        })
        .add_component(components::Pos3::new(-10.0, 0.0, -10.0))
        .build();

    // Cube 4
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Cube4"))
        .add_component(components::Model::Dynamic {
            obj_path: "res/models/cube/cube.obj",
        })
        .add_component(components::Pos3::new(-10.0, 0.0, 10.0))
        .build();

    // Add 5 spheres in a circle
    let mut moving_spheres = vec![];
    for i in 0..5 {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 5.0;
        let x = angle.cos() * 10.0;
        let z = angle.sin() * 10.0;

        let name = format!("Sphere_circle{}", i);

        let sphere = EntityBuilder::new_entity(&mut ecs)
            .add_component(components::Name(Box::leak(name.into_boxed_str())))
            .add_component(components::Model::Dynamic {
                obj_path: "res/models/sphere/sphere.obj",
            })
            .add_component(components::Pos3::new(x, 0.0, z))
            .build();

        moving_spheres.push(sphere);
    }

    // Create the app
    let mut app = app::GearsApp::default();
    let ecs = app.map_ecs(ecs);

    // Update loop
    if let Some(mut rx_dt) = app.get_dt_channel() {
        let ecs_update = Arc::clone(&ecs);
        let start_time = std::time::Instant::now();

        app.thread_pool.execute(move |stop_flag| {
            let mut accumulated_time: f32 = 0.0;

            while !stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                let dt = futures::executor::block_on(rx_dt.recv());

                if let Ok(dt) = dt {
                    let ecs = ecs_update.lock().unwrap();

                    // Convert delta time to seconds as f32
                    let delta_seconds = dt.as_secs_f32();

                    // Accumulate the time
                    accumulated_time += delta_seconds;
                    //info!("Accumulated time: {}", accumulated_time);

                    // Move the spheres in a circle considering accumulated time
                    for (i, sphere) in moving_spheres.iter().enumerate() {
                        if let Some(pos) =
                            ecs.get_component_from_entity::<components::Pos3>(*sphere)
                        {
                            let mut pos = pos.write().unwrap();
                            let angle = accumulated_time
                                + i as f32 * std::f32::consts::PI / moving_spheres.len() as f32;
                            pos.x = angle.cos() * 10.0;
                            pos.z = angle.sin() * 10.0;
                        }
                    }

                    // Move the red and blue lights in a circle considering accumulated time
                    if let Some(pos) = ecs.get_component_from_entity::<components::Pos3>(red_light)
                    {
                        let mut pos = pos.write().unwrap();
                        let angle = accumulated_time;
                        pos.x = angle.cos() * 15.0;
                        pos.z = angle.sin() * 15.0;
                    }

                    if let Some(pos) = ecs.get_component_from_entity::<components::Pos3>(blue_light)
                    {
                        let mut pos = pos.write().unwrap();
                        let angle = accumulated_time;
                        pos.x = angle.cos() * -15.0;
                        pos.z = angle.sin() * -15.0;
                    }
                }
            }
        });
    }

    app.run().await
}
