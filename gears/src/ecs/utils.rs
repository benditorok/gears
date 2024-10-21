use super::{Entity, Manager};

pub struct EntityBuilder<'a> {
    ecs: &'a mut Manager,
    entity: Entity,
}

impl<'a> EntityBuilder<'a> {
    pub fn new_entity(ecs: &'a mut Manager) -> Self {
        let entity = ecs.create_entity();
        Self { ecs, entity }
    }

    pub fn add_component<T: 'static + Send + Sync>(self, component: T) -> Self {
        self.ecs.add_component_to_entity(self.entity, component);
        self
    }

    pub fn build(self) -> Entity {
        self.entity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestComponent {
        value: i32,
    }

    #[test]
    fn test_create_entity() {
        let mut manager = Manager::default();
        EntityBuilder::new_entity(&mut manager).build();
        assert!(manager.entity_count() == 1);
    }

    #[test]
    fn test_add_component() {
        let mut manager = Manager::default();
        let entity = EntityBuilder::new_entity(&mut manager)
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
        let builder = EntityBuilder::new_entity(&mut manager);
        let entity = builder
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
