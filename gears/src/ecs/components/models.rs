use crate::ecs::traits::Component;
use gears_macro::Component;

/// A component that stores the source of a model.
#[derive(Component, Debug, Copy, Clone)]
pub enum ModelSource {
    Obj(&'static str),
    Gltf(&'static str),
}
