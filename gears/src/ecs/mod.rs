pub mod components;
pub mod utils;

use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};

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

    /// Get the number of entities with components stored.
    ///
    /// # Returns
    ///
    /// The number of entities with components stored.
    pub fn storage_len(&self) -> usize {
        self.storage.len()
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

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;

    #[derive(Debug)]
    struct TestComp(u32);
    impl Component for TestComp {}

    #[test]
    fn test_insert_component_into_storage() {
        let storage = ComponentStorage::<TestComp>::new();
        let entity = Entity::new(1);
        storage.insert(entity, TestComp(999));
        assert_eq!(storage.storage.len(), 1);
    }

    #[test]
    fn test_par_insert_into_storage() {
        let storage = ComponentStorage::<TestComp>::new();
        (0..100).into_par_iter().for_each(|i| {
            let entity = Entity::new(i);
            storage.insert(entity, TestComp(i * 10));
        });
        assert_eq!(storage.storage.len(), 100);
    }

    #[test]
    fn test_get_component_from_storage() {
        let storage = ComponentStorage::<TestComp>::new();
        let entity = Entity::new(1);
        storage.insert(entity, TestComp(999));
        let retrieved = storage.get(entity).unwrap();
        assert_eq!(retrieved.read().unwrap().0, 999);
    }

    #[test]
    fn test_remove_component_from_storage() {
        let storage = ComponentStorage::<TestComp>::new();
        let entity = Entity::new(1);
        storage.insert(entity, TestComp(999));
        storage.remove(entity);
        assert!(storage.get(entity).is_none());
    }

    #[test]
    fn test_iter_components_in_storage() {
        let storage = ComponentStorage::<TestComp>::new();
        for i in 0..3 {
            let entity = Entity::new(i);
            storage.insert(entity, TestComp(i * 10));
        }
        let comps: Vec<_> = storage.iter_components().collect();
        assert_eq!(comps.len(), 3);
    }

    #[test]
    fn test_iter_entities_in_storage() {
        let storage = ComponentStorage::<TestComp>::new();
        for i in 0..3 {
            let entity = Entity::new(i);
            storage.insert(entity, TestComp(i));
        }
        let entities: Vec<_> = storage.iter_entities().collect();
        assert_eq!(entities.len(), 3);
    }

    #[test]
    fn test_create_and_remove_entity() {
        let world = World::new();
        let entity = world.create_entity();
        assert_eq!(*entity, 0);

        world.remove_entity(entity);

        // Just validate it doesn't panic and we can still create a new entity
        let new_entity = world.create_entity();
        assert_eq!(*new_entity, 1);
    }

    #[test]
    fn test_add_and_get_component() {
        let world = World::new();
        let entity = world.create_entity();
        world.add_component(entity, TestComp(123));
        let comp = world.get_component::<TestComp>(entity).unwrap();
        assert_eq!(comp.read().unwrap().0, 123);
    }

    #[test]
    fn test_component_storage_insert_and_get() {
        let storage = ComponentStorage::<TestComp>::new();
        let entity = Entity::new(42);
        storage.insert(entity, TestComp(999));
        let retrieved = storage.get(entity).unwrap();
        assert_eq!(retrieved.read().unwrap().0, 999);
    }

    #[test]
    fn test_parallel_entity_creation() {
        let world = World::new();
        // Create many entities in parallel
        (0..100).into_par_iter().for_each(|_| {
            world.create_entity();
        });
        // Just ensure no panic and entity count is 100
        assert_eq!(*world.get_last().unwrap(), 99);
    }

    #[test]
    fn test_get_component_and_write() {
        let world = World::new();
        let entity = world.create_entity();
        world.add_component(entity, TestComp(123));
        let comp = world.get_component::<TestComp>(entity).unwrap();

        {
            let mut write = comp.write().unwrap();
            write.0 = 456;
        }

        let comp = world.get_component::<TestComp>(entity).unwrap();
        assert_eq!(comp.read().unwrap().0, 456);
    }

    #[test]
    fn test_get_component_write_lock_simultaneously() {
        let world = World::new();
        let entity = world.create_entity();
        world.add_component(entity, TestComp(123));

        let comp = world.get_component::<TestComp>(entity).unwrap();
        let _write = comp.write().unwrap();

        // Hold the lock while we try to get another write lock on a reference which should fail
        let comp2 = world.get_component::<TestComp>(entity).unwrap();
        let write2 = comp2.try_write();
        assert!(write2.is_err());
    }

    #[test]
    fn test_get_component_read_lock_simultaneously() {
        let world = World::new();
        let entity = world.create_entity();
        world.add_component(entity, TestComp(123));

        let comp = world.get_component::<TestComp>(entity).unwrap();
        let read = comp.read().unwrap();

        // Get a second read lock on the same component
        let comp2 = world.get_component::<TestComp>(entity).unwrap();
        let read2 = comp2.read().unwrap();

        // Ensure we can still read the value from both locks
        assert_eq!(read.0, 123);
        assert_eq!(read2.0, 123);
    }
}
