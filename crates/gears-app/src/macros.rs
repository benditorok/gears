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

/// A macro to create async systems easily.
///
/// This macro provides several patterns for creating async systems, from simple closures
/// to complex systems that capture and clone external variables. The macro automatically
/// handles the async system creation and registration.
///
/// # Patterns
///
/// ## 1. Variable Capture with Auto-Cloning (Returns System)
/// ```rust
/// use gears_app::prelude::*;
/// use std::sync::{Arc, Mutex};
/// use std::sync::mpsc;
///
/// let sender = mpsc::channel().0;
/// let counter = Arc::new(Mutex::new(0));
///
/// let _system = async_system!("update", (sender, counter), |world, dt| {
///     // Variables are automatically cloned before the async block
///     let _ = sender.send(dt);
///     *counter.lock().unwrap() += 1;
///     Ok(())
/// });
/// ```
///
/// ## 2. Variable Capture with Auto-Cloning (Direct Registration)
/// ```rust
/// use gears_app::prelude::*;
/// use std::sync::mpsc;
///
/// let mut app = GearsApp::default();
/// let sender = mpsc::channel().0;
///
/// async_system!(app, "update", (sender), |world, dt| {
///     // Automatically registers the system with the app
///     let _ = sender.send(dt);
///     Ok(())
/// });
/// ```
///
/// ## 3. Simple Closure (Returns System)
/// ```rust
/// use gears_app::prelude::*;
///
/// let _system = async_system!("physics_update", |world, dt| {
///     // Simple async system without external captures
///     println!("Delta time: {:?}", dt);
///     Ok(())
/// });
/// ```
///
/// ## 4. Simple Closure (Direct Registration)
/// ```rust
/// use gears_app::prelude::*;
///
/// let mut app = GearsApp::default();
///
/// async_system!(app, "physics_update", |world, dt| {
///     // Directly registers with app
///     println!("Delta time: {:?}", dt);
///     Ok(())
/// });
/// ```
///
/// ## 5. Move Closure (Returns System)
/// ```rust
/// use gears_app::prelude::*;
///
/// let entity_id = Entity::new(123);
///
/// let _system = async_system!("entity_update", move |world, dt| {
///     // Moves captured variables into the closure
///     if let Some(_pos) = world.get_component::<Pos3>(entity_id) {
///         // Update entity logic
///     }
///     Ok(())
/// });
/// ```
///
/// ## 6. Move Closure (Direct Registration)
/// ```rust
/// use gears_app::prelude::*;
///
/// let mut app = GearsApp::default();
/// let entity_id = Entity::new(123);
///
/// async_system!(app, "entity_update", move |world, dt| {
///     // Moves and registers directly
///     if let Some(_pos) = world.get_component::<Pos3>(entity_id) {
///         // Update entity logic
///     }
///     Ok(())
/// });
/// ```
///
/// ## 7. Custom Block (Returns System)
/// ```rust
/// use gears_app::prelude::*;
/// use std::sync::{Arc, Mutex};
/// use std::pin::Pin;
/// use std::future::Future;
///
/// let _system = async_system!("custom_system", {
///     let shared_data = Arc::new(Mutex::new(Vec::new()));
///     move |world, dt| {
///         let data = shared_data.clone();
///         Box::pin(async move {
///             // Custom async logic with manual control
///             data.lock().unwrap().push(dt.as_secs_f32());
///             Ok(())
///         })
///     }
/// });
/// ```
///
/// ## 8. Custom Block (Direct Registration)
/// ```rust
/// use gears_app::prelude::*;
/// use std::sync::{Arc, Mutex};
/// use std::pin::Pin;
/// use std::future::Future;
///
/// let mut app = GearsApp::default();
///
/// async_system!(app, "custom_system", {
///     let shared_data = Arc::new(Mutex::new(Vec::new()));
///     move |world, dt| {
///         let data = shared_data.clone();
///         Box::pin(async move {
///             data.lock().unwrap().push(dt.as_secs_f32());
///             Ok(())
///         })
///     }
/// });
/// ```
///
/// # Important Notes
///
/// - For non-`Copy` types (like `mpsc::Sender`), use pattern 1 or 2 for automatic cloning
/// - The `move` variants (5, 6) transfer ownership of captured variables
/// - Custom blocks (7, 8) provide full control but require manual async setup
/// - All systems receive `Arc<World>` and `dt` as individual parameters
/// - Systems should return `SystemResult<()>` (which is `Result<(), SystemError>`)
///
#[macro_export]
macro_rules! async_system {
    ($name:expr, ($($var:ident),* $(,)?), |$world:ident, $dt:ident| $body:block) => {
        $crate::systems::system($name, {
            move |$world, $dt| {
                std::boxed::Box::pin({
                    $(let $var = $var.clone();)*
                    async move  {
                        $body
                    }
                })
            }
        })
    };
    ($app:expr, $name:expr, ($($var:ident),* $(,)?), |$world:ident, $dt:ident| $body:block) => {
        {
            let system = $crate::systems::system($name, {
                move |$world, $dt| {
                    std::boxed::Box::pin({
                        $(let $var = $var.clone();)*
                        async move  {
                            $body
                        }
                    })
                }
            });
            $app.add_async_system(system);
        }
    };
    ($name:expr, |$world:ident, $dt:ident| $body:block) => {
        $crate::systems::system($name, |$world, $dt| std::boxed::Box::pin(async move $body))
    };
    ($app:expr, $name:expr, |$world:ident, $dt:ident| $body:block) => {
        {
            let system = $crate::systems::system($name, |$world, $dt| std::boxed::Box::pin(async move $body));
            $app.add_async_system(system);
        }
    };
    ($name:expr, move |$world:ident, $dt:ident| $body:block) => {
        $crate::systems::system($name, move |$world, $dt| std::boxed::Box::pin(async move $body))
    };
    ($app:expr, $name:expr, move |$world:ident, $dt:ident| $body:block) => {
        {
            let system = $crate::systems::system($name, move |$world, $dt| std::boxed::Box::pin(async move $body));
            $app.add_async_system(system);
        }
    };
    ($name:expr, $capture_block:block) => {
        $crate::systems::system($name, $capture_block)
    };
    ($app:expr, $name:expr, $capture_block:block) => {
        {
            let system = $crate::systems::system($name, $capture_block);
            $app.add_async_system(system);
        }
    };
}
