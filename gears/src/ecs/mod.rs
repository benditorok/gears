pub mod components;
pub mod utils;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u32);

type EntityStore = HashMap<Entity, HashMap<TypeId, Arc<RwLock<dyn Any + Send + Sync>>>>;

// TODO add a world with scenes and scene switching

/// Entity component system manager.
pub struct Manager {
    entities: RwLock<EntityStore>,
    next_entity: AtomicU32,
}

impl Default for Manager {
    fn default() -> Self {
        Manager {
            entities: RwLock::new(HashMap::new()),
            next_entity: AtomicU32::new(0),
        }
    }
}

impl Manager {
    /// Create a new EntityManager with a specific capacity preallocated.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The preallocated capacity of the EntityManager.
    pub fn new(capacity: usize) -> Self {
        Manager {
            entities: RwLock::new(HashMap::with_capacity(capacity)),
            next_entity: AtomicU32::new(0),
        }
    }

    /// Create a new entity and return it.
    pub fn create_entity(&self) -> Entity {
        let id = self.next_entity.fetch_add(1, Ordering::SeqCst);
        let entity = Entity(id);
        self.entities
            .write()
            .unwrap()
            .insert(entity, HashMap::new());
        entity
    }

    /// Get the number of entities currently in the EntityManager.
    pub fn entity_count(&self) -> usize {
        self.entities.read().unwrap().len()
    }

    /// Add a component of a specific type to a specific entity.
    pub fn add_component_to_entity<T: 'static + Send + Sync>(&self, entity: Entity, component: T) {
        let mut entities = self.entities.write().unwrap();
        if let Some(components) = entities.get_mut(&entity) {
            components.insert(TypeId::of::<T>(), Arc::new(RwLock::new(component)));
        }
    }

    /// Get a component of a specific type for a specific entity.
    pub fn get_component_from_entity<T: 'static + Send + Sync>(
        &self,
        entity: Entity,
    ) -> Option<Arc<RwLock<T>>> {
        let entities = self.entities.read().unwrap();
        entities.get(&entity).and_then(|components| {
            components.get(&TypeId::of::<T>()).map(|component| {
                let component = Arc::clone(component);
                unsafe {
                    // SAFETY: We ensure that the component is of type T
                    let component_ptr = Arc::into_raw(component) as *const RwLock<T>;
                    Arc::from_raw(component_ptr)
                }
            })
        })
    }

    /// Get an iterator over the entities currently in the EntityManager.
    pub fn iter_entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities
            .read()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Get all components of a specific type currently in the EntityManager.
    pub fn get_all_components_of_type<T: 'static + Send + Sync>(
        &self,
    ) -> Vec<(Entity, Arc<RwLock<T>>)> {
        let mut result: Vec<(Entity, Arc<RwLock<T>>)> = Vec::new();
        let entities = self.entities.read().unwrap();
        for (entity, components) in entities.iter() {
            if let Some(component) = components.get(&TypeId::of::<T>()) {
                let component = component.clone();
                unsafe {
                    // SAFETY: We ensure that the component is of type T
                    let component_ptr = Arc::into_raw(component) as *const RwLock<T>;
                    result.push((*entity, Arc::from_raw(component_ptr)));
                }
            }
        }

        result
    }

    /// Get all entities that have a specific component.
    pub fn get_entites_with_component<T: 'static + Send + Sync>(&self) -> Vec<Entity> {
        let mut result: Vec<Entity> = Vec::new();
        let entities = self.entities.read().unwrap();
        for (entity, components) in entities.iter() {
            if components.contains_key(&TypeId::of::<T>()) {
                result.push(*entity);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestComponent(i32);

    #[test]
    fn test_create_entity() {
        let manager = Manager::default();
        let entity = manager.create_entity();
        assert_eq!(entity, Entity(0));
        let entity2 = manager.create_entity();
        assert_eq!(entity2, Entity(1));
    }

    #[test]
    fn test_add_and_get_component() {
        let manager = Manager::default();
        let entity = manager.create_entity();
        let component = TestComponent(42);
        manager.add_component_to_entity(entity, component);

        let retrieved_component = manager
            .get_component_from_entity::<TestComponent>(entity)
            .unwrap();
        assert_eq!(*retrieved_component.read().unwrap(), TestComponent(42));
    }

    #[test]
    fn test_get_nonexistent_component() {
        let manager = Manager::default();
        let entity = manager.create_entity();
        let retrieved_component = manager.get_component_from_entity::<TestComponent>(entity);
        assert!(retrieved_component.is_none());
    }

    #[test]
    fn test_iter_entities() {
        let manager = Manager::default();
        let entity1 = manager.create_entity();
        let entity2 = manager.create_entity();
        let entities: Vec<Entity> = manager.iter_entities().collect();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&entity1));
        assert!(entities.contains(&entity2));
    }

    #[test]
    fn test_get_all_components_of_type() {
        let manager = Manager::default();
        let entity1 = manager.create_entity();
        manager.add_component_to_entity(entity1, TestComponent(10));
        let entity2 = manager.create_entity();
        manager.add_component_to_entity(entity2, TestComponent(20));

        let components = manager.get_all_components_of_type::<TestComponent>();
        assert_eq!(components.len(), 2);
        assert!(components
            .iter()
            .any(|(e, c)| *e == entity1 && *c.read().unwrap() == TestComponent(10)));
        assert!(components
            .iter()
            .any(|(e, c)| *e == entity2 && *c.read().unwrap() == TestComponent(20)));
    }

    #[test]
    fn test_add_multiple_components_to_entity() {
        let manager = Manager::default();
        let entity = manager.create_entity();
        manager.add_component_to_entity(entity, TestComponent(42));
        manager.add_component_to_entity(entity, TestComponent(84));

        let components = manager.get_all_components_of_type::<TestComponent>();
        assert_eq!(components.len(), 1);
        assert_eq!(*components[0].1.read().unwrap(), TestComponent(84));
    }

    #[test]
    fn test_get_all_components_of_type_with_no_components() {
        let manager = Manager::default();
        let _ = manager.create_entity();
        let components = manager.get_all_components_of_type::<TestComponent>();
        assert!(components.is_empty());
    }

    #[test]
    fn test_get_entities_with_component() {
        let manager = Manager::default();
        let entity1 = manager.create_entity();
        manager.add_component_to_entity(entity1, TestComponent(10));
        let entity2 = manager.create_entity();
        manager.add_component_to_entity(entity2, TestComponent(20));
        let entity3 = manager.create_entity();

        let entities_with_component = manager.get_entites_with_component::<TestComponent>();
        assert_eq!(entities_with_component.len(), 2);
        assert!(entities_with_component.contains(&entity1));
        assert!(entities_with_component.contains(&entity2));
        assert!(!entities_with_component.contains(&entity3));
    }

    #[test]
    fn test_get_entities_with_component_no_entities() {
        let manager = Manager::default();
        let entities_with_component = manager.get_entites_with_component::<TestComponent>();
        assert!(entities_with_component.is_empty());
    }

    #[test]
    fn test_get_entities_with_component_no_matching_component() {
        let manager = Manager::default();
        let _entity1 = manager.create_entity();
        let _entity2 = manager.create_entity();

        let entities_with_component = manager.get_entites_with_component::<TestComponent>();
        assert!(entities_with_component.is_empty());
    }
}
