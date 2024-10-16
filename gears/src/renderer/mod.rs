pub mod camera;
pub mod instance;
pub mod light;
pub mod model;
pub mod resources;
pub mod texture;

use crate::ecs::{self, components};
use cgmath::prelude::*;
use cgmath::*;
use instant::Duration;
use log::info;
use model::{DrawLight, Vertex};
use std::f32::consts::FRAC_PI_2;
use std::iter;
use std::sync::{Arc, Mutex};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalPosition;
use winit::event::*;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
    window::{Window, WindowBuilder},
};

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

/// The main event loop of the application
///
/// # Returns
///
/// A future which can be awaited.
pub async fn run(world: Arc<Mutex<ecs::Manager>>) -> anyhow::Result<()> {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window, world).await;
    let mut last_render_time = instant::Instant::now();

    event_loop
        .run(move |event, ewlt| match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => if state.mouse_pressed {
                state.camera_controller.process_mouse(delta.0, delta.1)
            },
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() && !state.input(event) => {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => ewlt.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    },
                    // WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => {
                    //     *inner_size_writer = state.size.to_logical::<f64>(*scale_factor);
                    // }
                    WindowEvent::RedrawRequested => {
                        let now = instant::Instant::now();
                        let dt = now - last_render_time;
                        last_render_time = now;

                        info!(
                            "FPS: {:.0}, frame time: {} ms",
                            1.0 / &dt.as_secs_f32(),
                            &dt.as_millis()
                        );

                        futures::executor::block_on(state.update(dt));

                        match state.render() {
                            Ok(_) => {}
                            // Reconfigure the surface if it's lost or outdated
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                state.resize(state.size)
                            }
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => ewlt.exit(),
                            // We're ignoring timeouts
                            Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                        }
                    }
                    _ => {}
                };
            }
            Event::AboutToWait => {
                // RedrawRequested will only trigger once unless manually requested.
                state.window().request_redraw();
            }
            _ => {}
        })
        .unwrap();

    Ok(())
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    light_render_pipeline: wgpu::RenderPipeline,
    camera: camera::Camera,
    camera_projection: camera::Projection,
    camera_controller: camera::CameraController,
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    // TODO add a buffer for the models
    light_model: model::Model,
    light_uniform: light::LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_bind_group_layout: wgpu::BindGroupLayout,
    #[allow(dead_code)]
    depth_texture: texture::Texture,
    window: &'a Window,
    ecs: Arc<Mutex<ecs::Manager>>,
    mouse_pressed: bool,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window, ecs: Arc<Mutex<ecs::Manager>>) -> State<'a> {
        log::warn!("[State] Setup starting...");
        let size = window.inner_size();

        // The instance is a handle to the GPU. BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        log::warn!("[State] Device and Queue");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        log::warn!("[State] Surface");
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
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
                ],
                label: Some("texture_bind_group_layout"),
            });

        let light_uniform = light::LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: None,
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let light_model = resources::load_model(
            "res/models/sphere/sphere.obj",
            &device,
            &queue,
            &texture_bind_group_layout,
        )
        .await
        .unwrap();

        /* INITIALIZINS STATE COMPONENTS */
        let mut state_camera: camera::Camera =
            camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let mut state_camera_controller: camera::CameraController =
            camera::CameraController::new(0.5, 0.2);
        /* INITIALIZINS STATE COMPONENTS */

        // # START Load models and create instances #
        {
            log::warn!("Loading models and instances");

            let ecs_lock = ecs.lock().unwrap();

            /* Separate entitites by components */

            /* CAMERA COMPONENT */
            {
                let mut camera_entity = ecs_lock.get_entites_with_component::<components::Camera>();
                assert!(
                    camera_entity.len() <= 1,
                    "There should be only one camera entity"
                );
                let camera_entity = camera_entity.pop().unwrap();

                let camera_pos = ecs_lock
                    .get_component_from_entity::<components::Pos3>(camera_entity)
                    .expect("No position provided for the camera!");
                let camera = ecs_lock
                    .get_component_from_entity::<components::Camera>(camera_entity)
                    .expect("No camera component provided for the camera!");

                let camera_pos = camera_pos.read().unwrap();
                let camera = camera.read().unwrap();

                match *camera {
                    components::Camera::FPS {
                        look_at,
                        speed,
                        sensitivity,
                    } => {
                        let camera_pos_converted: cgmath::Point3<f32> = (*camera_pos).into();
                        let look_at_converted: cgmath::Point3<f32> = look_at.into();
                        state_camera =
                            camera::Camera::new_look_at(camera_pos_converted, look_at_converted);
                        state_camera_controller = camera::CameraController::new(speed, sensitivity);
                    }
                    components::Camera::Fixed { look_at } => {
                        let camera_pos_converted: cgmath::Point3<f32> = (*camera_pos).into();
                        let look_at_converted: cgmath::Point3<f32> = look_at.into();
                        state_camera =
                            camera::Camera::new_look_at(camera_pos_converted, look_at_converted);
                        state_camera_controller = camera::CameraController::new(0.0, 0.0);
                    }
                }
            }
            /* CAMERA COMPONENT */

            /* All in one go */
            for entity in ecs_lock.iter_entities() {
                if let Some(model) =
                    ecs_lock.get_component_from_entity::<components::ModelSource>(entity)
                {
                    log::warn!("Loading model: {:?}", model.read().unwrap().0);
                    let obj_model = resources::load_model(
                        model.read().unwrap().0,
                        &device,
                        &queue,
                        &texture_bind_group_layout,
                    )
                    .await
                    .unwrap();

                    ecs_lock.add_component_to_entity(entity, obj_model);

                    if let Some(position) =
                        ecs_lock.get_component_from_entity::<components::Pos3>(entity)
                    {
                        // log position, with the models name
                        log::warn!(
                            "Model {:?}, position: {:?}",
                            model.read().unwrap().0,
                            position
                        );
                        ecs_lock.add_component_to_entity(
                            entity,
                            instance::Instance {
                                position: cgmath::Vector3::new(
                                    position.read().unwrap().x,
                                    position.read().unwrap().y,
                                    position.read().unwrap().z,
                                ),
                                rotation: cgmath::Quaternion::from_angle_z(cgmath::Rad(0.0)),
                            },
                        );

                        if let Some(instance) =
                            ecs_lock.get_component_from_entity::<instance::Instance>(entity)
                        {
                            // Convert instances to raw format
                            let instance_data = instance.read().unwrap().to_raw();

                            // Create a buffer for the instances
                            let instance_buffer =
                                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("Instance Buffer"),
                                    contents: bytemuck::cast_slice(&[instance_data]),
                                    usage: wgpu::BufferUsages::VERTEX
                                        | wgpu::BufferUsages::COPY_DST,
                                });

                            ecs_lock.add_component_to_entity(entity, instance_buffer);
                        }
                    }
                }
            }
        }

        /* CAMERA */
        let camera_projection =
            camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_uniform = camera::CameraUniform::new();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        /* CAMERA */

        // /* LIGHT */
        // let light_uniform = light::LightUniform {
        //     position: [2.0, 2.0, 2.0],
        //     _padding: 0,
        //     color: [1.0, 1.0, 1.0],
        //     _padding2: 0,
        // };

        // TODO same models should be in the same buffer

        // // We'll want to update our lights position, so we use COPY_DST
        // let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Light VB"),
        //     contents: bytemuck::cast_slice(&[light_uniform]),
        //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        // });
        // /* LIGHT */
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };
            Self::create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), instance::InstanceRaw::desc()],
                shader,
            )
        };

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            };
            Self::create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader,
            )
        };

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            light_render_pipeline,
            camera: state_camera,
            camera_projection,
            camera_controller: state_camera_controller,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            light_model,
            light_uniform,
            light_buffer,
            light_bind_group,
            light_bind_group_layout,
            depth_texture,
            window,
            ecs,
            mouse_pressed: false,
        }
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        color_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        shader: wgpu::ShaderModuleDescriptor,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(shader);

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: vertex_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
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

    pub fn window(&self) -> &Window {
        self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.camera_projection
            .resize(new_size.width, new_size.height);

        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.size = new_size;
            //self.camera.aspect = self.config.width as f32 / self.config.height as f32;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(&delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    async fn update(&mut self, dt: instant::Duration) {
        /* Camera updates */
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.camera_projection);

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        /* Camera updates */

        /* Light updates */
        let old_position: cgmath::Vector3<_> = self.light_uniform.position.into();
        self.light_uniform.position = (cgmath::Quaternion::from_axis_angle(
            (0.0, 1.0, 0.0).into(),
            cgmath::Deg(60.0 * dt.as_secs_f32()),
        ) * old_position)
            .into();
        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
        /* Light updates */

        // TODO ezeket egybe lehetne szedni pl. light, buffer es uniform egy entity,
        // elejen letrehozni, user elol elrejteni
        // Update positions and instace buffers
        {
            let ecs_lock = self.ecs.lock().unwrap();

            for entity in ecs_lock.iter_entities() {
                if let Some(position) =
                    ecs_lock.get_component_from_entity::<components::Pos3>(entity)
                {
                    ecs_lock.add_component_to_entity(
                        entity,
                        instance::Instance {
                            position: cgmath::Vector3::new(
                                position.read().unwrap().x,
                                position.read().unwrap().y,
                                position.read().unwrap().z,
                            ),
                            rotation: cgmath::Quaternion::from_angle_z(cgmath::Rad(0.0)),
                        },
                    );

                    if let Some(instance) =
                        ecs_lock.get_component_from_entity::<instance::Instance>(entity)
                    {
                        // Convert instances to raw format
                        let instance_data = instance.read().unwrap().to_raw();

                        // Create a buffer for the instances
                        let instance_buffer =
                            self.device
                                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("Instance Buffer"),
                                    contents: bytemuck::cast_slice(&[instance_data]),
                                    usage: wgpu::BufferUsages::VERTEX
                                        | wgpu::BufferUsages::COPY_DST,
                                });

                        ecs_lock.add_component_to_entity(entity, instance_buffer);
                    }
                }
            }
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Draw light
            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model(
                &self.light_model,
                &self.camera_bind_group,
                &self.light_bind_group,
            );
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            {
                let ecs_lock = self.ecs.lock().unwrap();

                for entity in ecs_lock.iter_entities() {
                    if let Some(instance_buffer) =
                        ecs_lock.get_component_from_entity::<wgpu::Buffer>(entity)
                    {
                        render_pass.set_vertex_buffer(1, instance_buffer.read().unwrap().slice(..));
                    }

                    if let Some(model) = ecs_lock.get_component_from_entity::<model::Model>(entity)
                    {
                        let model = unsafe { &*(&*model.read().unwrap() as *const _) };

                        model::DrawModel::draw_model_instanced(
                            &mut render_pass,
                            model,
                            0..1,
                            &self.camera_bind_group,
                            &self.light_bind_group,
                        );
                    }
                }
            }
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
