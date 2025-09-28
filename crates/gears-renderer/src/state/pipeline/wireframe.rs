use crate::{
    instance,
    model::{self, Vertex},
    state::{pipeline::hdr::HdrPipeline, resources},
    texture,
};

/// The pipeline used to render wireframes
pub struct WireframePipeline {
    pipeline: wgpu::RenderPipeline,
    #[allow(unused)]
    pipeline_layout: wgpu::PipelineLayout,
}

impl WireframePipeline {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera_layout: &wgpu::BindGroupLayout,
        hdr_pipeline: &HdrPipeline,
    ) -> Self {
        let width = config.width;
        let height = config.height;

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

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}
