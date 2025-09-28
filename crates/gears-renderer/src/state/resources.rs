/// Create a new render pipeline.
/// This function is used to create a new render pipeline for the given device.
///
/// # Arguments
///
/// * `device` - The wgpu device.
/// * `layout` - The pipeline layout.
/// * `color_format` - The texture format for the output color.
/// * `depth_format` - The texture format for the output depth.
/// * `vertex_layouts` - The vertex buffer layouts.
/// * `shader` - The shader module descriptor.
///
/// # Returns
///
// /// A new render pipeline.
// pub(crate) fn create_render_pipeline(
//     device: &wgpu::Device,
//     layout: &wgpu::PipelineLayout,
//     color_format: wgpu::TextureFormat,
//     depth_format: Option<wgpu::TextureFormat>,
//     vertex_layouts: &[wgpu::VertexBufferLayout],
//     topology: wgpu::PrimitiveTopology,
//     shader: wgpu::ShaderModuleDescriptor,
//     pipeline_label: &'static str,
// ) -> wgpu::RenderPipeline {
//     let shader = device.create_shader_module(shader);

//     device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//         label: Some(pipeline_label),
//         layout: Some(layout),
//         vertex: wgpu::VertexState {
//             module: &shader,
//             entry_point: Some("vs_main"),
//             buffers: vertex_layouts,
//             compilation_options: wgpu::PipelineCompilationOptions {
//                 ..Default::default()
//             },
//         },
//         fragment: Some(wgpu::FragmentState {
//             module: &shader,
//             entry_point: Some("fs_main"),
//             targets: &[Some(wgpu::ColorTargetState {
//                 format: color_format,
//                 blend: Some(wgpu::BlendState::REPLACE),
//                 write_mask: wgpu::ColorWrites::ALL,
//             })],
//             compilation_options: wgpu::PipelineCompilationOptions {
//                 ..Default::default()
//             },
//         }),
//         primitive: wgpu::PrimitiveState {
//             topology,
//             strip_index_format: None,
//             front_face: wgpu::FrontFace::Ccw,
//             cull_mode: Some(wgpu::Face::Back),
//             polygon_mode: wgpu::PolygonMode::Fill,
//             unclipped_depth: false,
//             conservative: false,
//         },
//         depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
//             format,
//             depth_write_enabled: true,
//             depth_compare: wgpu::CompareFunction::Less,
//             stencil: wgpu::StencilState::default(),
//             bias: wgpu::DepthBiasState::default(),
//         }),
//         multisample: wgpu::MultisampleState {
//             count: 1,
//             mask: !0,
//             alpha_to_coverage_enabled: false,
//         },
//         multiview: None,
//         cache: None,
//     })
// }

pub(crate) fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    topology: wgpu::PrimitiveTopology, // NEW!
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let label = format!("{:?}", shader.label);
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: vertex_layouts,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology, // NEW!
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual, // UDPATED!
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
        cache: None,
    })
}

pub(crate) fn create_wireframe_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let label = format!("{:?}", shader.label);
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: vertex_layouts,
            compilation_options: wgpu::PipelineCompilationOptions {
                ..Default::default()
            },
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions {
                ..Default::default()
            },
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}
