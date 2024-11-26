/// A macro to create a new entity and add multiple components when using the EntityBuilder trait.
#[macro_export]
macro_rules! new_entity {
    ($app:expr, $($component:expr),* $(,)?) => {{
        let mut entity_builder = $app.new_entity();
        $(
            entity_builder = entity_builder.add_component($component);
        )*
        entity_builder.build()
    }};
}

// #[macro_export]
// macro_rules! new_entity_prefab {
//     ($app:expr, $prefab:expr) => {{
//         let prefab = $prefab;
//         let components = prefab.unpack_prefab();
//         let mut entity_builder = $app.new_entity();
//         for component in components {
//             entity_builder = entity_builder.add_component(*component);
//         }
//         entity_builder.build()
//     }};
// }

/// A macro to aquire a read only lock for component of an entity.
/// UNSAFE: This macro uses unsafe code to bypass lifetime checks.
/// The caller must ensure the component outlives the returned guard.
#[macro_export]
macro_rules! read_component {
    ($ecs_lock:expr, $entity:expr, $component:ty) => {
        $ecs_lock
            .get_component_from_entity::<$component>($entity)
            .map(|component| unsafe {
                std::mem::transmute::<
                    std::sync::RwLockReadGuard<'_, $component>,
                    std::sync::RwLockReadGuard<'_, $component>,
                >(component.read().unwrap())
            })
    };
}

/// A macro to aquire a write lock for component of an entity.
#[macro_export]
macro_rules! write_component {
    ($ecs_lock:expr, $entity:expr, $component:ty) => {
        $ecs_lock
            .get_component_from_entity::<$component>($entity)
            .map(|component| unsafe {
                std::mem::transmute::<
                    std::sync::RwLockWriteGuard<'_, $component>,
                    std::sync::RwLockWriteGuard<'_, $component>,
                >(component.write().unwrap())
            })
    };
}
