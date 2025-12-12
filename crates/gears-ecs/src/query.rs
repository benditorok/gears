use crate::{Component, Entity, World};
use std::any::TypeId;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};

/// Unique identifier for a query request.
pub type QueryId = u64;

/// Represents the type of access needed for a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Read,
    Write,
}

/// Information about an active resource access.
#[derive(Debug, Clone)]
pub(crate) struct ResourceAccess {
    /// Unique identifier for the query request.
    pub(crate) query_id: QueryId,
    /// Type of access needed for the component.
    pub(crate) access_type: AccessType,
}

/// A request for accessing specific components on specific entities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentAccessRequest {
    /// Type identifier for the component.
    pub type_id: TypeId,
    /// Entities on which the component access is requested.
    pub entities: Vec<Entity>,
    /// Type of access needed for the component.
    pub access_type: AccessType,
}

/// A query builder for specifying component access requirements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentQuery {
    /// Component access requests.
    requests: Vec<ComponentAccessRequest>,
}

impl ComponentQuery {
    /// Create a new empty query.
    ///
    /// # Returns
    ///
    /// A new [`ComponentQuery`] instance.
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    /// Add a read request for a component type on specific entities.
    ///
    /// # Arguments
    ///
    /// * `entities` - The entities on which the component access is requested.
    ///
    /// # Returns
    ///
    /// The updated [`ComponentQuery`] instance.
    pub fn read<T: Component>(mut self, entities: Vec<Entity>) -> Self {
        if !entities.is_empty() {
            self.requests.push(ComponentAccessRequest {
                type_id: TypeId::of::<T>(),
                entities,
                access_type: AccessType::Read,
            });
        }
        self
    }

    /// Add a write request for a component type on specific entities.
    ///
    /// # Arguments
    ///
    /// * `entities` - The entities on which the component access is requested.
    ///
    /// # Returns
    ///
    /// The updated [`ComponentQuery`] instance.
    pub fn write<T: Component>(mut self, entities: Vec<Entity>) -> Self {
        if !entities.is_empty() {
            self.requests.push(ComponentAccessRequest {
                type_id: TypeId::of::<T>(),
                entities,
                access_type: AccessType::Write,
            });
        }
        self
    }

    /// Get all entity-component pairs that this query would access.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing the entity and component type ID.
    fn get_resource_keys(&self) -> Vec<(Entity, TypeId)> {
        self.requests
            .iter()
            .flat_map(|req| req.entities.iter().map(|&entity| (entity, req.type_id)))
            .collect()
    }
}

impl Default for ComponentQuery {
    /// Create a default component query.
    ///
    /// # Returns
    ///
    /// A new [`ComponentQuery`] instance.
    fn default() -> Self {
        Self::new()
    }
}

/// Result of successfully acquiring all requested resources.
pub struct AcquiredResources<'a> {
    /// The world from which the resources were acquired.
    world: &'a World,
    /// The Id of the query that acquired these resources.
    query_id: QueryId,
    /// The keys of the resources that were acquired.
    resource_keys: Vec<(Entity, TypeId)>,
}

impl<'a> AcquiredResources<'a> {
    /// Get access to a component for a specific entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity for which to get the component.
    ///
    /// # Returns
    ///
    /// An [`Arc<RwLock<T>>`] reference to the component if it exists.
    pub fn get<T: Component + 'static>(&self, entity: Entity) -> Option<Arc<RwLock<T>>> {
        self.world.get_component::<T>(entity)
    }
}

impl<'a> Drop for AcquiredResources<'a> {
    /// Drop the acquired resources, releasing their access.
    fn drop(&mut self) {
        // Remove this query's access from all resource keys
        for &key in &self.resource_keys {
            if let Some(mut accesses) = self.world.active_accesses.get_mut(&key) {
                accesses.retain(|access| access.query_id != self.query_id);
                if accesses.is_empty() {
                    drop(accesses);
                    self.world.active_accesses.remove(&key);
                }
            }
        }
    }
}

/// Extension trait for World to support query-based access.
pub trait WorldQueryExt {
    /// Acquire all resources specified in the query, blocking until available.
    ///
    /// # Arguments
    ///
    /// * `query` - The query to acquire resources for.
    ///
    /// # Returns
    ///
    /// An [`AcquiredResources`] containing the acquired resources if it could be acquired.
    fn acquire_query(&self, query: ComponentQuery) -> Option<AcquiredResources<'_>>;
}

impl WorldQueryExt for World {
    /// Acquire all resources specified in the query, blocking until available.
    ///
    /// # Arguments
    ///
    /// * `query` - The query to acquire resources for.
    ///
    /// # Returns
    ///
    /// An [`AcquiredResources`] containing the acquired resources if it could be acquired.
    fn acquire_query(&self, query: ComponentQuery) -> Option<AcquiredResources<'_>> {
        // Generate unique query Id
        let query_id = self.next_query_id.fetch_add(1, Ordering::Relaxed);

        // Get all resource keys this query needs
        let resource_keys = query.get_resource_keys();
        if resource_keys.is_empty() {
            // Empty query always succeeds
            return Some(AcquiredResources {
                world: self,
                query_id,
                resource_keys: Vec::new(),
            });
        }

        // Sort resource keys to ensure consistent ordering and prevent deadlocks
        let mut sorted_keys = resource_keys;
        sorted_keys.sort_by_key(|&(entity, type_id)| (*entity, type_id));
        sorted_keys.dedup();

        // Block until we can acquire all resources
        loop {
            let mut acquired_keys = Vec::new();
            let mut can_acquire_all = true;

            // Try to acquire all resources atomically
            for &(entity, type_id) in &sorted_keys {
                // Find the access type for this resource
                let access_type = query
                    .requests
                    .iter()
                    .find(|req| req.type_id == type_id && req.entities.contains(&entity))
                    .map(|req| req.access_type)
                    .unwrap_or(AccessType::Read);

                // Check if we can acquire this resource
                if self.can_acquire_resource((entity, type_id), access_type) {
                    acquired_keys.push((entity, type_id));
                } else {
                    can_acquire_all = false;
                    break;
                }
            }

            if can_acquire_all {
                // We can acquire all resources, do it atomically
                for &(entity, type_id) in &acquired_keys {
                    let access_type = query
                        .requests
                        .iter()
                        .find(|req| req.type_id == type_id && req.entities.contains(&entity))
                        .map(|req| req.access_type)
                        .unwrap_or(AccessType::Read);

                    let access = ResourceAccess {
                        query_id,
                        access_type,
                    };

                    self.active_accesses
                        .entry((entity, type_id))
                        .or_default()
                        .push(access);
                }

                return Some(AcquiredResources {
                    world: self,
                    query_id,
                    resource_keys: acquired_keys,
                });
            }

            // Can't acquire all resources right now, yield and try again
            std::thread::yield_now();
        }
    }
}

/// Helper methods for World to support query system
impl World {
    /// Check if a resource can be acquired with the given access type.
    ///
    /// # Arguments
    ///
    /// * `key` - The resource key to check.
    /// * `access_type` - The access type to check.
    ///
    /// # Returns
    ///
    /// Returns `true` if the resource can be acquired.
    fn can_acquire_resource(&self, key: (Entity, TypeId), access_type: AccessType) -> bool {
        if let Some(accesses) = self.active_accesses.get(&key) {
            // Check for conflicts with active accesses
            for access in accesses.iter() {
                match (access_type, access.access_type) {
                    // Write conflicts with everything
                    (AccessType::Write, _) | (_, AccessType::Write) => return false,
                    // Read-Read is always safe
                    (AccessType::Read, AccessType::Read) => continue,
                }
            }
        }
        true
    }

    /// Get the number of currently active resource accesses (for debugging/monitoring).
    ///
    /// # Returns
    ///
    /// Returns the number of currently active resource accesses.
    pub fn active_query_count(&self) -> usize {
        self.active_accesses.len()
    }

    /// Clear all active queries (for emergency cleanup).
    ///
    /// # Safety
    ///
    /// This function should only be called when it is safe to discard all active queries.
    ///
    /// # Returns
    ///
    /// Returns `true` if the active queries were successfully cleared.
    pub fn clear_active_queries(&self) -> bool {
        self.active_accesses.clear();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Component;

    #[derive(Debug)]
    #[allow(unused)]
    struct TestComponent1(u32);
    impl Component for TestComponent1 {}

    #[derive(Debug)]
    #[allow(unused)]
    struct TestComponent2(String);
    impl Component for TestComponent2 {}

    #[test]
    fn test_query_builder() {
        let world = World::new();
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        let query = ComponentQuery::new()
            .read::<TestComponent1>(vec![entity1])
            .write::<TestComponent2>(vec![entity2]);

        assert_eq!(query.requests.len(), 2);
        assert_eq!(query.requests[0].access_type, AccessType::Read);
        assert_eq!(query.requests[1].access_type, AccessType::Write);
    }

    #[test]
    fn test_resource_keys() {
        let world = World::new();
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        let query = ComponentQuery::new()
            .read::<TestComponent1>(vec![entity1])
            .write::<TestComponent2>(vec![entity2]);

        let keys = query.get_resource_keys();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_empty_query() {
        let world = World::new();
        let empty_query = ComponentQuery::new();
        assert!(world.acquire_query(empty_query).is_some());
    }

    #[test]
    fn test_concurrent_read_access() {
        let world = World::new();
        let entity1 = world.create_entity();
        world.add_component(entity1, TestComponent1(42));

        // Create two read queries for the same entity
        let query1 = ComponentQuery::new().read::<TestComponent1>(vec![entity1]);
        let query2 = ComponentQuery::new().read::<TestComponent1>(vec![entity1]);

        // Both read queries should succeed
        let resources1 = world.acquire_query(query1);
        let resources2 = world.acquire_query(query2);

        assert!(resources1.is_some(), "First read query should succeed");
        assert!(resources2.is_some(), "Second read query should succeed");
    }

    #[test]
    fn test_write_conflicts() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::thread;
        use std::time::Duration;

        let world = Arc::new(World::new());
        let entity1 = world.create_entity();
        world.add_component(entity1, TestComponent1(42));

        // First query should succeed
        let query1 = ComponentQuery::new().write::<TestComponent1>(vec![entity1]);
        let resources1 = world.acquire_query(query1);
        assert!(resources1.is_some(), "First write query should succeed");

        // Second conflicting query should block while first is active
        let world_clone = Arc::clone(&world);
        let second_acquired = Arc::new(AtomicBool::new(false));
        let second_acquired_clone = Arc::clone(&second_acquired);

        let thread_handle = thread::spawn(move || {
            let query2 = ComponentQuery::new().write::<TestComponent1>(vec![entity1]);
            let _resources2 = world_clone.acquire_query(query2);
            second_acquired_clone.store(true, Ordering::SeqCst);
        });

        // Give the thread time to start and attempt to acquire
        thread::sleep(Duration::from_millis(50));

        // Second query should still be blocked
        assert!(
            !second_acquired.load(Ordering::SeqCst),
            "Second write query should be blocked while first is active"
        );

        // After dropping first resources, second query should succeed
        drop(resources1);

        // Wait for the second thread to acquire the resource
        thread_handle
            .join()
            .expect("Thread should complete successfully");

        assert!(
            second_acquired.load(Ordering::SeqCst),
            "Second write query should succeed after first is dropped"
        );
    }

    #[test]
    fn test_non_conflicting_entities() {
        let world = World::new();
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();
        world.add_component(entity1, TestComponent1(42));
        world.add_component(entity2, TestComponent1(99));

        // Create two non-conflicting queries (different entities)
        let query1 = ComponentQuery::new().write::<TestComponent1>(vec![entity1]);
        let query2 = ComponentQuery::new().write::<TestComponent1>(vec![entity2]);

        // Both queries should succeed since they access different entities
        let resources1 = world.acquire_query(query1);
        let resources2 = world.acquire_query(query2);

        assert!(resources1.is_some(), "First query should succeed");
        assert!(
            resources2.is_some(),
            "Second query should succeed (different entity)"
        );
    }

    #[test]
    fn test_cleanup() {
        let world = World::new();
        let entity = world.create_entity();
        world.add_component(entity, TestComponent1(42));

        let query = ComponentQuery::new().write::<TestComponent1>(vec![entity]);
        let _resources = world.acquire_query(query);

        // Should have active accesses
        assert!(world.active_query_count() > 0);

        // Clear all queries
        world.clear_active_queries();
        assert_eq!(world.active_query_count(), 0);
    }
}
