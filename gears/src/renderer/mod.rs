pub mod camera;
pub mod instance;
pub mod light;
pub mod model;
pub mod resources;
pub mod texture;
pub mod traits;

use crate::core::Dt;
use crate::ecs::components::{Flip, Name, Scale};
use crate::ecs::{self, components};
use crate::gui::EguiRenderer;
use cgmath::prelude::*;
use cgmath::*;
use egui_wgpu::ScreenDescriptor;
use instant::Duration;
use log::{info, warn};
use model::{DrawModel, Vertex};
use std::f32::consts::FRAC_PI_2;
use std::num::NonZero;
use std::sync::{Arc, Mutex};
use std::{any, iter};
use tokio::sync::{broadcast, Mutex as TokioMutex};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalPosition;
use winit::event::{self, *};
use winit::window::WindowAttributes;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
    window::Window,
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
pub async fn run(
    ecs: Arc<Mutex<ecs::Manager>>,
    tx_dt: broadcast::Sender<Dt>,
    egui_windows: Option<Vec<Box<dyn FnMut(&egui::Context)>>>,
) -> anyhow::Result<()> {
    // * Window creation
    let event_loop = EventLoop::new()?;
    let window_attributes = WindowAttributes::default()
        .with_title("Winit window")
        .with_transparent(true)
        .with_window_icon(None);

    let window = event_loop.create_window(window_attributes)?;
    let mut state = State::new(&window, ecs).await;
    state.init_components().await?;

    if let Some(egui_windows) = egui_windows {
        state.egui_windows = egui_windows;
    }

    let mut last_render_time = instant::Instant::now();

    // * Event loop
    event_loop
        .run(move |event, ewlt| {
            // if let Event::DeviceEvent {
            //     event: DeviceEvent::MouseMotion{ delta, },
            //     .. // We're not using device_id currently
            // } = event {
            //     if state.mouse_pressed {
            //         state.camera_controller.process_mouse(delta.0, delta.1);
            //     }
            // }

            match event {
                // todo HANDLE this on a separate thread
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
                        }
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

                            // Send the delta time using the broadcast channel
                            if let Err(e) = tx_dt.send(dt) {
                                log::warn!("Failed to send delta time: {:?}", e);
                            }

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
            }
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
    camera: camera::Camera,
    camera_projection: camera::Projection,
    camera_controller: camera::CameraController,
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    light_entities: Option<Vec<ecs::Entity>>,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    model_entities: Option<Vec<ecs::Entity>>,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    light_bind_group_layout: wgpu::BindGroupLayout,
    depth_texture: texture::Texture,
    window: &'a Window,
    ecs: Arc<Mutex<ecs::Manager>>,
    mouse_pressed: bool,
    draw_colliders: bool,
    egui_renderer: EguiRenderer,
    egui_windows: Vec<Box<dyn FnMut(&egui::Context)>>,
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

        let power_pref = wgpu::PowerPreference::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_pref,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        log::warn!("[State] Device and Queue");
        let required_features = wgpu::Features::BUFFER_BINDING_ARRAY;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features,
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
                label: Some("light_bind_group_layout"),
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

        // * INITIALIZING STATE COMPONENTS
        // ! CAMERA COMPONENT
        let (state_camera, state_camera_controller) = Self::init_camera(Arc::clone(&ecs));

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: &[0; std::mem::size_of::<light::LightData>()], // ! Initialize the buffer for the maximum number of lights
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: Some("light_bind_group"),
        });
        // ! MODELS -> init_models()
        // * INITIALIZING STATE COMPONENTS

        /* CAMERA */
        let camera_projection =
            camera::Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_uniform = camera::CameraUniform::new();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // TODO same models should be in the same buffer

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

        // let light_render_pipeline = {
        //     let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //         label: Some("Light Pipeline Layout"),
        //         bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
        //         push_constant_ranges: &[],
        //     });
        //     let shader = wgpu::ShaderModuleDescriptor {
        //         label: Some("Light Shader"),
        //         source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
        //     };
        //     Self::create_render_pipeline(
        //         &device,
        //         &layout,
        //         config.format,
        //         Some(texture::Texture::DEPTH_FORMAT),
        //         &[model::ModelVertex::desc()],
        //         shader,
        //     )
        // };

        let egui_renderer = EguiRenderer::new(&device, surface_format, None, 1, window);
        let egui_windows = vec![];

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            camera: state_camera,
            camera_projection,
            texture_bind_group_layout,
            camera_controller: state_camera_controller,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            light_entities: None,
            light_buffer,
            light_bind_group,
            model_entities: None,
            light_bind_group_layout,
            depth_texture,
            window,
            ecs,
            mouse_pressed: false,
            draw_colliders: true,
            egui_renderer,
            egui_windows,
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

    async fn init_components(&mut self) -> anyhow::Result<()> {
        self.init_lights().await;
        self.init_models().await;

        Ok(())
    }

    fn init_camera(ecs: Arc<Mutex<ecs::Manager>>) -> (camera::Camera, camera::CameraController) {
        let ecs_lock = ecs.lock().unwrap();
        let mut camera_entity = ecs_lock.get_entites_with_component::<components::Camera>();
        assert!(
            camera_entity.len() <= 1,
            "There should be only one camera entity"
        );

        // If there is no camera entity provide a default implementation
        if camera_entity.is_empty() {
            let camera =
                camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
            let controller = camera::CameraController::new(0.5, 0.2);

            return (camera, controller);
        }

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
                let pos_point = cgmath::Point3::from_vec(camera_pos.pos);
                let look_at_point = look_at;
                let camera = camera::Camera::new_look_at(pos_point, look_at_point);
                let controller = camera::CameraController::new(speed, sensitivity);

                (camera, controller)
            }
            components::Camera::Fixed { look_at } => {
                let pos_point = cgmath::Point3::from_vec(camera_pos.pos);
                let look_at_point = look_at;
                let camera = camera::Camera::new_look_at(pos_point, look_at_point);
                let controller = camera::CameraController::new(0.0, 0.0);

                (camera, controller)
            }
        }
    }

    async fn init_lights(&mut self) {
        let ecs_lock = self.ecs.lock().unwrap();
        let light_entities = ecs_lock.get_entites_with_component::<components::Light>();

        for entity in light_entities.iter() {
            let pos = ecs_lock
                .get_component_from_entity::<components::Pos3>(*entity)
                .expect("No position provided for the light!");

            let light = ecs_lock
                .get_component_from_entity::<components::Light>(*entity)
                .unwrap();

            let light_uniform = {
                let rlock_pos = pos.read().unwrap();
                let rlock_light = light.read().unwrap();

                match *rlock_light {
                    components::Light::Point { radius, intensity } => light::LightUniform {
                        position: [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z],
                        light_type: light::LightType::Point as u32,
                        color: [1.0, 1.0, 1.0],
                        radius,
                        direction: [0.0; 3],
                        intensity,
                    },
                    components::Light::PointColoured {
                        radius,
                        color,
                        intensity,
                    } => light::LightUniform {
                        position: [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z],
                        light_type: light::LightType::Point as u32,
                        color,
                        radius,
                        direction: [0.0; 3],
                        intensity,
                    },
                    components::Light::Ambient { intensity } => light::LightUniform {
                        position: [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z],
                        light_type: light::LightType::Ambient as u32,
                        color: [1.0, 1.0, 1.0],
                        radius: 0.0,
                        direction: [0.0; 3],
                        intensity,
                    },
                    components::Light::AmbientColoured { color, intensity } => {
                        light::LightUniform {
                            position: [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z],
                            light_type: light::LightType::Ambient as u32,
                            color,
                            radius: 0.0,
                            direction: [0.0; 3],
                            intensity,
                        }
                    }
                    components::Light::Directional {
                        direction,
                        intensity,
                    } => light::LightUniform {
                        position: [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z],
                        light_type: light::LightType::Directional as u32,
                        color: [1.0, 1.0, 1.0],
                        radius: 0.0,
                        direction,
                        intensity,
                    },
                    components::Light::DirectionalColoured {
                        direction,
                        color,
                        intensity,
                    } => light::LightUniform {
                        position: [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z],
                        light_type: light::LightType::Directional as u32,
                        color,
                        radius: 0.0,
                        direction,
                        intensity,
                    },
                }
            };
            ecs_lock.add_component_to_entity(*entity, light_uniform);
        }

        if light_entities.len() > light::NUM_MAX_LIGHTS as usize {
            panic!("The number of lights exceeds the maximum number of lights supported by the renderer!");
        }

        self.light_entities = Some(light_entities);
    }

    async fn init_models(&mut self) {
        let ecs_lock = self.ecs.lock().unwrap();
        let model_entities = ecs_lock.get_entites_with_component::<components::Model>();

        for entity in model_entities.iter() {
            let name = ecs_lock
                .get_component_from_entity::<components::Name>(*entity)
                .expect("No name provided for the Model!");

            let pos = ecs_lock
                .get_component_from_entity::<components::Pos3>(*entity)
                .expect("No position provided for the Model!");

            let model = ecs_lock
                .get_component_from_entity::<components::Model>(*entity)
                .unwrap();

            let flip = ecs_lock.get_component_from_entity::<components::Flip>(*entity);

            let scale = ecs_lock.get_component_from_entity::<components::Scale>(*entity);

            let obj_model = {
                let model = model.read().unwrap();

                match *model {
                    components::Model::Dynamic { obj_path } => resources::load_model(
                        obj_path,
                        &self.device,
                        &self.queue,
                        &self.texture_bind_group_layout,
                    )
                    .await
                    .unwrap(),
                    components::Model::Static { obj_path } => resources::load_model(
                        obj_path,
                        &self.device,
                        &self.queue,
                        &self.texture_bind_group_layout,
                    )
                    .await
                    .unwrap(),
                }
            };
            ecs_lock.add_component_to_entity(*entity, obj_model);

            // TODO rename instance to model::ModelUniform
            let mut instance = {
                let rlock_pos = pos.read().unwrap();
                instance::Instance {
                    position: rlock_pos.pos,
                    rotation: rlock_pos
                        .rot
                        .unwrap_or(cgmath::Quaternion::from_angle_y(cgmath::Rad(0.0))),
                }
            };

            if let Some(flip) = flip {
                let rlock_flip = flip.read().unwrap();

                match *rlock_flip {
                    Flip::Horizontal => {
                        instance.rotation =
                            cgmath::Quaternion::from_angle_y(cgmath::Rad(std::f32::consts::PI));
                    }
                    Flip::Vertical => {
                        instance.rotation =
                            cgmath::Quaternion::from_angle_x(cgmath::Rad(std::f32::consts::PI));
                    }
                    Flip::Both => {
                        instance.rotation =
                            cgmath::Quaternion::from_angle_y(cgmath::Rad(std::f32::consts::PI));
                        instance.rotation =
                            cgmath::Quaternion::from_angle_x(cgmath::Rad(std::f32::consts::PI));
                    }
                }
            }

            // if let Some(scale) = scale {
            //     let rlock_scale = scale.read().unwrap();

            //     match *rlock_scale {
            //         Scale::Uniform(s) => {
            //             instance.scale = cgmath::Vector3::new(s, s, s);
            //         }
            //         Scale::NonUniform { x, y, z } => {
            //             instance.scale = cgmath::Vector3::new(x, y, z);
            //         }
            //     }
            // }

            let instance_raw = instance.to_raw();
            let instance_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(format!("{} Instance Buffer", name.read().unwrap().0).as_str()),
                        contents: bytemuck::cast_slice(&[instance_raw]),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });
            ecs_lock.add_component_to_entity(*entity, instance);
            ecs_lock.add_component_to_entity(*entity, instance_buffer);
        }

        self.model_entities = Some(model_entities);
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
        // TODO is this important? chek perf on DGPU
        //self.window.request_redraw();

        // * Capture the input for the custom windows
        if self.egui_renderer.handle_input(self.window, event) {
            // If a window consumed the event return true since no other component should handle it again
            return true;
        }

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
                self.camera_controller.process_scroll(delta);
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
        // Update camera
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.camera_projection);

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        self.update_lights();
        self.update_models();
        //self.update_colliders();
    }

    fn update_lights(&mut self) {
        if let Some(light_entities) = &self.light_entities {
            let mut light_uniforms: Vec<light::LightUniform> = Vec::new();

            for entity in light_entities {
                let ecs_lock = self.ecs.lock().unwrap();

                let pos = ecs_lock
                    .get_component_from_entity::<components::Pos3>(*entity)
                    .unwrap();
                let light_uniform = ecs_lock
                    .get_component_from_entity::<light::LightUniform>(*entity)
                    .unwrap();

                {
                    // TODO update the colors
                    let rlock_pos = pos.read().unwrap();

                    light_uniform.write().unwrap().position =
                        [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z];
                }

                let rlock_light_uniform = light_uniform.read().unwrap();

                light_uniforms.push(*rlock_light_uniform);
            }

            let num_lights = light_uniforms.len() as u32;

            let light_data = light::LightData {
                lights: {
                    let mut array =
                        [light::LightUniform::default(); light::NUM_MAX_LIGHTS as usize];
                    for (i, light) in light_uniforms.iter().enumerate() {
                        array[i] = *light;
                    }
                    array
                },
                num_lights,
                _padding: [0; 3],
            };

            self.queue
                .write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[light_data]));
        }
    }

    fn update_models(&mut self) {
        if let Some(model_entities) = &self.model_entities {
            for entity in model_entities {
                let ecs_lock = self.ecs.lock().unwrap();

                let model_type = ecs_lock.get_component_from_entity::<components::Model>(*entity);

                if let Some(model_type) = model_type {
                    let model_type = model_type.read().unwrap();
                    if let components::Model::Static { .. } = *model_type {
                        continue;
                    }
                }

                let pos = ecs_lock
                    .get_component_from_entity::<components::Pos3>(*entity)
                    .unwrap();
                let instance = ecs_lock
                    .get_component_from_entity::<instance::Instance>(*entity)
                    .unwrap();
                let buffer = ecs_lock
                    .get_component_from_entity::<wgpu::Buffer>(*entity)
                    .unwrap();

                // TODO rotation
                {
                    let mut wlock_instance = instance.write().unwrap();
                    let rlock_pos3 = pos.read().unwrap();

                    wlock_instance.position = rlock_pos3.pos;
                    wlock_instance.rotation = rlock_pos3
                        .rot
                        .unwrap_or(cgmath::Quaternion::from_angle_y(cgmath::Rad(0.0)));
                }

                let instance_raw = instance.read().unwrap().to_raw();
                self.queue.write_buffer(
                    &buffer.write().unwrap(),
                    0,
                    bytemuck::cast_slice(&[instance_raw]),
                );
            }
        }
    }

    // fn update_colliders(&mut self) {
    //     let ecs_lock = self.ecs.lock().unwrap();
    //     let collider_entities = ecs_lock.get_entites_with_component::<components::Collider>();

    //     for entity in collider_entities.iter() {
    //         let pos = ecs_lock
    //             .get_component_from_entity::<components::Pos3>(*entity)
    //             .unwrap();
    //         let collider = ecs_lock
    //             .get_component_from_entity::<components::Collider>(*entity)
    //             .unwrap();

    //         let pos = pos.read().unwrap();
    //         let collider = collider.read().unwrap();
    //     }
    // }

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

        // ! Graphical render pass
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(2, &self.light_bind_group, &[]);

            if let Some(model_entities) = &self.model_entities {
                for entity in model_entities {
                    let ecs_lock = self.ecs.lock().unwrap();

                    let model = ecs_lock
                        .get_component_from_entity::<model::Model>(*entity)
                        .unwrap();
                    let instance_buffer = ecs_lock
                        .get_component_from_entity::<wgpu::Buffer>(*entity)
                        .unwrap();

                    let model: &model::Model = unsafe { &*(&*model.read().unwrap() as *const _) };

                    render_pass.set_vertex_buffer(1, instance_buffer.read().unwrap().slice(..));

                    // Draw model
                    render_pass.draw_model(model, &self.camera_bind_group, &self.light_bind_group);
                }
            }
        }

        // ! Egui render pass for the custom UI windows
        if !self.egui_windows.is_empty() {
            // * if a custom ui is present
            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: self.window.scale_factor() as f32,
            };

            self.egui_renderer.draw_multiple_ui_full(
                &self.device,
                &self.queue,
                &mut encoder,
                self.window,
                &view,
                &screen_descriptor,
                &mut self.egui_windows,
            );
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
