use wgpu::util::DeviceExt;

/// Pipeline for rendering a procedural crosshair overlay.
pub struct CrosshairPipeline {
    /// The render pipeline for the crosshair.
    pipeline: wgpu::RenderPipeline,
    /// The bind group for passing screen dimensions.
    bind_group: wgpu::BindGroup,
    /// The bind group layout.
    layout: wgpu::BindGroupLayout,
    /// Uniform buffer for screen dimensions.
    uniform_buffer: wgpu::Buffer,
    /// Whether the crosshair is visible.
    visible: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CrosshairUniforms {
    screen_width: f32,
    screen_height: f32,
    gap: f32,
    length: f32,
    thickness: f32,
    _padding0: [u32; 3],
    color: [f32; 4],
}

impl CrosshairPipeline {
    /// Creates a new crosshair pipeline.
    ///
    /// # Arguments
    ///
    /// * `device` - The GPU device for resource creation.
    /// * `config` - The surface configuration for format information.
    ///
    /// # Returns
    ///
    /// A new [`CrosshairPipeline`] instance.
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let uniforms = CrosshairUniforms {
            screen_width: config.width as f32,
            screen_height: config.height as f32,
            gap: 5.0,
            length: 15.0,
            thickness: 2.0,
            _padding0: [0; 3],
            color: [1.0, 1.0, 1.0, 1.0], // White
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Crosshair::uniform_buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Crosshair::layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Crosshair::bind_group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let shader = wgpu::include_wgsl!("../../../shaders/crosshair.wgsl");
        let shader_module = device.create_shader_module(shader);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Crosshair::pipeline_layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        // Create pipeline with alpha blending
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Crosshair::pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format.add_srgb_suffix(),
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
            layout,
            uniform_buffer,
            visible: true,
        }
    }

    /// Updates the uniform buffer with current settings.
    ///
    /// # Arguments
    ///
    /// * `queue` - The GPU queue for buffer updates.
    /// * `width` - Screen width.
    /// * `height` - Screen height.
    pub fn update_uniforms(
        &self,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        gap: f32,
        length: f32,
        thickness: f32,
        color: [f32; 4],
    ) {
        let uniforms = CrosshairUniforms {
            screen_width: width as f32,
            screen_height: height as f32,
            gap,
            length,
            thickness,
            _padding0: [0; 3],
            color,
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Resizes the crosshair pipeline (updates screen dimensions).
    ///
    /// # Arguments
    ///
    /// * `queue` - The GPU queue for buffer updates.
    /// * `width` - The new width in pixels.
    /// * `height` - The new height in pixels.
    pub fn resize(&self, queue: &wgpu::Queue, width: u32, height: u32) {
        self.update_uniforms(queue, width, height, 5.0, 15.0, 2.0, [1.0, 1.0, 1.0, 1.0]);
    }

    /// Sets the visibility of the crosshair.
    ///
    /// # Arguments
    ///
    /// * `visible` - Whether the crosshair should be visible.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Gets the visibility of the crosshair.
    ///
    /// # Returns
    ///
    /// `true` if the crosshair is visible, `false` otherwise.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Toggles the visibility of the crosshair.
    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }

    /// Renders the crosshair overlay to the output.
    ///
    /// # Arguments
    ///
    /// * `encoder` - The command encoder for rendering.
    /// * `output` - The output texture view to render to.
    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        if !self.visible {
            return;
        }

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Crosshair::render"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                ops: wgpu::Operations {
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
