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
