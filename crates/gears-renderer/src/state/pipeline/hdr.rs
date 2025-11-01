use crate::{state::resources, texture};
use wgpu::Operations;

/// Owns the render texture and controls tonemapping.
pub struct HdrPipeline {
    /// The render pipeline for HDR processing.
    pipeline: wgpu::RenderPipeline,
    /// The bind group for HDR texture access.
    bind_group: wgpu::BindGroup,
    /// The HDR render target texture.
    texture: texture::Texture,
    /// The width of the HDR texture.
    width: u32,
    /// The height of the HDR texture.
    height: u32,
    /// The texture format used for HDR.
    format: wgpu::TextureFormat,
    /// The bind group layout for HDR textures.
    layout: wgpu::BindGroupLayout,
}

impl HdrPipeline {
    /// Creates a new HDR pipeline.
    ///
    /// # Arguments
    ///
    /// * `device` - The GPU device for resource creation.
    /// * `config` - The surface configuration for sizing.
    ///
    /// # Returns
    ///
    /// A new [`HdrPipeline`] instance.
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let width = config.width;
        let height = config.height;

        let format = wgpu::TextureFormat::Rgba16Float;

        let texture = texture::Texture::create_2d_texture(
            device,
            width,
            height,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::FilterMode::Nearest,
            Some("Hdr::texture"),
        );

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Hdr::layout"),
            entries: &[
                // This is the HDR texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Hdr::bind_group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        let shader = wgpu::include_wgsl!("../../../shaders/hdr.wgsl");
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let pipeline = resources::create_render_pipeline(
            device,
            &pipeline_layout,
            config.format.add_srgb_suffix(),
            None,
            // Vertex data is generated in the shader
            &[],
            wgpu::PrimitiveTopology::TriangleList,
            shader,
        );

        Self {
            pipeline,
            bind_group,
            layout,
            texture,
            width,
            height,
            format,
        }
    }

    /// Resizes the HDR texture to new dimensions.
    ///
    /// # Arguments
    ///
    /// * `device` - The GPU device for texture creation.
    /// * `width` - The new width in pixels.
    /// * `height` - The new height in pixels.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.texture = texture::Texture::create_2d_texture(
            device,
            width,
            height,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::FilterMode::Nearest,
            Some("Hdr::texture"),
        );
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Hdr::bind_group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.texture.sampler),
                },
            ],
        });
        self.width = width;
        self.height = height;
    }

    /// Exposes the HDR texture view.
    ///
    /// # Returns
    ///
    /// A reference to the HDR texture view.
    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    /// The format of the HDR texture.
    ///
    /// # Returns
    ///
    /// The texture format used for HDR rendering.
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// Renders the internal HDR texture to the output with tonemapping.
    ///
    /// # Arguments
    ///
    /// * `encoder` - The command encoder for rendering.
    /// * `output` - The output texture view to render to.
    pub fn process(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Hdr::process"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
