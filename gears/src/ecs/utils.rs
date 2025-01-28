use log::warn;

use super::{Component, Entity, EntityBuilder, World};

pub struct EcsBuilder<'a> {
    ecs: &'a mut World,
}

impl<'a> EcsBuilder<'a> {
    pub fn new(ecs: &'a mut World) -> Self {
        Self { ecs }
    }
}

impl EntityBuilder for EcsBuilder<'_> {
    fn new_entity(&mut self) -> &mut Self {
        self.ecs.create_entity();

        self
    }

    fn add_component(&mut self, component: impl Component) -> &mut Self {
        if let Some(entity) = self.ecs.get_last() {
            self.ecs.add_component(entity, component);
        } else {
            warn!("No entity found, creating a new one...");

            let entity = self.ecs.create_entity();
            self.ecs.add_component(entity, component);
        }

        self
    }

    fn build(&mut self) -> Entity {
        if let Some(entity) = self.ecs.get_last() {
            entity
        } else {
            warn!("No entity found, creating a new one...");

            self.ecs.create_entity()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestComponent {
        value: i32,
    }

    impl Component for TestComponent {}

    #[test]
    fn test_create_entity_with_component() {
        let mut manager = World::default();
        let entity = EcsBuilder::new(&mut manager)
            .new_entity()
            .add_component(TestComponent { value: 42 })
            .build();

        assert_eq!(Entity(0), entity);
        assert_eq!(manager.storage_len(), 1);
    }

    #[test]
    fn test_add_component() {
        let mut manager = World::default();
        let entity = EcsBuilder::new(&mut manager)
            .new_entity()
            .add_component(TestComponent { value: 42 })
            .build();
        let binding = manager.get_component::<TestComponent>(entity).unwrap();
        let component = binding.read().unwrap();
        assert_eq!(*component, TestComponent { value: 42 });
    }

    #[test]
    fn test_chain_add_components() {
        let mut manager = World::default();
        let entity = EcsBuilder::new(&mut manager)
            .new_entity()
            .add_component(TestComponent { value: 42 })
            .add_component(TestComponent { value: 100 })
            .build();

        let component = manager.get_component::<TestComponent>(entity).unwrap();
        let component = component.read().unwrap();
        assert_eq!(*component, TestComponent { value: 100 });
    }
}
