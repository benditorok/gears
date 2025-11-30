use crate::Component;
use gears_macro::Component;

/// A component that stores the source of a model.
#[derive(Component, Debug, Copy, Clone)]
pub enum ModelSource {
    /// A Wavefront .obj file.
    ///
    /// # Arguments
    ///
    /// * `&'static str` - The path to the .obj file relative to the resource directory.
    Obj(&'static str),
    /// A glTF file.
    ///
    /// # Arguments
    ///
    /// * `&'static str` - The path to the glTF file relative to the resource directory.
    Gltf(&'static str),
}
