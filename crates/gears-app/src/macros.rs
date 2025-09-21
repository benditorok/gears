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
