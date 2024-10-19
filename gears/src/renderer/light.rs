use std::default;

use cgmath::{Point3, Vector3};
use wgpu::util::DeviceExt;

pub(crate) const NUM_MAX_LIGHTS: u32 = 20;

#[repr(u32)]
pub(crate) enum LightType {
    Point = 0,
    Ambient = 1,
    Directional = 2,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightUniform {
    pub position: [f32; 3],
    pub light_type: u32,
    pub color: [f32; 3],
    pub radius: f32,
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            light_type: LightType::Ambient as u32,
            color: [0.0; 3],
            radius: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightData {
    pub lights: [LightUniform; NUM_MAX_LIGHTS as usize],
    pub num_lights: u32,
    pub _padding: [u32; 3], // Padding to align to 16 bytes
}

impl Default for LightData {
    fn default() -> Self {
        Self {
            lights: [LightUniform::default(); NUM_MAX_LIGHTS as usize],
            num_lights: 0,
            _padding: [0; 3],
        }
    }
}
