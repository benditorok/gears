pub mod core;
pub mod ecs;
pub mod gui;
pub mod macros;
pub mod prelude;
pub mod renderer;

use std::f32::consts::FRAC_PI_2;

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;
