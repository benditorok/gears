pub mod camera;
pub mod instance;
pub mod light;
pub mod model;
pub mod resources;
pub mod state;
pub mod texture;

use gears_ecs::Component;
use std::ops::Deref;

/// Wrapper for wgpu::Buffer to implement Component
#[derive(Debug)]
pub struct BufferComponent(pub wgpu::Buffer);

impl Deref for BufferComponent {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Component for BufferComponent {}
