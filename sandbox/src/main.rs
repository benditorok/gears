use futures::executor::block_on;
use gears::{
    core::app::{self, App},
    ecs::{components::Position, utils::EntityBuilder, World},
    window,
};

pub struct Health(i32);

pub struct Name(&'static str);

fn main() {
    //run_sample_code();

    let mut world = World::new();

    EntityBuilder::new_entity(&mut world)
        .add_component(Name("Entity1"))
        .add_component(Health(100))
        .add_component(Position::new(0.0, 0.0, 0.0))
        .build();

    EntityBuilder::new_entity(&mut world)
        .add_component(Name("Entity2"))
        .add_component(Health(100))
        .add_component(Position::new(1.0, 1.0, 1.0))
        .build();

    EntityBuilder::new_entity(&mut world)
        .add_component(Name("Ent3"))
        .add_component(Position::new(12.0, 30.0, 120.0))
        .build();

    let mut app = app::GearsApp::default();
    app.map_world(world);

    block_on(app.run());
}
