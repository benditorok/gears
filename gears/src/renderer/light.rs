use std::default;

use cgmath::{Point3, Vector3};
use wgpu::util::DeviceExt;

pub(crate) const NUM_MAX_LIGHTS: u32 = 20;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightUniform {
    pub position: [f32; 3],
    /// Padding for correct alignment, **do not read this field**
    pub _padding: u32,
    pub color: [f32; 3],
    /// Padding for correct alignment, **do not read this field**
    pub _padding2: u32,
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            _padding: 0,
            color: [0.0; 3],
            _padding2: 0,
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

pub(crate) trait ToLightUniform {
    fn to_light_uniform(&self) -> LightUniform;
}

pub(crate) struct Light {
    pub position: Point3<f32>,
    pub color: Vector3<f32>,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl ToLightUniform for Light {
    fn to_light_uniform(&self) -> LightUniform {
        LightUniform {
            position: [self.position.x, self.position.y, self.position.z],
            _padding: 0,
            color: [self.color.x, self.color.y, self.color.z],
            _padding2: 0,
        }
    }
}

// impl Light {
//     pub fn new(
//         device: &wgpu::Device,
//         position: Point3<f32>,
//         color: Vector3<f32>,
//         bind_group_layout: &wgpu::BindGroupLayout,
//     ) -> Self {
//         let light_uniform = LightUniform {
//             position: [position.x, position.y, position.z],
//             _padding: 0,
//             color: [color.x, color.y, color.z],
//             _padding2: 0,
//         };

//         let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Light Buffer"),
//             contents: bytemuck::cast_slice(&[light_uniform]),
//             usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
//         });

//         let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
//             layout: bind_group_layout,
//             entries: &[wgpu::BindGroupEntry {
//                 binding: 0,
//                 resource: buffer.as_entire_binding(),
//             }],
//             label: Some("Light Bind Group"),
//         });

//         Self {
//             position,
//             color,
//             buffer,
//             bind_group,
//         }
//     }
// }
