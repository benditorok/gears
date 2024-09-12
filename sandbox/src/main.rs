use futures::executor::block_on;
use gears::prelude::*;
use rand::Rng;
use std::sync::Arc;
use std::thread;

pub struct Health(i32);

pub struct Name(&'static str);

fn main() {
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

    let ecs = ecs::Manager::new();

    let entity = ecs.create_entity();
    ecs.add_component_to_entity(entity, "Hello, ECS!".to_string());
    ecs.add_component_to_entity(
        entity,
        components::GearsModelData::new("res/models/cube/cube.obj"),
    );
    ecs.add_component_to_entity(entity, components::Position::new(1.0, 1.0, 1.0));

    if let Some(component) = ecs.get_component_from_entity::<String>(entity) {
        println!("Entity {:?} has component: {}", entity, component);
    }

    let strigns = ecs.get_all_components_of_type::<String>();
    for (entity, component) in strigns {
        println!("Entity {:?} has a String component: {}", entity, component);
    }

    // let entity = ecs.create_entity();
    // ecs.add_component_to_entity(entity, "Hello, ECS!".to_string());
    // ecs.add_component_to_entity(entity, GearsModelData::new("res/models/cube/cube.obj"));
    // ecs.add_component_to_entity(entity, Position::new(2.0, 2.0, 2.0));

    let entity = ecs.create_entity();
    ecs.add_component_to_entity(entity, "Hello, ECS!".to_string());
    ecs.add_component_to_entity(
        entity,
        components::GearsModelData::new("res/models/cube/cube.obj"),
    );
    ecs.add_component_to_entity(entity, components::Position::new(10.0, -10.0, 10.0));

    let entity = ecs.create_entity();
    ecs.add_component_to_entity(entity, "Hello, ECS!".to_string());
    ecs.add_component_to_entity(
        entity,
        components::GearsModelData::new("res/models/cube/cube.obj"),
    );
    ecs.add_component_to_entity(entity, components::Position::new(5.0, -5.0, 5.0));

    let entity = ecs.create_entity();
    ecs.add_component_to_entity(entity, "SPHERE".to_string());
    ecs.add_component_to_entity(
        entity,
        components::GearsModelData::new("res/models/sphere/v2/sphere.obj"),
    );
    ecs.add_component_to_entity(entity, components::Position::new(0.0, 0.0, 0.0));

    for i in 0..=20 {
        let entity = ecs.create_entity();
        ecs.add_component_to_entity(entity, "SPHERE".to_string());
        ecs.add_component_to_entity(
            entity,
            components::GearsModelData::new("res/models/sphere/v2/sphere.obj"),
        );
        // add a randdom position to them in the range of -20 to 20
        ecs.add_component_to_entity(
            entity,
            components::Position::new(
                rand::random::<f32>() * 40.0 - 20.0,
                rand::random::<f32>() * 40.0 - 20.0,
                rand::random::<f32>() * 40.0 - 20.0,
            ),
        );
    }

    let mut app = app::GearsApp::default();
    let ecs = app.map_ecs(ecs);

    let ecs_clone = Arc::clone(&ecs);

    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        loop {
            {
                let ecs = ecs_clone.lock().unwrap();

                let entity = ecs.create_entity();
                ecs.add_component_to_entity(entity, "SPHERE".to_string());
                ecs.add_component_to_entity(
                    entity,
                    components::GearsModelData::new("res/models/sphere/v2/sphere.obj"),
                );
                // add a randdom position to them in the range of -20 to 20
                ecs.add_component_to_entity(
                    entity,
                    components::Position::new(
                        rng.gen::<f32>() * 40.0 - 20.0,
                        rng.gen::<f32>() * 40.0 - 20.0,
                        rng.gen::<f32>() * 40.0 - 20.0,
                    ),
                );
            }

            thread::sleep(std::time::Duration::from_secs(1));
        }
    });

    block_on(app.run());
}
