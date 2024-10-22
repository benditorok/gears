use super::{Entity, Manager};

pub struct EcsBuilder<'a> {
    ecs: &'a mut Manager,
    entity: Entity,
}

impl<'a> EcsBuilder<'a> {
    pub fn new(ecs: &'a mut Manager) -> Self {
        let entity = ecs.create_entity();

        Self { ecs, entity }
    }
}

impl super::traits::EntityBuilder for EcsBuilder<'_> {
    fn new_entity(&mut self) -> &mut Self {
        self.entity = self.ecs.create_entity();

        self
    }

    fn add_component<T: 'static + Send + Sync>(&mut self, component: T) -> &mut Self {
        self.ecs.add_component_to_entity(self.entity, component);

        self
    }

    fn build(&mut self) -> Entity {
        self.entity
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::{self, traits::EntityBuilder};

    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestComponent {
        value: i32,
    }

    #[test]
    fn test_create_entity() {
        let mut manager = Manager::default();
        EcsBuilder::new(&mut manager).new_entity().build();
        assert!(manager.entity_count() == 1);
    }

    #[test]
    fn test_add_component() {
        let mut manager = Manager::default();
        let entity = EcsBuilder::new(&mut manager)
            .new_entity()
            .add_component(TestComponent { value: 42 })
            .build();
        let binding = manager
            .get_component_from_entity::<TestComponent>(entity)
            .unwrap();
        let component = binding.read().unwrap();
        assert_eq!(*component, TestComponent { value: 42 });
    }

    #[test]
    fn test_chain_add_components() {
        let mut manager = Manager::default();
        let entity = EcsBuilder::new(&mut manager)
            .new_entity()
            .add_component(TestComponent { value: 42 })
            .add_component(TestComponent { value: 100 })
            .build();

        let component = manager
            .get_component_from_entity::<TestComponent>(entity)
            .unwrap();
        let component = component.read().unwrap();
        assert_eq!(*component, TestComponent { value: 100 });
    }
}
