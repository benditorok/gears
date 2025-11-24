use crate::{
    camera, instance, light,
    model::{self, Vertex},
    state::{pipeline::hdr::HdrPipeline, resources},
    texture,
};
use gears_ecs::components;
use wgpu::util::DeviceExt;

/// The base pipeline for rendering.
pub struct BasePipeline {
    /// The render pipeline.
    pipeline: wgpu::RenderPipeline,
    /// The bind group for the lights.
    light_bind_group: wgpu::BindGroup,
    /// The bind group for the camera.
    camera_bind_group: wgpu::BindGroup,
    /// The depth texture.
    texture: texture::Texture,
    /// The width of the texture.
    width: u32,
    /// The height of the texture.
    height: u32,
    /// The texture format.
    format: wgpu::TextureFormat,
    /// The bind group layout for the texture.
    texture_layout: wgpu::BindGroupLayout,
    /// The bind group layout for the camera.
    camera_layout: wgpu::BindGroupLayout,
    /// The bind group layout for the lights.
    #[allow(unused)]
    light_layout: wgpu::BindGroupLayout,
    /// The pipeline layout.
    #[allow(unused)]
    pipeline_layout: wgpu::PipelineLayout,
    /// The camera projection.
    camera_projection: camera::Projection,
    /// The camera uniform.
    camera_uniform: camera::CameraUniform,
    /// The camera buffer.
    camera_buffer: wgpu::Buffer,
    /// The light buffer.
    light_buffer: wgpu::Buffer,
}

impl BasePipeline {
    /// Creates a new renderer pipeline instance.
    ///
    /// # Arguments
    ///
    /// * `device` - The wgpu device to create the pipeline on.
    /// * `config` - The surface configuration.
    /// * `texture_format` - The texture format to use.
    ///   Use the format that the HDR pipeline uses.
    ///
    /// # Returns
    ///
    /// A new [`BasePipeline`] instance.
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let width = config.width;
        let height = config.height;

        let format = wgpu::TextureFormat::Rgba16Float;

        let depth_texture = texture::Texture::create_depth_texture(
            &device,
            config.width,
            config.height,
            Some("Base::depth_texture"),
        );

        // Bind group layouts
        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Base::texture_layout"),
            entries: &[
                // Diffuse map
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Normal map
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let light_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Base::light_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Base::camera_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Initialize he camera
        let camera_projection =
            camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_uniform = camera::CameraUniform::new();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Base:camera_buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        // Initialize the camera and lights
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Base::light_buffer"),
            contents: &[0; std::mem::size_of::<light::LightData>()], // ! Initialize the buffer for the maximum number of lights
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind groups
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Base:camera_bind_group"),
        });
        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: Some("Base::light_bind_group"),
        });

        // Prepare the shader and pipeline layout
        let shader = wgpu::ShaderModuleDescriptor {
            label: Some("Base::shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../shaders/shader.wgsl").into()),
        };
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Base::pipeline_layout"),
            bind_group_layouts: &[&texture_layout, &camera_layout, &light_layout],
            push_constant_ranges: &[],
        });

        // Construct the pipeline
        let pipeline = resources::create_render_pipeline(
            &device,
            &pipeline_layout,
            texture_format,
            Some(texture::Texture::DEPTH_FORMAT),
            &[model::ModelVertex::desc(), instance::InstanceRaw::desc()],
            wgpu::PrimitiveTopology::TriangleList,
            shader,
        );

        Self {
            pipeline,
            // texture_bind_group,
            light_bind_group,
            camera_bind_group,
            texture: depth_texture,
            width,
            height,
            format,
            texture_layout,
            camera_layout,
            light_layout,
            pipeline_layout,
            camera_projection,
            camera_buffer,
            camera_uniform,
            light_buffer,
        }
    }

    /// Resize the depth texture.
    ///
    /// # Arguments
    ///
    /// * `device` - The wgpu device to create the texture on.
    /// * `width` - The new width of the texture.
    /// * `height` - The new height of the texture.
    pub(crate) fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.texture = texture::Texture::create_depth_texture(
            &device,
            width,
            height,
            Some("Base::depth_texture"),
        );

        self.width = width;
        self.height = height;
    }

    /// Exposes the texture view.
    ///
    /// # Returns
    ///
    /// A reference to the texture view.
    #[allow(unused)]
    pub(crate) fn texture_view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    /// The format of the texture.
    ///
    /// # Returns
    ///
    /// The texture format.
    #[allow(unused)]
    pub(crate) fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// Begins a render pass for the internal texture.
    ///
    /// # Arguments
    ///
    /// * `encoder` - The command encoder to begin the render pass on.
    /// * `output` - The output texture view to render to.
    ///
    /// # Returns
    ///
    /// The render pass which can be used to issue draw calls.
    pub(crate) fn begin<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
        output: &wgpu::TextureView,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Base::render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.53,
                        g: 0.81,
                        b: 0.92,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        })
    }

    /// Exposes the render pipeline.
    ///
    /// # Returns
    ///
    /// A reference to the render pipeline.
    pub(crate) fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    /// Exposes the camera bind group.
    ///
    /// # Returns
    ///
    /// A reference to the camera bind group.
    pub(crate) fn camera_bind_group(&self) -> &wgpu::BindGroup {
        &self.camera_bind_group
    }

    /// Exposes the light bind group.
    ///
    /// # Returns
    ///
    /// A reference to the light bind group.
    pub fn light_bind_group(&self) -> &wgpu::BindGroup {
        &self.light_bind_group
    }

    /// Exposes the texture bind group layout.
    ///
    /// # Returns
    ///
    /// A reference to the texture bind group layout.
    pub fn texture_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_layout
    }

    /// Exposes the camera bind group layout.
    ///
    /// # Returns
    ///
    /// A reference to the camera bind group layout.
    pub fn camera_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_layout
    }

    /// Exposes the light bind group layout.
    ///
    /// # Returns
    ///
    /// A reference to the light bind group layout.
    pub(crate) fn camera_projection_mut(&mut self) -> &mut camera::Projection {
        &mut self.camera_projection
    }

    /// Exposes the camera uniform.
    ///
    /// # Returns
    ///
    /// A reference to the camera uniform.
    pub(crate) fn camera_uniform(&self) -> &camera::CameraUniform {
        &self.camera_uniform
    }

    /// Exposes the camera buffer.
    ///
    /// # Returns
    ///
    /// A reference to the camera buffer.
    pub(crate) fn camera_buffer(&self) -> &wgpu::Buffer {
        &self.camera_buffer
    }

    /// Exposes the light buffer.
    ///
    /// # Returns
    ///
    /// A reference to the light buffer.
    pub fn light_buffer(&self) -> &wgpu::Buffer {
        &self.light_buffer
    }

    /// Updates the camera view projection matrix.
    ///
    /// # Arguments
    ///
    /// * `pos3` - The position component of the camera.
    /// * `controller` - The view controller component of the camera.
    pub(crate) fn update_camera_view_proj(
        &mut self,
        pos3: &components::transforms::Pos3,
        controller: &components::controllers::ViewController,
    ) {
        self.camera_uniform
            .update_view_proj(pos3, controller, &self.camera_projection);
    }
}
