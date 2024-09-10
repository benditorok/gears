use wgpu::core::instance;

use crate::renderer::state;

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

pub enum ModelDataInstance {
    Single(instance::Instance),
    Multiple(Vec<instance::Instance>),
    None,
}

pub struct ModelData<'a> {
    entity: &'a usize,
    file_path: &'a str,
    instances: ModelDataInstance,
}

impl<'a> ModelData<'a> {
    pub fn new(entity: &'a usize, file_path: &'a str) -> Self {
        Self {
            entity,
            file_path,
            instances: ModelDataInstance::None,
        }
    }

    pub fn set_instance(&mut self, instance: instance::Instance) {
        self.instances = ModelDataInstance::Single(instance);
    }

    pub fn set_instances(&mut self, instances: Vec<instance::Instance>) {
        self.instances = ModelDataInstance::Multiple(instances);
    }
}
