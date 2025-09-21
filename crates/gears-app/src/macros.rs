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
/// let sender = mpsc::channel().0;
/// let counter = Arc::new(Mutex::new(0));
///
/// let system = async_system!("update", (sender, counter), |sa| {
///     // Variables are automatically cloned before the async block
///     sender.send(sa.dt).unwrap();
///     *counter.lock().unwrap() += 1;
///     Ok(())
/// });
/// ```
///
/// ## 2. Variable Capture with Auto-Cloning (Direct Registration)
/// ```rust
/// let mut app = GearsApp::default();
/// let sender = mpsc::channel().0;
///
/// async_system!(app, "update", (sender), |sa| {
///     // Automatically registers the system with the app
///     sender.send(sa.dt).unwrap();
///     Ok(())
/// });
/// ```
///
/// ## 3. Simple Closure (Returns System)
/// ```rust
/// let system = async_system!("physics_update", |sa| {
///     // Simple async system without external captures
///     println!("Delta time: {:?}", sa.dt);
///     Ok(())
/// });
/// ```
///
/// ## 4. Simple Closure (Direct Registration)
/// ```rust
/// let mut app = GearsApp::default();
///
/// async_system!(app, "physics_update", |sa| {
///     // Directly registers with app
///     println!("Delta time: {:?}", sa.dt);
///     Ok(())
/// });
/// ```
///
/// ## 5. Move Closure (Returns System)
/// ```rust
/// let entity_id = Entity::new(123);
///
/// let system = async_system!("entity_update", move |sa| {
///     // Moves captured variables into the closure
///     if let Some(pos) = sa.world.get_component::<Pos3>(entity_id) {
///         // Update entity logic
///     }
///     Ok(())
/// });
/// ```
///
/// ## 6. Move Closure (Direct Registration)
/// ```rust
/// let mut app = GearsApp::default();
/// let entity_id = Entity::new(123);
///
/// async_system!(app, "entity_update", move |sa| {
///     // Moves and registers directly
///     if let Some(pos) = sa.world.get_component::<Pos3>(entity_id) {
///         // Update entity logic
///     }
///     Ok(())
/// });
/// ```
///
/// ## 7. Custom Block (Returns System)
/// ```rust
/// let system = async_system!("custom_system", {
///     let shared_data = Arc::new(Mutex::new(Vec::new()));
///     move |sa| {
///         let data = shared_data.clone();
///         Box::pin(async move {
///             // Custom async logic with manual control
///             data.lock().unwrap().push(sa.dt.as_secs_f32());
///             Ok(())
///         })
///     }
/// });
/// ```
///
/// ## 8. Custom Block (Direct Registration)
/// ```rust
/// let mut app = GearsApp::default();
///
/// async_system!(app, "custom_system", {
///     let shared_data = Arc::new(Mutex::new(Vec::new()));
///     move |sa| {
///         let data = shared_data.clone();
///         Box::pin(async move {
///             data.lock().unwrap().push(sa.dt.as_secs_f32());
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
/// - All systems receive `SystemAccessors` containing `world` and `dt`
/// - Systems should return `SystemResult<()>` (which is `Result<(), SystemError>`)
///
#[macro_export]
macro_rules! async_system {
    ($name:expr, ($($var:ident),* $(,)?), |$sa:ident| $body:block) => {
        $crate::systems::system($name, {
            move |$sa| {
                std::boxed::Box::pin({
                    $(let $var = $var.clone();)*
                    async move  {
                        $body
                    }
                })
            }
        })
    };
    ($app:expr, $name:expr, ($($var:ident),* $(,)?), |$sa:ident| $body:block) => {
        {
            let system = $crate::systems::system($name, {
                move |$sa| {
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
    ($name:expr, |$sa:ident| $body:block) => {
        $crate::systems::system($name, |$sa| std::boxed::Box::pin(async move $body))
    };
    ($app:expr, $name:expr, |$sa:ident| $body:block) => {
        {
            let system = $crate::systems::system($name, |$sa| std::boxed::Box::pin(async move $body));
            $app.add_async_system(system);
        }
    };
    ($name:expr, move |$sa:ident| $body:block) => {
        $crate::systems::system($name, move |$sa| std::boxed::Box::pin(async move $body))
    };
    ($app:expr, $name:expr, move |$sa:ident| $body:block) => {
        {
            let system = $crate::systems::system($name, move |$sa| std::boxed::Box::pin(async move $body));
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
