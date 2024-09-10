pub mod components;
pub mod utils;

use core::fmt;
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    sync::{Arc, Mutex, RwLock, RwLockReadGuard},
};

pub trait ComponentVec {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn push_none(&mut self);
}

impl<ComponentType: 'static> ComponentVec for RefCell<Vec<Option<ComponentType>>> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    fn push_none(&mut self) {
        self.get_mut().push(None)
    }
}

impl<ComponentType: 'static> ComponentVec for RwLock<Vec<Option<ComponentType>>> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    fn push_none(&mut self) {
        self.get_mut().unwrap().push(None);
    }
}

impl<ComponentType: 'static> ComponentVec for Arc<RwLock<Vec<Option<ComponentType>>>> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    fn push_none(&mut self) {
        self.write().unwrap().push(None);
    }
}

pub struct World {
    entities_count: Arc<RwLock<usize>>,
    component_vecs: Arc<Mutex<Vec<Arc<RwLock<Box<dyn ComponentVec>>>>>>,
}

impl fmt::Display for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "World {{ entities_count: {}, component_vecs: {:?} }}",
            self.entities_count.read().unwrap(),
            self.component_vecs.lock().unwrap().len()
        )
    }
}

impl World {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            entities_count: Arc::new(RwLock::new(0)),
            component_vecs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn new_entity(&self) -> usize {
        let mut entities_count = self.entities_count.write().unwrap();
        let mut component_vecs = self.component_vecs.lock().unwrap();

        let entity_id = *entities_count;

        for component_vec in component_vecs.iter_mut() {
            component_vec.write().unwrap().push_none();
        }

        *entities_count += 1;
        entity_id
    }

    pub fn add_component_to_entity<ComponentType: 'static>(
        &self,
        entity: usize,
        component: ComponentType,
    ) {
        let mut component_vecs_lock = self.component_vecs.lock().unwrap();

        // Search for any existing ComponentVecs that match the type of the component being added.
        for component_vec in component_vecs_lock.iter_mut() {
            if let Some(component_vec) = component_vec
                .write()
                .unwrap()
                .as_any_mut()
                .downcast_mut::<Arc<RwLock<Vec<Option<ComponentType>>>>>()
            {
                let mut component_vec_wlock = component_vec.write().unwrap();

                if entity < component_vec_wlock.len() {
                    component_vec_wlock[entity] = Some(component);
                } else {
                    component_vec_wlock.push(Some(component));
                }

                return;
            }
        }

        // No matching ComponentVec found, so we create a new one.
        let mut new_component_vec = Vec::with_capacity(*self.entities_count.read().unwrap());
        new_component_vec.resize_with(*self.entities_count.read().unwrap(), || None);
        new_component_vec[entity] = Some(component);

        component_vecs_lock.push(Arc::new(RwLock::new(
            Box::new(RefCell::new(new_component_vec)) as Box<dyn ComponentVec>,
        )));
    }

    pub fn borrow_component_vec_mut<ComponentType: 'static>(
        &self,
    ) -> Option<Arc<RwLock<Vec<Option<ComponentType>>>>> {
        for component_vec_lock in self.component_vecs.lock().unwrap().iter() {
            if let Some(component_vec) = component_vec_lock
                .read()
                .unwrap()
                .as_any()
                .downcast_ref::<Arc<RwLock<Vec<Option<ComponentType>>>>>()
            {
                return Some(Arc::clone(component_vec));
            }
        }

        None
    }
}

pub struct WorldSingle {
    entities_count: usize,
    component_vecs: Vec<Box<dyn ComponentVec>>,
}

impl WorldSingle {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            entities_count: 0,
            component_vecs: Vec::new(),
        }
    }

    pub fn new_entity(&mut self) -> usize {
        let entity_id = self.entities_count;

        for component_vec in self.component_vecs.iter_mut() {
            component_vec.push_none();
        }

        self.entities_count += 1;
        entity_id
    }

    pub fn add_component_to_entity<ComponentType: 'static>(
        &mut self,
        entity: usize,
        component: ComponentType,
    ) {
        // Search for any existing ComponentVecs that match the type of the component being added.
        for component_vec in self.component_vecs.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
            {
                component_vec.borrow_mut()[entity] = Some(component);
                return;
            }
        }

        // No matching component storage exists yet, so we have to make one.
        let mut new_component_vec: Vec<Option<ComponentType>> =
            Vec::with_capacity(self.entities_count);

        // All existing entities don't have this component, so we give them `None`
        for _ in 0..self.entities_count {
            new_component_vec.push(None);
        }

        // Give this Entity the Component.
        new_component_vec[entity] = Some(component);
        self.component_vecs
            .push(Box::new(RefCell::new(new_component_vec)));
    }

    pub fn borrow_component_vec_mut<ComponentType: 'static>(
        &self,
    ) -> Option<RefMut<Vec<Option<ComponentType>>>> {
        for component_vec in self.component_vecs.iter() {
            if let Some(component_vec) = component_vec
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return Some(component_vec.borrow_mut());
            }
        }

        None
    }
}

// pub struct WorldConcurrent {
//     entities_count: usize,
//     entites: Arc<RwLock<HashMap<usize, Box<dyn ComponentVec>>>>,
// }

// impl WorldConcurrent {
//     fn new() -> Self {
//         Self {
//             entities_count: 0,
//             entites: Arc::new(RwLock::new(HashMap::new())),
//         }
//     }

//     pub fn new_entity(&mut self) -> usize {
//         let entity = self.entities_count;

//         let entities = Arc::clone(&self.entites);
//         let mut entities_val = entities.write().unwrap();
//         entities_val.get_mut(&entity).unwrap().push_none();

//         self.entities_count += 1;
//         entity
//     }

//     pub fn add_component_to_entity<ComponentType: 'static>(
//         &mut self,
//         entity: usize,
//         component: ComponentType,
//     ) {
//         let entities = Arc::clone(&self.entites);
//         let mut entities_val = entities.write().unwrap();

//         if let Some(entity) = entities_val.get_mut(&entity) {
//             if let Some(component_vec) = entity
//                 .as_any_mut()
//                 .downcast_mut::<RwLock<Vec<Option<ComponentType>>>>()
//             {
//                 let mut component_vec = component_vec.write().unwrap();
//                 if !component_vec.is_empty() {
//                     component_vec[entity] = Some(component);
//                     return;
//                 }
//             }
//         } else {
//             let mut new_component_vec: Vec<Option<ComponentType>> =
//                 Vec::with_capacity(self.entities_count);
//                 new_component_vec

//             for _ in 0..self.entities_count {
//                 new_component_vec.push(None);
//             }

//             new_component_vec[entity] = Some(component);
//             entities_val.insert(entity, Box::new(RwLock::new(new_component_vec)));
//         }

//         // No matching component storage exists yet, so we have to make one.
//         let mut new_component_vec: Vec<Option<ComponentType>> =
//             Vec::with_capacity(self.entities_count);

//         // All existing entities don't have this component, so we give them `None`
//         for _ in 0..self.entities_count {
//             new_component_vec.push(None);
//         }

//         // TODO az elozo megoldasba mindegyiken vegigmenni es az idx- komponensneket visszaadni --> megkapjuk 1 entity adott elemeit

//         // Give this Entity the Component.
//         new_component_vec[entity] = Some(component);
//         self.component_vecs
//             .push(Box::new(RefCell::new(new_component_vec)));
//     }
// }
