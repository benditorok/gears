pub mod components;
pub mod utils;

use dashmap::{mapref::one::RefMut, DashMap};
use gltf::accessor::Item;
use log::info;
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// An entity is a unique identifier that can be attached to components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u32);

impl Entity {
    /// Create a new entity.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the entity.
    ///
    /// # Returns
    ///
    /// A new entity.
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl Deref for Entity {
    type Target = u32;

    /// Get the Id field of the entity.
    ///
    /// # Returns
    ///
    /// The id of the entity.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u32> for Entity {
    /// Create a new entity from a number.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the entity.
    ///
    /// # Returns
    ///
    /// A new entity.
    fn from(id: u32) -> Self {
        Self(id)
    }
}

/// The EntityBuilder trait is responsible for creating entities and adding components to them.
pub trait EntityBuilder {
    fn new_entity(&mut self) -> &mut Self;
    fn add_component(&mut self, component: impl Component) -> &mut Self;
    fn build(&mut self) -> Entity;
}

/// A component marker that can be attached to an entity.
pub trait Component: Send + Sync + Any + Debug {}

impl Component for Box<dyn Component> {}

/// The ComponentStorage struct is responsible for storing components of a specific type.
/// It uses a DashMap to store the components, which allows for concurrent reads and writes.
/// The components are stored in an Arc<RwLock<T>> to allow for multiple reads or a single write.
struct ComponentStorage<T> {
    storage: DashMap<Entity, Arc<RwLock<T>>>,
}

impl<T: Component> Default for ComponentStorage<T> {
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

impl<T: Component> ComponentStorage<T> {
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

    /// Get a reference to the storage as an Any trait object.
    ///
    /// # Returns
    ///
    /// A reference to the storage as an Any trait object.
    fn as_any(&self) -> &dyn std::any::Any {
        self
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

    /// Get a reference to a component.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to get the component for.
    fn get(&self, entity: Entity) -> Option<Arc<RwLock<T>>> {
        self.storage
            .get(&entity)
            .map(|component| Arc::clone(&component))
    }

    /// Remove a component from the storage.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove the component for.
    fn remove(&self, entity: Entity) {
        self.storage.remove(&entity);
    }

    /// Get an iterator over the entities in the storage.
    ///
    /// # Returns
    ///
    /// An iterator over the entities in the storage.
    fn iter_components(&self) -> Box<dyn Iterator<Item = Arc<RwLock<T>>> + '_> {
        Box::new(self.storage.iter().map(|entry| Arc::clone(entry.value())))
    }

    /// Get an iterator over the entities in the storage.
    ///
    /// # Returns
    ///
    /// An iterator over the entities in the storage.
    fn iter_entities(&self) -> Box<dyn Iterator<Item = Entity> + '_> {
        Box::new(self.storage.iter().map(|entry| entry.key().to_owned()))
    }
}

/// The World struct is the main entry point for the ECS system.
/// It is responsible for creating entities and storing components.
pub struct World {
    next_entity: AtomicU32,
    // Change storage to hold Arc<dyn Any>
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

    /// Get the number of entities in the world.
    ///
    /// # Returns
    ///
    /// The number of entities in the world.
    pub fn storage_len(&self) -> usize {
        self.storage.len().saturating_sub(1)
    }

    /// Get the Id of the last entity created.
    ///
    /// # Returns
    ///
    /// The Id of the last entity created.
    pub fn get_last(&self) -> Option<Entity> {
        self.next_entity
            .load(Ordering::SeqCst)
            .checked_sub(1)
            .map(|id| id.into())
    }

    /// Create a new entity.
    ///
    /// # Returns
    ///
    /// The Id of the new entity.
    pub fn create_entity(&self) -> Entity {
        self.next_entity.fetch_add(1, Ordering::SeqCst).into()
    }

    /// Remove an entity from the world with all of its components.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove.
    pub fn remove_entity(&self, entity: Entity) {
        for entry in self.storage.iter() {
            let storage = entry.value();
            if let Some(typed_storage) =
                storage.downcast_ref::<ComponentStorage<Box<dyn Component>>>()
            {
                typed_storage.remove(entity);
            }
        }
    }

    /// Add a component to an entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to add the component to.
    /// * `component` - The component to add.
    pub fn add_component<T: Component>(&self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let entry = self.storage.entry(type_id).or_insert_with(|| {
            Arc::new(ComponentStorage::<T>::new()) as Arc<dyn Any + Send + Sync>
        });
        // Downcast to the correct storage type
        let typed_storage = entry.downcast_ref::<ComponentStorage<T>>().unwrap();
        typed_storage.insert(entity, component);
    }

    /// Remove a component from an entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove the component from.
    pub fn remove_component<T: Component>(&self, entity: Entity) {
        if let Some(entry) = self.storage.get(&TypeId::of::<T>()) {
            if let Some(typed_storage) = entry.downcast_ref::<ComponentStorage<T>>() {
                typed_storage.remove(entity);
            }
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
        let entry = self.storage.get(&TypeId::of::<T>())?;
        if let Some(typed_storage) = entry.downcast_ref::<ComponentStorage<T>>() {
            typed_storage.get(entity)
        } else {
            None
        }
    }

    /// Get mutable references to all components of a specific type.
    ///
    /// # Returns
    ///
    /// A vector of mutable references to the components.
    pub fn get_components<T: Component + 'static>(&self) -> Vec<Arc<RwLock<dyn Component>>> {
        if let Some(entry) = self.storage.get(&TypeId::of::<T>()) {
            if let Some(typed_storage) = entry.downcast_ref::<ComponentStorage<T>>() {
                typed_storage
                    .iter_components()
                    .map(|component| {
                        let component = Arc::clone(&component);
                        unsafe {
                            let raw = Arc::into_raw(component);
                            let raw = raw as *const RwLock<dyn Component>;
                            Arc::from_raw(raw)
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    /// Get all entities which have a specific component.
    ///
    /// # Returns
    ///
    /// A vector of entities which have the specified component.
    pub fn get_entities_with_component<T: Component>(&self) -> Vec<Entity> {
        if let Some(entry) = self.storage.get(&TypeId::of::<T>()) {
            if let Some(typed_storage) = entry.downcast_ref::<ComponentStorage<T>>() {
                return typed_storage.iter_entities().collect();
            }
        }
        Vec::new()
    }

    /// Get mutable references to all components of a specific type with their associated entities.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing the entity and the component.
    pub fn get_entities_and_component<T: Component>(&self) -> Vec<(Entity, Arc<RwLock<T>>)> {
        if let Some(entry) = self.storage.get(&TypeId::of::<T>()) {
            if let Some(typed_storage) = entry.downcast_ref::<ComponentStorage<T>>() {
                return typed_storage
                    .iter_entities()
                    .filter_map(|entity| typed_storage.get(entity).map(|c| (entity, c)))
                    .collect();
            }
        }
        Vec::new()
    }
}
