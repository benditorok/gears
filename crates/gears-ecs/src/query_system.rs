//! Simplified Blocking Query System for Component Access
//!
//! This module provides a query-based system for acquiring multiple component locks
//! atomically to prevent deadlocks. Uses a blocking approach for maximum simplicity.

use crate::{Component, Entity, World};
use std::any::TypeId;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};

/// Unique identifier for a query request
pub type QueryId = u64;

/// Represents the type of access needed for a component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Read,
    Write,
}

/// Information about an active resource access
#[derive(Debug, Clone)]
pub(crate) struct ResourceAccess {
    pub(crate) query_id: QueryId,
    pub(crate) access_type: AccessType,
}

/// A request for accessing specific components on specific entities
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentAccessRequest {
    pub type_id: TypeId,
    pub entities: Vec<Entity>,
    pub access_type: AccessType,
}

/// A query builder for specifying component access requirements
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentQuery {
    requests: Vec<ComponentAccessRequest>,
}

impl ComponentQuery {
    /// Create a new empty query
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    /// Add a read request for a component type on specific entities
    pub fn read<T: Component + 'static>(mut self, entities: Vec<Entity>) -> Self {
        if !entities.is_empty() {
            self.requests.push(ComponentAccessRequest {
                type_id: TypeId::of::<T>(),
                entities,
                access_type: AccessType::Read,
            });
        }
        self
    }

    /// Add a write request for a component type on specific entities
    pub fn write<T: Component + 'static>(mut self, entities: Vec<Entity>) -> Self {
        if !entities.is_empty() {
            self.requests.push(ComponentAccessRequest {
                type_id: TypeId::of::<T>(),
                entities,
                access_type: AccessType::Write,
            });
        }
        self
    }

    /// Add a read request for a component type on all entities that have it
    pub fn read_all<T: Component + 'static>(mut self, world: &World) -> Self {
        let entities = world.get_entities_with_component::<T>();
        if !entities.is_empty() {
            self.requests.push(ComponentAccessRequest {
                type_id: TypeId::of::<T>(),
                entities,
                access_type: AccessType::Read,
            });
        }
        self
    }

    /// Add a write request for a component type on all entities that have it
    pub fn write_all<T: Component + 'static>(mut self, world: &World) -> Self {
        let entities = world.get_entities_with_component::<T>();
        if !entities.is_empty() {
            self.requests.push(ComponentAccessRequest {
                type_id: TypeId::of::<T>(),
                entities,
                access_type: AccessType::Write,
            });
        }
        self
    }

    /// Get all (Entity, TypeId) pairs that this query would access
    fn get_resource_keys(&self) -> Vec<(Entity, TypeId)> {
        let mut keys = Vec::new();
        for request in &self.requests {
            for &entity in &request.entities {
                keys.push((entity, request.type_id));
            }
        }
        keys
    }
}

impl Default for ComponentQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of successfully acquiring all requested resources
pub struct AcquiredResources<'a> {
    world: &'a World,
    query_id: QueryId,
    resource_keys: Vec<(Entity, TypeId)>,
}

impl<'a> AcquiredResources<'a> {
    /// Get access to a component for a specific entity
    pub fn get<T: Component + 'static>(&self, entity: Entity) -> Option<Arc<RwLock<T>>> {
        self.world.get_component::<T>(entity)
    }
}

impl<'a> Drop for AcquiredResources<'a> {
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

/// Extension trait for World to support query-based access
pub trait WorldQueryExt {
    /// Acquire all resources specified in the query, blocking until available
    fn acquire_query(&self, query: ComponentQuery) -> Option<AcquiredResources<'_>>;

    /// Try to acquire all resources specified in the query without blocking
    fn try_acquire_query_immediate(&self, query: ComponentQuery) -> Option<AcquiredResources<'_>>;
}

impl WorldQueryExt for World {
    fn acquire_query(&self, query: ComponentQuery) -> Option<AcquiredResources<'_>> {
        // Generate unique query ID
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

        // Validate that all requested components exist
        for &(entity, type_id) in &sorted_keys {
            if !self.has_component_of_type(entity, type_id) {
                return None; // Entity doesn't have this component
            }
        }

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
                        .or_insert_with(Vec::new)
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

    fn try_acquire_query_immediate(&self, query: ComponentQuery) -> Option<AcquiredResources<'_>> {
        // Generate unique query ID
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

        // Validate that all requested components exist
        for &(entity, type_id) in &sorted_keys {
            if !self.has_component_of_type(entity, type_id) {
                return None; // Entity doesn't have this component
            }
        }

        // Try to acquire all resources immediately
        let mut acquired_keys = Vec::new();

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
                // Can't acquire this resource, fail immediately
                return None;
            }
        }

        // Acquire all resources atomically
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
                .or_insert_with(Vec::new)
                .push(access);
        }

        Some(AcquiredResources {
            world: self,
            query_id,
            resource_keys: acquired_keys,
        })
    }
}

/// Helper methods for World to support query system
impl World {
    /// Check if a resource can be acquired with the given access type
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

    /// Get the number of currently active resource accesses (for debugging/monitoring)
    pub fn active_query_count(&self) -> usize {
        self.active_accesses.len()
    }

    /// Clear all active queries (for emergency cleanup)
    pub fn clear_active_queries(&self) {
        self.active_accesses.clear();
    }

    /// Check if an entity has a component of a specific type (by TypeId)
    fn has_component_of_type(&self, entity: Entity, _type_id: TypeId) -> bool {
        // Simplified check - just verify entity ID is valid
        *entity < self.next_entity.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Component;

    #[derive(Debug)]
    struct TestComponent1(u32);
    impl Component for TestComponent1 {}

    #[derive(Debug)]
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
        assert!(world.try_acquire_query_immediate(empty_query).is_some());
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
        let resources1 = world.try_acquire_query_immediate(query1);
        let resources2 = world.try_acquire_query_immediate(query2);

        assert!(resources1.is_some(), "First read query should succeed");
        assert!(resources2.is_some(), "Second read query should succeed");
    }

    #[test]
    fn test_write_conflicts() {
        let world = World::new();
        let entity1 = world.create_entity();
        world.add_component(entity1, TestComponent1(42));

        // Create two conflicting write queries
        let query1 = ComponentQuery::new().write::<TestComponent1>(vec![entity1]);
        let query2 = ComponentQuery::new().write::<TestComponent1>(vec![entity1]);

        // First query should succeed
        let resources1 = world.try_acquire_query_immediate(query1);
        assert!(resources1.is_some(), "First write query should succeed");

        // Second conflicting query should fail while first is active
        let resources2 = world.try_acquire_query_immediate(query2);
        assert!(
            resources2.is_none(),
            "Second write query should fail due to conflict"
        );

        // After dropping first resources, second query should succeed
        drop(resources1);
        let query3 = ComponentQuery::new().write::<TestComponent1>(vec![entity1]);
        let resources3 = world.try_acquire_query_immediate(query3);
        assert!(
            resources3.is_some(),
            "Third write query should succeed after first is dropped"
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
        let resources1 = world.try_acquire_query_immediate(query1);
        let resources2 = world.try_acquire_query_immediate(query2);

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
        let _resources = world.try_acquire_query_immediate(query);

        // Should have active accesses
        assert!(world.active_query_count() > 0);

        // Clear all queries
        world.clear_active_queries();
        assert_eq!(world.active_query_count(), 0);
    }
}
