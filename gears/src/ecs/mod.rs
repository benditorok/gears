pub mod components;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u32);

pub struct Manager {
    entities: RwLock<HashMap<Entity, HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
    next_entity: AtomicU32,
}

impl Manager {
    pub fn new() -> Self {
        Manager {
            entities: RwLock::new(HashMap::new()),
            next_entity: AtomicU32::new(0),
        }
    }

    pub fn create_entity(&self) -> Entity {
        let id = self.next_entity.fetch_add(1, Ordering::SeqCst);
        let entity = Entity(id);
        self.entities
            .write()
            .unwrap()
            .insert(entity, HashMap::new());
        entity
    }

    pub fn add_component<T: 'static + Send + Sync>(&self, entity: Entity, component: T) {
        if let Some(components) = self.entities.write().unwrap().get_mut(&entity) {
            components.insert(TypeId::of::<T>(), Arc::new(component));
        }
    }

    pub fn get_component<T: 'static + Send + Sync>(&self, entity: Entity) -> Option<Arc<T>> {
        self.entities
            .write()
            .unwrap()
            .get(&entity)
            .and_then(|components| {
                components
                    .get(&TypeId::of::<T>())
                    .and_then(|component| component.clone().downcast::<T>().ok())
            })
    }
    pub fn iter_entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities
            .read()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn get_all_components_of_type<T: 'static + Send + Sync>(&self) -> Vec<(Entity, Arc<T>)> {
        let mut result = Vec::new();
        for (entity, components) in self.entities.write().unwrap().iter() {
            if let Some(component) = components.get(&TypeId::of::<T>()) {
                if let Ok(component) = component.clone().downcast::<T>() {
                    result.push((*entity, component));
                }
            }
        }
        result
    }
}
