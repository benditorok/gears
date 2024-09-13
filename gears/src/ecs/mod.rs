pub mod components;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u32);

/// Entity component system manager.
pub struct Manager {
    entities: RwLock<HashMap<Entity, HashMap<TypeId, Arc<RwLock<dyn Any + Send + Sync>>>>>,
    next_entity: AtomicU32,
}

impl Manager {
    #[allow(clippy::new_without_default)]
    /// Create a new EntityManager.
    pub fn new() -> Self {
        Manager {
            entities: RwLock::new(HashMap::new()),
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
            components.get(&TypeId::of::<T>()).and_then(|component| {
                let component = component.clone();
                unsafe {
                    // SAFETY: We ensure that the component is of type T
                    let component_ptr = Arc::into_raw(component) as *const RwLock<T>;
                    Some(Arc::from_raw(component_ptr))
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
}
