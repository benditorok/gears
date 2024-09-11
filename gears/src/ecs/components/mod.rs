use crate::renderer::{instance, model, state};

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Position {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Renderable;

pub struct GearsModelData<'a> {
    pub file_path: &'a str,
}

impl<'a> GearsModelData<'a> {
    pub fn new(file_path: &'a str) -> Self {
        Self { file_path }
    }
}
