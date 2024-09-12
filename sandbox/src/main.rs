use gears::prelude::*;
use rand::Rng;
use std::sync::Arc;
use std::{any, thread};

pub struct Health(i32);

pub struct Name(&'static str);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ecs = ecs::Manager::new();

    // Cube 1
    let entity = ecs.create_entity();
    ecs.add_component_to_entity(entity, Name("Cube1"));
    ecs.add_component_to_entity(
        entity,
        components::GearsModelData::new("res/models/cube/cube.obj"),
    );
    ecs.add_component_to_entity(entity, components::Pos3::new(10.0, 0.0, 10.0));

    // Cube 2
    let entity = ecs.create_entity();
    ecs.add_component_to_entity(entity, Name("Cube2"));
    ecs.add_component_to_entity(
        entity,
        components::GearsModelData::new("res/models/cube/cube.obj"),
    );
    ecs.add_component_to_entity(entity, components::Pos3::new(10.0, 0.0, -10.0));

    // Cube 3
    let entity = ecs.create_entity();
    ecs.add_component_to_entity(entity, Name("Cube3"));
    ecs.add_component_to_entity(
        entity,
        components::GearsModelData::new("res/models/cube/cube.obj"),
    );
    ecs.add_component_to_entity(entity, components::Pos3::new(-10.0, 0.0, -10.0));

    // Cube 4
    let entity = ecs.create_entity();
    ecs.add_component_to_entity(entity, Name("Cube4"));
    ecs.add_component_to_entity(
        entity,
        components::GearsModelData::new("res/models/cube/cube.obj"),
    );
    ecs.add_component_to_entity(entity, components::Pos3::new(-10.0, 0.0, 10.0));

    // Center sphere
    let entity = ecs.create_entity();
    ecs.add_component_to_entity(entity, Name("Sphere1"));
    ecs.add_component_to_entity(
        entity,
        components::GearsModelData::new("res/models/sphere/sphere.obj"),
    );
    ecs.add_component_to_entity(entity, components::Pos3::new(00.0, 0.0, 0.0));

    // Add random spheres
    for i in 0..=20 {
        let entity = ecs.create_entity();
        ecs.add_component_to_entity(entity, "SPHERE".to_string());
        ecs.add_component_to_entity(
            entity,
            components::GearsModelData::new("res/models/sphere/sphere.obj"),
        );
        // add a randdom position to them in the range of -20 to 20
        ecs.add_component_to_entity(
            entity,
            components::Pos3::new(
                rand::random::<f32>() * 40.0 - 20.0,
                rand::random::<f32>() * 40.0 - 20.0,
                rand::random::<f32>() * 40.0 - 20.0,
            ),
        );
    }

    let mut app = app::GearsApp::default();
    let ecs = app.map_ecs(ecs);

    let ecs_clone = Arc::clone(&ecs);
    app.thread_pool.execute(move || {
        let mut rng = rand::thread_rng();
        loop {
            {
                let ecs = ecs_clone.lock().unwrap();
                for entity in ecs.iter_entities() {
                    if let Some(pos) = ecs.get_component_from_entity::<components::Pos3>(entity) {
                        let mut pos = pos.write().unwrap();
                        *pos.x = rand::random::<f32>() * 40.0 - 20.0;
                        *pos.y = rand::random::<f32>() * 40.0 - 20.0;
                        *pos.z = rand::random::<f32>() * 40.0 - 20.0;
                    }
                }
            }

            thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    app.run().await
}

// let ecs_clone = Arc::clone(&ecs);

// thread::spawn(move || {
//     let mut rng = rand::thread_rng();
//     loop {
//         {
//             let ecs = ecs_clone.lock().unwrap();

//             let entity = ecs.create_entity();
//             ecs.add_component_to_entity(entity, "SPHERE".to_string());
//             ecs.add_component_to_entity(
//                 entity,
//                 components::GearsModelData::new("res/models/sphere/v2/sphere.obj"),
//             );
//             // add a randdom position to them in the range of -20 to 20
//             ecs.add_component_to_entity(
//                 entity,
//                 components::Position::new(
//                     rng.gen::<f32>() * 40.0 - 20.0,
//                     rng.gen::<f32>() * 40.0 - 20.0,
//                     rng.gen::<f32>() * 40.0 - 20.0,
//                 ),
//             );
//         }

//         thread::sleep(std::time::Duration::from_secs(1));
//     }
// });

// //run_sample_code();

// let mut world = GearsWorld::new();

// EntityBuilder::new_entity(&mut world)
//     .add_component(Name("Entity1"))
//     .add_component(Health(100))
//     .add_component(Position::new(0.0, 0.0, 0.0))
//     .build();

// EntityBuilder::new_entity(&mut world)
//     .add_component(Name("Entity2"))
//     .add_component(Health(100))
//     .add_component(Position::new(1.0, 1.0, 1.0))
//     .build();

// EntityBuilder::new_entity(&mut world)
//     .add_component(Name("Ent3"))
//     .add_component(Position::new(12.0, 30.0, 120.0))
//     .add_component(GearsModelData::new("res/models/cube/cube.obj"))
//     .build();
