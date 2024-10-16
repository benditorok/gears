use ecs::utils::EntityBuilder;
use ecs::Entity;
use gears::prelude::*;
use rand::Rng;
use std::sync::Arc;
use std::thread;

pub struct Health(i32);

pub struct Name(&'static str);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut ecs = ecs::Manager::default();

    // Add FPS camera
    EntityBuilder::new_entity(&mut ecs)
        .add_component(Name("FPS Camera"))
        .add_component(components::Pos3::new(5.0, 10.0, 0.0))
        .add_component(components::Camera::FPS {
            look_at: components::Pos3::new(0.0, 0.0, 0.0),
            speed: 10.0,
            sensitivity: 0.5,
        })
        .build();

    // // Add fixed camera
    // EntityBuilder::new_entity(&mut ecs)
    //     .add_component(Name("Fixed Camera"))
    //     .add_component(components::Pos3::new(20.0, 15.0, 20.0))
    //     .add_component(components::Camera::Fixed {
    //         look_at: components::Pos3::new(0.0, 10.0, 0.0),
    //     })
    //     .build();

    // Cube 1
    EntityBuilder::new_entity(&mut ecs)
        .add_component(Name("Cube1"))
        .add_component(components::ModelSource("res/models/cube/cube.obj"))
        .add_component(components::Pos3::new(10.0, 0.0, 10.0))
        .build();

    // Cube 2
    EntityBuilder::new_entity(&mut ecs)
        .add_component(Name("Cube2"))
        .add_component(components::ModelSource("res/models/cube/cube.obj"))
        .add_component(components::Pos3::new(10.0, 0.0, -10.0))
        .build();

    // Cube 3
    EntityBuilder::new_entity(&mut ecs)
        .add_component(Name("Cube3"))
        .add_component(components::ModelSource("res/models/cube/cube.obj"))
        .add_component(components::Pos3::new(-10.0, 0.0, -10.0))
        .build();

    // Cube 4
    EntityBuilder::new_entity(&mut ecs)
        .add_component(Name("Cube4"))
        .add_component(components::ModelSource("res/models/cube/cube.obj"))
        .add_component(components::Pos3::new(-10.0, 0.0, 10.0))
        .build();

    // Center sphere
    EntityBuilder::new_entity(&mut ecs)
        .add_component(Name("Sphere1"))
        .add_component(components::ModelSource("res/models/sphere/sphere.obj"))
        .add_component(components::Pos3::new(0.0, 0.0, 0.0))
        .build();

    // Add random spheres
    for i in 0..=20 {
        let name = format!("Sphere_rand{}", i);

        EntityBuilder::new_entity(&mut ecs)
            .add_component(Name(Box::leak(name.into_boxed_str())))
            .add_component(components::ModelSource("res/models/sphere/sphere.obj"))
            .add_component(components::Pos3::new(
                rand::random::<f32>() * 40.0 - 20.0,
                rand::random::<f32>() * 40.0 - 20.0,
                rand::random::<f32>() * 40.0 - 20.0,
            ))
            .build();
    }

    // Create the app
    let mut app = app::GearsApp::default();
    let ecs = app.map_ecs(ecs);

    let ecs_sanbox_t2_access = Arc::clone(&ecs);

    app.thread_pool.execute(move |stop_flag| {
        let mut rng = rand::thread_rng();
        while !stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
            {
                let ecs = ecs_sanbox_t2_access.lock().unwrap();
                for entity in ecs.iter_entities() {
                    if let Some(name) = ecs.get_component_from_entity::<Name>(entity) {
                        if name.read().unwrap().0.contains("Sphere_rand") {
                            if let Some(pos) =
                                ecs.get_component_from_entity::<components::Pos3>(entity)
                            {
                                let mut pos = pos.write().unwrap();
                                pos.x = rng.gen::<f32>() * 40.0 - 20.0;
                                pos.y = rng.gen::<f32>() * 40.0 - 20.0;
                                pos.z = rng.gen::<f32>() * 40.0 - 20.0;
                            }
                        }
                    }
                }
            }

            thread::sleep(std::time::Duration::from_millis(250));
        }
    });

    app.run().await
}
