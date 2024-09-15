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
