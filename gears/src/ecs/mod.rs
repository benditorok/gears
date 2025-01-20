pub mod components;
pub mod traits;
pub mod utils;

use dashmap::{mapref::one::RefMut, DashMap};
use gltf::accessor::Item;
use std::any::{Any, TypeId};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use traits::Component;

pub type Entity = u32;

/// The ComponentStorage struct is responsible for storing components of a specific type.
/// It uses a DashMap to store the components, which allows for concurrent reads and writes.
/// The components are stored in an Arc<RwLock<T>> to allow for multiple reads or a single write.
#[derive(Debug)]
struct ComponentStorage<T> {
    storage: DashMap<Entity, Arc<RwLock<T>>>,
}

impl<T> Default for ComponentStorage<T> {
    /// Create a new ComponentStorage instance with a default capacity of 11.
    ///
    /// # Returns
    ///
    /// A new ComponentStorage instance.
    fn default() -> Self {
        Self {
            storage: DashMap::with_capacity(11),
        }
    }
}

impl<T> ComponentStorage<T> {
    /// Create a new ComponentStorage instance.
    ///
    /// # Returns
    ///
    /// A new ComponentStorage instance.
    fn new() -> Self {
        Self {
            storage: DashMap::new(),
        }
    }

    /// Create a new ComponentStorage instance with a specified
    /// initial capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The capacity of the storage.
    fn new_with_capacity(capacity: usize) -> Self {
        Self {
            storage: DashMap::with_capacity(capacity),
        }
    }

    /// Insert a component into the storage.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to associate the component with.
    fn insert(&self, entity: Entity, component: T) {
        self.storage
            .insert(entity, Arc::new(RwLock::new(component)));
    }

    /// Get a mutable reference to a component.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to get the component for.
    fn get(&self, entity: Entity) -> Option<Arc<RwLock<T>>> {
        self.storage
            .get(&entity)
            .map(|entry| Arc::clone(&entry.value()))
    }

    /// Remove a component from the storage.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove the component for.
    fn remove(&self, entity: Entity) {
        self.storage.remove(&entity);
    }
}

/// The World struct is the main entry point for the ECS system.
/// It is responsible for creating entities and storing components.
#[derive(Debug)]
pub struct World {
    next_entity: AtomicU32,
    storage: DashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Default for World {
    /// Create a new World instance with a default capacity of 41.
    ///
    /// # Returns
    ///
    /// A new World instance.
    fn default() -> Self {
        Self {
            next_entity: AtomicU32::new(0),
            storage: DashMap::with_capacity(41),
        }
    }
}

impl World {
    /// Create a new World instance.
    ///
    /// # Returns
    ///
    /// A new World instance.
    pub fn new() -> Self {
        Self {
            next_entity: AtomicU32::new(0),
            storage: DashMap::new(),
        }
    }

    /// Create a new World instance with a specified
    /// initial capacity.
    ///     
    /// # Returns
    ///
    /// A new World instance.
    pub fn with_capacity(capacity: u32) -> Self {
        Self {
            next_entity: AtomicU32::new(0),
            storage: DashMap::with_capacity(capacity as usize),
        }
    }

    /// Create a new entity.
    ///
    /// # Returns
    ///
    /// The Id of the new entity.
    pub fn create_entity(&mut self) -> Entity {
        self.next_entity.fetch_add(1, Ordering::SeqCst)
    }

    /// Remove an entity from the world with all of it's components.
    ///
    /// # Returns
    ///
    /// True if the entity was removed, false otherwise.
    pub fn remove_entity(&mut self) -> bool {
        let entity_id = self.next_entity.load(Ordering::SeqCst);
        self.storage.iter().for_each(|entry| {
            let storage = entry.value();
            storage
                .as_ref()
                .as_any()
                .downcast_ref::<ComponentStorage<dyn Any>>()
                .map(|component_storage| {
                    component_storage.storage.retain(|key, _| key != &entity_id);
                });
        });
        true
    }

    /// Get the number of entities in the world.
    ///
    /// # Returns
    ///
    /// The number of entities in the world.
    pub fn storage_len(&self) -> usize {
        self.storage.len().checked_sub(1).unwrap_or(0)
    }

    /// Add a component to an entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to add the component to.
    /// * `component` - The component to add.
    pub fn add_component<T: 'static + Send + Sync>(&self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();

        let storage = self.storage.entry(type_id).or_insert_with(|| {
            Arc::new(ComponentStorage::<T>::default()) as Arc<dyn Any + Send + Sync>
        });

        let storage = storage.downcast_ref::<ComponentStorage<T>>().unwrap();
        storage.insert(entity, component);
    }

    /// Remove a component from an entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove the component from.
    pub fn remove_component<T: Component>(&self, entity: Entity) {
        if let Some(storage) = self.storage.get(&TypeId::of::<T>()) {
            let storage = storage.clone();
            let storage = storage.downcast_ref::<ComponentStorage<T>>().unwrap();
            storage.remove(entity);
        }
    }

    /// Get a mutable reference to a component.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to get the component for.
    ///
    /// # Returns
    ///
    /// A mutable reference to the component if it exists.
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<Arc<RwLock<T>>> {
        self.storage.get(&TypeId::of::<T>()).and_then(|storage| {
            let storage = storage.clone();
            let storage = storage.downcast_ref::<ComponentStorage<T>>()?;
            storage.get(entity)
        })
    }

    /// Get mutable references to all components of a specific type.
    ///
    /// # Returns
    ///
    /// A vector of mutable references to the components.
    pub fn get_components<T: Component>(&self) -> Vec<Arc<RwLock<T>>> {
        let storage = self.storage.get(&TypeId::of::<T>());
        if let Some(storage) = storage {
            let storage = storage.clone();
            let storage = storage.downcast_ref::<ComponentStorage<T>>().unwrap();
            storage
                .storage
                .iter()
                .map(|entry| Arc::clone(&entry.value()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get mutable references to all components of a specific type with their associated entities.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing the entity and the component.
    pub fn get_entities_with_component<T: Component>(&self) -> Vec<(Entity, Arc<RwLock<T>>)> {
        let storage = self.storage.get(&TypeId::of::<T>());
        if let Some(storage) = storage {
            let storage = storage.clone();
            let storage = storage.downcast_ref::<ComponentStorage<T>>().unwrap();
            storage
                .storage
                .iter()
                .map(|entry| (entry.key().to_owned(), Arc::clone(&entry.value())))
                .collect()
        } else {
            Vec::new()
        }
    }
}
