use gears_ecs::{
    Component,
    components::{lights::Light, transforms::Pos3},
};
use gears_macro::Component;

pub const NUM_MAX_LIGHTS: u32 = 20;

#[repr(u32)]
pub(crate) enum LightType {
    Point = 0,
    Ambient = 1,
    Directional = 2,
}

#[repr(C)]
#[derive(Component, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub light_type: u32,
    pub color: [f32; 3],
    pub radius: f32,
    pub direction: [f32; 3],
    pub intensity: f32,
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            light_type: LightType::Ambient as u32,
            color: [1.0; 3],
            radius: 0.0,
            direction: [0.0; 3],
            intensity: 0.1,
        }
    }
}

impl LightUniform {
    pub fn from_components(light: &Light, pos3: &Pos3) -> Self {
        match light {
            Light::Point { radius, intensity } => Self {
                position: [pos3.pos.x, pos3.pos.y, pos3.pos.z],
                light_type: LightType::Point as u32,
                color: [1.0, 1.0, 1.0],
                radius: *radius,
                direction: [0.0; 3],
                intensity: *intensity,
            },
            Light::PointColoured {
                radius,
                color,
                intensity,
            } => Self {
                position: [pos3.pos.x, pos3.pos.y, pos3.pos.z],
                light_type: LightType::Point as u32,
                color: *color,
                radius: *radius,
                direction: [0.0; 3],
                intensity: *intensity,
            },
            Light::Ambient { intensity } => Self {
                position: [pos3.pos.x, pos3.pos.y, pos3.pos.z],
                light_type: LightType::Ambient as u32,
                color: [1.0, 1.0, 1.0],
                radius: 0.0,
                direction: [0.0; 3],
                intensity: *intensity,
            },
            Light::AmbientColoured { color, intensity } => Self {
                position: [pos3.pos.x, pos3.pos.y, pos3.pos.z],
                light_type: LightType::Ambient as u32,
                color: *color,
                radius: 0.0,
                direction: [0.0; 3],
                intensity: *intensity,
            },
            Light::Directional {
                direction,
                intensity,
            } => Self {
                position: [pos3.pos.x, pos3.pos.y, pos3.pos.z],
                light_type: LightType::Directional as u32,
                color: [1.0, 1.0, 1.0],
                radius: 0.0,
                direction: *direction,
                intensity: *intensity,
            },
            Light::DirectionalColoured {
                direction,
                color,
                intensity,
            } => Self {
                position: [pos3.pos.x, pos3.pos.y, pos3.pos.z],
                light_type: LightType::Directional as u32,
                color: *color,
                radius: 0.0,
                direction: *direction,
                intensity: *intensity,
            },
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightData {
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
