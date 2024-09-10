use super::{GearsWorld, World};

/// Builder for creating entities.
pub struct EntityBuilder<'a> {
    entity: usize,
    world: &'a mut GearsWorld,
}

impl<'a> EntityBuilder<'a> {
    /// Create a new entity in the world.
    pub fn new_entity(world: &'a mut GearsWorld) -> Self {
        Self {
            entity: world.new_entity(),
            world,
        }
    }

    /// Select an existing entity from the world.
    pub fn entity(entity: usize, world: &'a mut GearsWorld) -> Self {
        Self { entity, world }
    }

    /// Add a component to the entity.
    pub fn add_component<ComponentType: 'static>(self, component: ComponentType) -> Self {
        self.world.add_component_to_entity(self.entity, component);
        self
    }

    /// Build the entity.
    pub fn build(self) {}
}
