use crate::ecs::Component;
use gears_macro::Component;

/// A component that stores the light type.
#[derive(Component, Debug, Copy, Clone)]
pub enum Light {
    Point {
        radius: f32,
        intensity: f32,
    },
    PointColoured {
        radius: f32,
        color: [f32; 3],
        intensity: f32,
    },
    Ambient {
        intensity: f32,
    },
    AmbientColoured {
        color: [f32; 3],
        intensity: f32,
    },
    Directional {
        direction: [f32; 3],
        intensity: f32,
    },
    DirectionalColoured {
        direction: [f32; 3],
        color: [f32; 3],
        intensity: f32,
    },
}

// TODO lightuniforms should take into impls instead of defined types? so that the user can create new types of lights
