use crate::renderer::{instance, model};

#[derive(Clone, Copy, Debug)]
pub struct Pos3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Pos3 {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl Pos3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

// TODO create Renderable enum with model, pos, rotation, ...
pub enum Renderable {
    Spatial { model: model::Model, position: Pos3 },
    Sprite,
}

pub struct GearsModelData<'a> {
    pub file_path: &'a str,
}

impl<'a> GearsModelData<'a> {
    pub fn new(file_path: &'a str) -> Self {
        Self { file_path }
    }
}
