use crate::{
    instance,
    model::{self, Vertex},
    state::{pipeline::hdr::HdrPipeline, resources},
    texture,
};

/// The pipeline used to render wireframes for collision boxes.
pub struct WireframePipeline {
    /// The render pipeline for wireframe rendering.
    pipeline: wgpu::RenderPipeline,
    /// The pipeline layout for bind groups.
    #[allow(unused)]
    pipeline_layout: wgpu::PipelineLayout,
}

impl WireframePipeline {
    /// Creates a new wireframe rendering pipeline.
    ///
    /// # Arguments
    ///
    /// * `device` - The GPU device for pipeline creation.
    /// * `_config` - The surface configuration (unused).
    /// * `camera_layout` - The bind group layout for camera data.
    /// * `hdr_pipeline` - The HDR pipeline to match texture format.
    ///
    /// # Returns
    ///
    /// A new [`WireframePipeline`] instance.
    pub fn new(
        device: &wgpu::Device,
        _config: &wgpu::SurfaceConfiguration,
        camera_layout: &wgpu::BindGroupLayout,
        hdr_pipeline: &HdrPipeline,
    ) -> Self {
        // Prepare the shader and pipeline layout
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Base::wireframe_shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../shaders/wireframe.wgsl").into(),
            ),
        };
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Base::pipeline_layout"),
            bind_group_layouts: &[&camera_layout],
            push_constant_ranges: &[],
        });

        // Construct the pipeline
        let pipeline = resources::create_wireframe_render_pipeline(
            &device,
            &pipeline_layout,
            hdr_pipeline.format(), // Use the format that the HDR pipeline uses
            Some(texture::Texture::DEPTH_FORMAT),
            &[model::ColliderVertex::desc(), instance::InstanceRaw::desc()],
            shader,
        );

        Self {
            pipeline,
            pipeline_layout,
        }
    }

    /// Exposes the wireframe render pipeline.
    ///
    /// # Returns
    ///
    /// A reference to the render pipeline.
    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}
