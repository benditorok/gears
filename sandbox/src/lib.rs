// use gears::ecs::{components::Position, utils::EntityBuilder, World};

// pub struct Health(i32);

// pub struct Name(&'static str);

// pub fn run_sample_code() {
//     // Create "ECS"
//     // This contains all the entities and their components
//     let mut world = World::new();

//     // Create entities
//     create_entities(&mut world);

//     let mut healths = world.borrow_component_vec_mut::<Health>().unwrap();
//     let mut names = world.borrow_component_vec_mut::<Name>().unwrap();
//     let mut positions = world.borrow_component_vec_mut::<Position>().unwrap();
//     let zip = healths.iter_mut().zip(names.iter_mut());
//     let iter = zip.filter_map(|(health, name)| Some((health.as_mut()?, name.as_mut()?)));

//     for (health, name) in iter {
//         println!("Iterating over {} with health {}", name.0, health.0);
//     }

//     for position in positions
//         .iter_mut()
//         .filter_map(|position| Some(position.as_mut()?))
//     {
//         println!("Reported position: {:?}", position);
//     }
// }

// fn create_entities(mut world: &mut World) {
//     // Create entity without a builder.
//     let entity_basic = world.new_entity();
//     world.add_component_to_entity(entity_basic, Name("Entity1"));
//     world.add_component_to_entity(entity_basic, Health(-10));

//     // Use EntityBuilder with existing entity.
//     let entity_existing_builder = world.new_entity();
//     EntityBuilder::entity(entity_existing_builder, world)
//         .add_component(Name("Entity2"))
//         .add_component(Health(100))
//         .add_component(Position::new(1.0, 1.0, 1.0))
//         .build();

//     EntityBuilder::new_entity(world)
//         .add_component(Name("Ent3"))
//         .add_component(Position::new(12.0, 30.0, 120.0))
//         .build();
// }
