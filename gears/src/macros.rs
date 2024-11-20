/// A macro to create a new entity and add multiple components when using the EntityBuilder trait.
#[macro_export]
macro_rules! new_entity {
    // Single component case
    ($app:expr, single: $component:expr) => {{
        let mut entity_builder = $app.new_entity();
        entity_builder = entity_builder.add_component($component);
        entity_builder.build()
    }};
    // Array/Vec of components case - now handles boxed components
    ($app:expr, array: $components:expr) => {{
        let mut entity_builder = $app.new_entity();
        for component in $components {
            entity_builder = entity_builder.add_component(component);
        }
        entity_builder.build()
    }};
    // Multiple components case
    ($app:expr, $($component:expr),* $(,)?) => {{
        let mut entity_builder = $app.new_entity();
        $(
            entity_builder = entity_builder.add_component($component);
        )*
        entity_builder.build()
    }};
}
