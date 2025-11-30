use gears_ecs::{
    Component,
    components::{lights::Light, transforms::Pos3},
};
use gears_macro::Component;

/// Maximum number of lights supported in a scene.
pub const NUM_MAX_LIGHTS: u32 = 20;

/// The type of light for shader processing.
#[repr(u32)]
pub(crate) enum LightType {
    /// Point light that emits in all directions.
    Point = 0,
    /// Ambient light that affects everything equally.
    Ambient = 1,
    /// Directional light from a specific direction.
    Directional = 2,
}

/// Uniform data for a single light sent to the GPU.
#[repr(C)]
#[derive(Component, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    /// The position of the light in world space.
    pub position: [f32; 3],
    /// The type of light (point, ambient, or directional).
    pub light_type: u32,
    /// The RGB color of the light.
    pub color: [f32; 3],
    /// The radius for point lights.
    pub radius: f32,
    /// The direction for directional lights.
    pub direction: [f32; 3],
    /// The intensity/brightness of the light.
    pub intensity: f32,
}

impl Default for LightUniform {
    /// Creates a default light uniform with ambient light settings.
    ///
    /// # Returns
    ///
    /// The default [`LightUniform`] instance.
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
    /// Creates a light uniform from ECS components.
    ///
    /// # Arguments
    ///
    /// * `light` - The light component containing light properties.
    /// * `pos3` - The position component for the light.
    ///
    /// # Returns
    ///
    /// A new [`LightUniform`] instance populated from the components.
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

/// Container for all lights in a scene sent to the GPU.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightData {
    /// Array of all light uniforms in the scene.
    pub lights: [LightUniform; NUM_MAX_LIGHTS as usize],
    /// The actual number of active lights.
    pub num_lights: u32,
    /// Padding to align to 16 bytes.
    pub _padding: [u32; 3],
}

impl Default for LightData {
    /// Creates a default light data container with no active lights.
    ///
    /// # Returns
    ///
    /// The default [`LightData`] instance.
    fn default() -> Self {
        Self {
            lights: [LightUniform::default(); NUM_MAX_LIGHTS as usize],
            num_lights: 0,
            _padding: [0; 3],
        }
    }
}
