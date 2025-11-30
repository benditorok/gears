//! Core functionalities for the gears engine.

#![forbid(unsafe_code)]

pub mod config;

/// Type alias for a duration which can be used to represent time intervals.
pub type Dt = std::time::Duration;

/// Safe fraction of pi over 2.
pub const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;

/// A matrix to scale and translate from OpenGL to WebGPU coordinates.
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);
