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

pub enum ModelDataInstance {
    Single(instance::Instance),
    Multiple(Vec<instance::Instance>),
    None,
}

pub struct GearsModelData<'a> {
    pub file_path: &'a str,
    pub(crate) model: Option<model::Model>,
    pub(crate) instance: Option<instance::Instance>,
    pub(crate) instance_buffer: Option<wgpu::Buffer>,
}

impl<'a> GearsModelData<'a> {
    pub fn new(file_path: &'a str) -> Self {
        Self {
            file_path,
            model: None,
            instance: None,
            instance_buffer: None,
        }
    }
}

pub struct ModelData<'a> {
    file_path: &'a str,
    instances: ModelDataInstance,
}

impl<'a> ModelData<'a> {
    pub fn new(file_path: &'a str) -> Self {
        Self {
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
