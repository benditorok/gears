use components::ModelSource;
use ecs::utils::EntityBuilder;
use ecs::Entity;
use gears::prelude::*;
use rand::Rng;
use std::sync::Arc;
use std::thread;

pub struct Health(i32);

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
        .add_component(components::Pos3::new(0.0, 5.0, 0.0))
        .build();

    // * Add moving red light
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Red Light"))
        .add_component(components::Light::PointColoured {
            radius: 15.0,
            color: [0.8, 0.0, 0.0],
        })
        .add_component(components::Pos3::new(15.0, 5.0, 0.0))
        .build();

    // * Add moving blue light
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Blue Light"))
        .add_component(components::Light::PointColoured {
            radius: 15.0,
            color: [0.0, 0.0, 0.8],
        })
        .add_component(components::Pos3::new(-15.0, 5.0, 0.0))
        .build();

    // Road segment
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Road"))
        .add_component(components::Model)
        .add_component(components::ModelSource("res/models/road/road.obj"))
        .add_component(components::Pos3::new(0.0, 0.0, 0.0))
        .add_component(components::Scale::NonUniform {
            x: 2.0,
            y: 2.0,
            z: 1.0,
        })
        .add_component(components::Flip::Horizontal)
        .build();

    // Cube 1
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Cube1"))
        .add_component(components::Model)
        .add_component(components::ModelSource("res/models/cube/cube.obj"))
        .add_component(components::Pos3::new(10.0, 0.0, 10.0))
        .build();

    // Cube 2
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Cube2"))
        .add_component(components::Model)
        .add_component(components::ModelSource("res/models/cube/cube.obj"))
        .add_component(components::Pos3::new(10.0, 0.0, -10.0))
        .build();

    // Cube 3
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Cube3"))
        .add_component(components::Model)
        .add_component(components::ModelSource("res/models/cube/cube.obj"))
        .add_component(components::Pos3::new(-10.0, 0.0, -10.0))
        .build();

    // Cube 4
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Cube4"))
        .add_component(components::Model)
        .add_component(components::ModelSource("res/models/cube/cube.obj"))
        .add_component(components::Pos3::new(-10.0, 0.0, 10.0))
        .build();

    // Center sphere
    EntityBuilder::new_entity(&mut ecs)
        .add_component(components::Name("Sphere1"))
        .add_component(components::Model)
        .add_component(components::ModelSource("res/models/sphere/sphere.obj"))
        .add_component(components::Pos3::new(0.0, 0.0, 0.0))
        .add_component(components::Flip::Vertical)
        .build();

    // // Add random spheres
    // for i in 0..=20 {
    //     let name = format!("Sphere_rand{}", i);

    //     EntityBuilder::new_entity(&mut ecs)
    //         .add_component(Name(Box::leak(name.into_boxed_str())))
    //         .add_component(components::ModelSource("res/models/sphere/sphere.obj"))
    //         .add_component(components::Pos3::new(
    //             rand::random::<f32>() * 40.0 - 20.0,
    //             rand::random::<f32>() * 40.0 - 20.0,
    //             rand::random::<f32>() * 40.0 - 20.0,
    //         ))
    //         .build();
    // }

    // Add 5 spheres in a circle
    for i in 0..5 {
        let angle = i as f32 * std::f32::consts::PI * 2.0 / 5.0;
        let x = angle.cos() * 10.0;
        let z = angle.sin() * 10.0;

        let name = format!("Sphere_circle{}", i);

        EntityBuilder::new_entity(&mut ecs)
            .add_component(components::Name(Box::leak(name.into_boxed_str())))
            .add_component(components::Model)
            .add_component(components::ModelSource("res/models/sphere/sphere.obj"))
            .add_component(components::Pos3::new(x, 0.0, z))
            .build();
    }

    // Create the app
    let mut app = app::GearsApp::default();
    let ecs = app.map_ecs(ecs);

    // TODO leak the last frame time trough channesl try_recv and update a components pos from outside with * dt

    // // Randomly move spheres
    // let ecs_sanbox_t2_access = Arc::clone(&ecs);
    // app.thread_pool.execute(move |stop_flag| {
    //     let mut rng = rand::thread_rng();
    //     while !stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
    //         {
    //             let ecs = ecs_sanbox_t2_access.lock().unwrap();
    //             for entity in ecs.iter_entities() {
    //                 if let Some(name) = ecs.get_component_from_entity::<Name>(entity) {
    //                     if name.read().unwrap().0.contains("Sphere_rand") {
    //                         if let Some(pos) =
    //                             ecs.get_component_from_entity::<components::Pos3>(entity)
    //                         {
    //                             let mut pos = pos.write().unwrap();
    //                             pos.x = rng.gen::<f32>() * 40.0 - 20.0;
    //                             pos.y = rng.gen::<f32>() * 40.0 - 20.0;
    //                             pos.z = rng.gen::<f32>() * 40.0 - 20.0;
    //                         }
    //                     }
    //                 }
    //             }
    //         }

    //         thread::sleep(std::time::Duration::from_millis(250));
    //     }
    // });

    // TODO if no workers are available then add more workers threads to the vec
    // * app thread pool should be reserved. create a way to move closures into the update call of the renderer and provide the deltat time for moving the objects correctly
    let ecs_sanbox_t2_access = Arc::clone(&ecs);
    app.thread_pool.execute(move |stop_flag| {
        let start_time = std::time::Instant::now();
        while !stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
            {
                let ecs = ecs_sanbox_t2_access.lock().unwrap();
                let elapsed = start_time.elapsed().as_secs_f32();
                for entity in ecs.iter_entities() {
                    if let Some(name) = ecs.get_component_from_entity::<components::Name>(entity) {
                        if name.read().unwrap().0.contains("Sphere_circle") {
                            if let Some(pos) =
                                ecs.get_component_from_entity::<components::Pos3>(entity)
                            {
                                let mut pos = pos.write().unwrap();
                                let angle =
                                    elapsed + (entity.0 as f32 * std::f32::consts::PI * 2.0 / 5.0);
                                pos.x = angle.cos() * 10.0;
                                pos.z = angle.sin() * 10.0;
                            }
                        }
                    }
                }
            }

            thread::sleep(std::time::Duration::from_millis(16));
        }
    });

    let ecs_t3_access = Arc::clone(&ecs);
    // Move the red sphere around in a circle
    app.thread_pool.execute(move |stop_flag| {
        let start_time = std::time::Instant::now();
        while !stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
            {
                let ecs = ecs_t3_access.lock().unwrap();
                let elapsed = start_time.elapsed().as_secs_f32();
                for entity in ecs.iter_entities() {
                    if let Some(name) = ecs.get_component_from_entity::<components::Name>(entity) {
                        let name = name.read().unwrap();
                        if name.0 == "Red Light" || name.0 == "Blue Light" {
                            if let Some(pos) =
                                ecs.get_component_from_entity::<components::Pos3>(entity)
                            {
                                let mut pos = pos.write().unwrap();
                                let angle = elapsed
                                    + if name.0 == "Red Light" {
                                        0.0
                                    } else {
                                        std::f32::consts::PI
                                    };
                                pos.x = angle.cos() * 10.0;
                                pos.z = angle.sin() * 10.0;
                            }
                        }
                    }
                }
            }

            thread::sleep(std::time::Duration::from_millis(16));
        }
    });

    app.run().await
}
