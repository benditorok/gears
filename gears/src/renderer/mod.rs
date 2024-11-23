pub mod camera;
pub mod instance;
pub mod light;
pub mod model;
pub mod resources;
pub mod texture;
pub mod traits;

use crate::core::Dt;
use crate::ecs::components::controllers;
use crate::ecs::components::prefabs::Player;
use crate::ecs::traits::{Marker, Tick};
use crate::ecs::{self, components};
use crate::gui::EguiRenderer;
use cgmath::prelude::*;
use egui_wgpu::ScreenDescriptor;
use log::{info, warn};
use model::{DrawModelMesh, DrawWireframeMesh, Vertex};
use std::f32::consts::FRAC_PI_2;
use std::iter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{self, Instant};
use tokio::sync::broadcast;
use wgpu::util::DeviceExt;
use winit::event::*;
use winit::window::WindowAttributes;
use winit::{
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

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

    let mut last_render_time = time::Instant::now();

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
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    // Handle the mouse motion for the camera if the state is NOT in a paused state
                    if !state.is_state_paused.load(Ordering::Relaxed) {
                        if let Some(view_controller) = &state.view_controller {
                            let mut wlock_view_controller = view_controller.write().unwrap();
                            wlock_view_controller.process_mouse(delta.0, delta.1);
                        }
                    }
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                    // TODO the state.input should handle device events as well as window events
                } if window_id == state.window().id() && !state.input(event) => {
                    match event {
                        WindowEvent::CloseRequested => ewlt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        // WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => {
                        //     *inner_size_writer = state.size.to_logical::<f64>(*scale_factor);
                        // }
                        WindowEvent::RedrawRequested => {
                            let now = time::Instant::now();
                            let dt = now - last_render_time;
                            last_render_time = now;

                            // If the state is paused, busy wait
                            if state.is_state_paused.load(Ordering::Relaxed) {
                                std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 fps
                                return;
                            }

                            // * Log FPS
                            // info!(
                            //     "FPS: {:.0}, frame time: {} ms",
                            //     1.0 / &dt.as_secs_f32(),
                            //     &dt.as_millis()
                            // );

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

/// Global state of the application. This is where all rendering related data is stored.
///
/// The State is responsible for handling the rendering pipeline, the camera, the lights,
/// the models, the window, etc.
struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    movement_controller: Option<Arc<RwLock<components::controllers::MovementController>>>,
    view_controller: Option<Arc<RwLock<components::controllers::ViewController>>>,
    player_entity: Option<ecs::Entity>,
    camera_owner_entity: Option<ecs::Entity>,
    camera_projection: camera::Projection,
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    light_entities: Option<Vec<ecs::Entity>>,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    static_model_entities: Option<Vec<ecs::Entity>>,
    physics_entities: Option<Vec<ecs::Entity>>,
    drawable_entities: Option<Vec<ecs::Entity>>,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    light_bind_group_layout: wgpu::BindGroupLayout,
    depth_texture: texture::Texture,
    window: &'a Window,
    ecs: Arc<Mutex<ecs::Manager>>,
    mouse_pressed: bool,
    draw_colliders: bool,
    egui_renderer: EguiRenderer,
    egui_windows: Vec<Box<dyn FnMut(&egui::Context)>>,
    is_state_paused: AtomicBool,
    time: Instant,
    collider_render_pipeline: wgpu::RenderPipeline,
}

impl<'a> State<'a> {
    /// Create a new instance of the State.
    ///
    /// # Arguments
    ///
    /// * `window` - The window to render to.
    /// * `ecs` - The ECS manager.
    ///
    /// # Returns
    ///
    /// A new instance of the State.
    async fn new(window: &'a Window, ecs: Arc<Mutex<ecs::Manager>>) -> State<'a> {
        // * Initializing the backend
        // The instance is a handle to the GPU. BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
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

        // * Initializing the device and queue
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

        // * Configuring the surface
        let size = window.inner_size();
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

        // ! BIND GROUP LAYOUTS
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

        // * Initializing the camera
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

        // * Light buffer and bind group
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

        // ! Global render pipeline
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");
        let render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Main Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Main Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };
            Self::create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), instance::InstanceRaw::desc()],
                shader,
                "Main Render Pipeline",
            )
        };

        // * Initializing the egui renderer
        let egui_renderer = EguiRenderer::new(&device, surface_format, None, 1, window);
        let egui_windows = vec![];

        // * Wireframe render pipeline
        let collider_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Collider Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Collider Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader_collider.wgsl").into()),
            };

            Self::create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ColliderVertex::desc(), instance::InstanceRaw::desc()],
                shader,
                "Collider Render Pipeline",
            )
        };

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            movement_controller: None,
            view_controller: None,
            camera_owner_entity: None,
            camera_projection,
            texture_bind_group_layout,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            light_entities: None,
            light_buffer,
            light_bind_group,
            static_model_entities: None,
            physics_entities: None,
            drawable_entities: None,
            light_bind_group_layout,
            depth_texture,
            window,
            ecs,
            mouse_pressed: false,
            draw_colliders: true,
            egui_renderer,
            egui_windows,
            is_state_paused: AtomicBool::new(false),
            time: time::Instant::now(),
            collider_render_pipeline,
            player_entity: None,
        }
    }

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
    /// A new render pipeline.
    fn create_render_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        color_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        shader: wgpu::ShaderModuleDescriptor,
        pipeline_label: &'static str,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(shader);

        let is_collider_pipeline = pipeline_label.contains("Collider");

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(pipeline_label),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: vertex_layouts,
                compilation_options: wgpu::PipelineCompilationOptions {
                    ..Default::default()
                },
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(if is_collider_pipeline {
                        wgpu::BlendState {
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
                        }
                    } else {
                        wgpu::BlendState::REPLACE
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions {
                    ..Default::default()
                },
            }),
            primitive: wgpu::PrimitiveState {
                topology: if is_collider_pipeline {
                    wgpu::PrimitiveTopology::LineList
                } else {
                    wgpu::PrimitiveTopology::TriangleList
                },
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: if is_collider_pipeline {
                    None
                } else {
                    Some(wgpu::Face::Back)
                },
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

    /// Initialize the components which can be rendered.
    async fn init_components(&mut self) -> anyhow::Result<()> {
        if !self.init_player() {
            self.init_camera();
        }

        self.init_lights().await;

        // * The order of these is important!
        self.init_models().await;
        self.init_physics_models().await;

        Ok(())
    }

    /// Initialie the player component.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the player was found.
    fn init_player(&mut self) -> bool {
        let ecs_lock = self.ecs.lock().unwrap();

        // * Look for a player first and retrieve it's camera
        let mut player_entity =
            ecs_lock.get_entites_with_component::<components::misc::PlayerMarker>();

        if !player_entity.is_empty() {
            let player_entity = player_entity.pop().unwrap();
            self.player_entity = Some(player_entity);
            self.camera_owner_entity = Some(player_entity);
            //self.camera_type = ecs::components::misc::CameraType::Player;

            let view_controller = ecs_lock
                .get_component_from_entity::<components::controllers::ViewController>(player_entity)
                .expect(components::misc::PlayerMarker::describe());
            self.view_controller = Some(Arc::clone(&view_controller));

            let movement_controller = ecs_lock
                .get_component_from_entity::<components::controllers::MovementController>(
                    player_entity,
                )
                .expect(components::misc::PlayerMarker::describe());
            self.movement_controller = Some(Arc::clone(&movement_controller));

            return true;
        }

        false
    }

    /// Initialize the camera component.
    fn init_camera(&mut self) {
        let ecs_lock = self.ecs.lock().unwrap();

        let mut static_camera_entity =
            ecs_lock.get_entites_with_component::<components::misc::StaticCameraMarker>();

        if !static_camera_entity.is_empty() {
            let static_camera_entity = static_camera_entity.pop().unwrap();
            self.camera_owner_entity = Some(static_camera_entity);
            //self.camera_type = ecs::components::misc::CameraType::Static;

            let controller = ecs_lock
                .get_component_from_entity::<components::controllers::ViewController>(
                    static_camera_entity,
                )
                .expect(components::misc::PlayerMarker::describe());
            self.view_controller = Some(Arc::clone(&controller));
            return;
        }

        let mut dynamic_camera_entity =
            ecs_lock.get_entites_with_component::<components::misc::DynamicCameraMarker>();

        if !dynamic_camera_entity.is_empty() {
            let dynamic_camera_entity = dynamic_camera_entity.pop().unwrap();
            self.camera_owner_entity = Some(dynamic_camera_entity);
            //self.camera_type = ecs::components::misc::CameraType::Dynamic;

            let controller = ecs_lock
                .get_component_from_entity::<components::controllers::ViewController>(
                    dynamic_camera_entity,
                )
                .expect(components::misc::PlayerMarker::describe());
            self.view_controller = Some(Arc::clone(&controller));
            return;
        }

        panic!("No camera found in the ECS!");
    }

    /// Initialize the light components.
    ///
    /// # Returns
    ///
    /// A future which can be awaited.
    async fn init_lights(&mut self) {
        let ecs_lock = self.ecs.lock().unwrap();
        let light_entities = ecs_lock.get_entites_with_component::<components::misc::LightMarker>();

        for entity in light_entities.iter() {
            let pos = ecs_lock
                .get_component_from_entity::<components::transforms::Pos3>(*entity)
                .expect(components::misc::LightMarker::describe());

            let light = ecs_lock
                .get_component_from_entity::<components::lights::Light>(*entity)
                .expect(components::misc::LightMarker::describe());

            let light_uniform = {
                let rlock_pos = pos.read().unwrap();
                let rlock_light = light.read().unwrap();

                match *rlock_light {
                    components::lights::Light::Point { radius, intensity } => light::LightUniform {
                        position: [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z],
                        light_type: light::LightType::Point as u32,
                        color: [1.0, 1.0, 1.0],
                        radius,
                        direction: [0.0; 3],
                        intensity,
                    },
                    components::lights::Light::PointColoured {
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
                    components::lights::Light::Ambient { intensity } => light::LightUniform {
                        position: [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z],
                        light_type: light::LightType::Ambient as u32,
                        color: [1.0, 1.0, 1.0],
                        radius: 0.0,
                        direction: [0.0; 3],
                        intensity,
                    },
                    components::lights::Light::AmbientColoured { color, intensity } => {
                        light::LightUniform {
                            position: [rlock_pos.pos.x, rlock_pos.pos.y, rlock_pos.pos.z],
                            light_type: light::LightType::Ambient as u32,
                            color,
                            radius: 0.0,
                            direction: [0.0; 3],
                            intensity,
                        }
                    }
                    components::lights::Light::Directional {
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
                    components::lights::Light::DirectionalColoured {
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

    /// Initialize the model components.
    ///
    /// # Returns
    ///
    /// A future which can be awaited.
    async fn init_models(&mut self) {
        let ecs_lock = self.ecs.lock().unwrap();
        let model_entities =
            ecs_lock.get_entites_with_component::<components::misc::StaticModelMarker>();

        for entity in model_entities.iter() {
            let name = ecs_lock
                .get_component_from_entity::<components::misc::Name>(*entity)
                .expect(components::misc::StaticModelMarker::describe());
            let pos3 = ecs_lock
                .get_component_from_entity::<components::transforms::Pos3>(*entity)
                .expect(components::misc::StaticModelMarker::describe());
            let model_source = ecs_lock
                .get_component_from_entity::<components::models::ModelSource>(*entity)
                .expect(components::misc::StaticModelMarker::describe());

            let flip = ecs_lock.get_component_from_entity::<components::transforms::Flip>(*entity);

            let scale =
                ecs_lock.get_component_from_entity::<components::transforms::Scale>(*entity);

            let obj_model = {
                let rlock_model_source = model_source.read().unwrap();

                match *rlock_model_source {
                    components::models::ModelSource::Obj(path) => resources::load_model_obj(
                        path,
                        &self.device,
                        &self.queue,
                        &self.texture_bind_group_layout,
                    )
                    .await
                    .unwrap(),
                    components::models::ModelSource::Gltf(path) => resources::load_model_gltf(
                        path,
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
                let rlock_pos3 = pos3.read().unwrap();
                instance::Instance {
                    position: rlock_pos3.pos,
                    rotation: rlock_pos3.rot,
                }
            };

            if let Some(flip) = flip {
                let rlock_flip = flip.read().unwrap();

                match *rlock_flip {
                    components::transforms::Flip::Horizontal => {
                        instance.rotation =
                            cgmath::Quaternion::from_angle_y(cgmath::Rad(std::f32::consts::PI));
                    }
                    components::transforms::Flip::Vertical => {
                        instance.rotation =
                            cgmath::Quaternion::from_angle_x(cgmath::Rad(std::f32::consts::PI));
                    }
                    components::transforms::Flip::Both => {
                        instance.rotation =
                            cgmath::Quaternion::from_angle_y(cgmath::Rad(std::f32::consts::PI));
                        instance.rotation =
                            cgmath::Quaternion::from_angle_x(cgmath::Rad(std::f32::consts::PI));
                    }
                }
            }

            // TODO scale should update the rot (quaternion)??
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

        self.static_model_entities = Some(model_entities.clone());
        self.drawable_entities = Some(model_entities);
    }

    async fn init_physics_models(&mut self) {
        let ecs_lock = self.ecs.lock().unwrap();
        let physics_entities =
            ecs_lock.get_entites_with_component::<components::misc::RigidBodyMarker>();

        for entity in physics_entities.iter() {
            let name = ecs_lock
                .get_component_from_entity::<components::misc::Name>(*entity)
                .expect("No name provided for the Model!");

            let physics_body = ecs_lock
                .get_component_from_entity::<components::physics::RigidBody>(*entity)
                .expect(components::misc::RigidBodyMarker::describe());
            let model_source = ecs_lock
                .get_component_from_entity::<components::models::ModelSource>(*entity)
                .expect(components::misc::RigidBodyMarker::describe());
            let pos3 = ecs_lock
                .get_component_from_entity::<components::transforms::Pos3>(*entity)
                .expect(components::misc::RigidBodyMarker::describe());

            let flip = ecs_lock.get_component_from_entity::<components::transforms::Flip>(*entity);

            let scale =
                ecs_lock.get_component_from_entity::<components::transforms::Scale>(*entity);

            let obj_model = {
                let rlock_model_source = model_source.read().unwrap();

                match *rlock_model_source {
                    components::models::ModelSource::Obj(path) => resources::load_model_obj(
                        path,
                        &self.device,
                        &self.queue,
                        &self.texture_bind_group_layout,
                    )
                    .await
                    .unwrap(),
                    components::models::ModelSource::Gltf(path) => resources::load_model_gltf(
                        path,
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
                let rlock_physics_body = physics_body.read().unwrap();
                let rlock_pos3 = pos3.read().unwrap();
                instance::Instance {
                    position: rlock_pos3.pos,
                    rotation: rlock_pos3.rot,
                }
            };

            if let Some(flip) = flip {
                let rlock_flip = flip.read().unwrap();

                match *rlock_flip {
                    components::transforms::Flip::Horizontal => {
                        instance.rotation =
                            cgmath::Quaternion::from_angle_y(cgmath::Rad(std::f32::consts::PI));
                    }
                    components::transforms::Flip::Vertical => {
                        instance.rotation =
                            cgmath::Quaternion::from_angle_x(cgmath::Rad(std::f32::consts::PI));
                    }
                    components::transforms::Flip::Both => {
                        instance.rotation =
                            cgmath::Quaternion::from_angle_y(cgmath::Rad(std::f32::consts::PI));
                        instance.rotation =
                            cgmath::Quaternion::from_angle_x(cgmath::Rad(std::f32::consts::PI));
                    }
                }
            }

            // TODO scale should update the rot (quaternion)??
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

            // Create a wireframe collider from the RigidBody's data
            let wireframe = model::WireframeMesh::new(&self.device, &physics_body.read().unwrap());
            ecs_lock.add_component_to_entity(*entity, wireframe);
        }

        self.physics_entities = Some(physics_entities.clone());

        if let Some(drawable_entities) = &mut self.drawable_entities {
            drawable_entities.extend(physics_entities);
        }
    }

    /// Get a reference to the window used by the state.
    ///
    /// # Returns
    ///
    /// A reference to the window.
    pub fn window(&self) -> &Window {
        self.window
    }

    /// Resize the window when the size changes.
    ///
    /// # Arguments
    ///
    /// * `new_size` - The new size of the window.
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

    /// Handle the input events.
    ///
    /// # Arguments
    ///
    /// * `event` - The window event.
    ///
    /// # Returns
    ///
    /// A boolean indicating if the event was consumed.
    fn input(&mut self, event: &WindowEvent) -> bool {
        // * Pause the state
        if let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    ..
                },
            ..
        } = event
        {
            self.is_state_paused.store(
                !self.is_state_paused.load(Ordering::Relaxed),
                Ordering::Relaxed,
            );
            return true;
        }

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
            } => {
                if let Some(movement_controller) = &self.movement_controller {
                    let mut wlock_movement_controller = movement_controller.write().unwrap();
                    wlock_movement_controller.process_keyboard(*key, *state);
                }

                true
            }
            // WindowEvent::MouseWheel { delta, .. } => {
            //     self.view_controller.process_scroll(delta);
            //     true
            // }
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

    /// Update the state of the application.
    /// This function is called every frame to update the state of the application.
    ///
    /// # Arguments
    ///
    /// * `dt` - The delta time since the last frame.
    ///
    /// # Returns
    ///
    /// A future which can be awaited.
    async fn update(&mut self, dt: time::Duration) {
        // ! Update the camera (view controller). If the camera is a player, then update the movement controller as well.
        if let Some(view_controller) = &self.view_controller {
            let ecs_lock = self.ecs.lock().unwrap();
            let camera_entity = self.camera_owner_entity.unwrap();
            let pos3 = ecs_lock.get_component_from_entity(camera_entity).unwrap();
            let mut wlock_pos3 = pos3.write().unwrap();
            let mut wlock_view_controller = view_controller.write().unwrap();
            wlock_view_controller.update_rot(&mut wlock_pos3, dt.as_secs_f32());

            if let Some(movement_controller) = &self.movement_controller {
                let rlock_movement_controller = movement_controller.read().unwrap();
                if let Some(rigid_body) = ecs_lock
                    .get_component_from_entity::<components::physics::RigidBody>(camera_entity)
                {
                    let mut wlock_rigid_body = rigid_body.write().unwrap();

                    rlock_movement_controller.update_pos(
                        &wlock_view_controller,
                        &mut wlock_pos3,
                        Some(&mut wlock_rigid_body),
                        dt.as_secs_f32(),
                    );
                } else {
                    rlock_movement_controller.update_pos(
                        &wlock_view_controller,
                        &mut wlock_pos3,
                        None,
                        dt.as_secs_f32(),
                    );
                }
            }

            self.camera_uniform.update_view_proj(
                &wlock_pos3,
                &wlock_view_controller,
                &self.camera_projection,
            );

            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );
        }

        self.update_physics_system(dt);
        self.update_lights();
        self.update_models();
    }

    /// Update the lights in the scene.
    ///
    /// # Returns
    ///
    /// A future which can be awaited.
    fn update_lights(&mut self) {
        if let Some(light_entities) = &self.light_entities {
            let mut light_uniforms: Vec<light::LightUniform> = Vec::new();

            for entity in light_entities {
                let ecs_lock = self.ecs.lock().unwrap();

                let pos = ecs_lock
                    .get_component_from_entity::<components::transforms::Pos3>(*entity)
                    .unwrap();
                let light_uniform = ecs_lock
                    .get_component_from_entity::<light::LightUniform>(*entity)
                    .unwrap();
                let light = ecs_lock
                    .get_component_from_entity::<components::lights::Light>(*entity)
                    .unwrap();

                {
                    // TODO update the colors
                    let rlock_pos = pos.read().unwrap();
                    let mut wlock_light_uniform = light_uniform.write().unwrap();

                    wlock_light_uniform.position =
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

    /// Update the models in the scene.
    ///
    /// # Returns
    ///
    /// A future which can be awaited.
    fn update_models(&mut self) {
        if let Some(model_entities) = &self.static_model_entities {
            for entity in model_entities {
                let ecs_lock = self.ecs.lock().unwrap();

                let name = ecs_lock
                    .get_component_from_entity::<components::misc::Name>(*entity)
                    .expect(components::misc::StaticModelMarker::describe());
                let pos3 = ecs_lock
                    .get_component_from_entity::<components::transforms::Pos3>(*entity)
                    .expect(components::misc::StaticModelMarker::describe());
                let instance = ecs_lock
                    .get_component_from_entity::<instance::Instance>(*entity)
                    .expect(components::misc::StaticModelMarker::describe());
                let buffer = ecs_lock
                    .get_component_from_entity::<wgpu::Buffer>(*entity)
                    .unwrap();
                let model = ecs_lock
                    .get_component_from_entity::<model::Model>(*entity)
                    .unwrap();
                let animation_queue =
                    ecs_lock.get_component_from_entity::<components::misc::AnimationQueue>(*entity);

                // ! Animations testing
                if let Some(animation_queue) = animation_queue {
                    // * This will run if an animation is queued
                    if let Some(selected_animation) = animation_queue.write().unwrap().pop() {
                        let rlock_model = model.read().unwrap();

                        let current_time = self.time.elapsed().as_secs_f32();
                        let animation = &rlock_model.get_animation(selected_animation).unwrap();
                        let mut current_keyframe_index = 0;

                        // Find the two keyframes surrounding the current_time
                        for (i, timestamp) in animation.timestamps.iter().enumerate() {
                            if *timestamp > current_time {
                                current_keyframe_index = i - 1;
                                break;
                            }
                            current_keyframe_index = i;
                        }

                        // Loop the animation
                        if current_keyframe_index >= animation.timestamps.len() - 1 {
                            self.time = Instant::now();
                            current_keyframe_index = 0;
                        }

                        let next_keyframe_index = current_keyframe_index + 1;
                        let t0 = animation.timestamps[current_keyframe_index];
                        let t1 = animation.timestamps[next_keyframe_index];
                        let factor = (current_time - t0) / (t1 - t0);

                        // TODO animations should also take positions into consideration while playing
                        let current_animation = &animation.keyframes;
                        match current_animation {
                            model::Keyframes::Translation(frames) => {
                                let start_frame = &frames[current_keyframe_index];
                                let end_frame = &frames[next_keyframe_index];

                                // Ensure frames have exactly 3 elements
                                if start_frame.len() == 3 && end_frame.len() == 3 {
                                    let start = cgmath::Vector3::new(
                                        start_frame[0],
                                        start_frame[1],
                                        start_frame[2],
                                    );
                                    let end = cgmath::Vector3::new(
                                        end_frame[0],
                                        end_frame[1],
                                        end_frame[2],
                                    );
                                    let interpolated = start.lerp(end, factor);
                                    let mut wlock_instance = instance.write().unwrap();
                                    wlock_instance.position = interpolated;
                                } else {
                                    warn!("Translation frames do not have exactly 3 elements.");
                                }
                            }
                            model::Keyframes::Rotation(quats) => {
                                let start_quat = &quats[current_keyframe_index];
                                let end_quat = &quats[next_keyframe_index];

                                // Ensure quaternions have exactly 4 elements
                                if start_quat.len() == 4 && end_quat.len() == 4 {
                                    let start = cgmath::Quaternion::new(
                                        start_quat[0],
                                        start_quat[1],
                                        start_quat[2],
                                        start_quat[3],
                                    );
                                    let end = cgmath::Quaternion::new(
                                        end_quat[0],
                                        end_quat[1],
                                        end_quat[2],
                                        end_quat[3],
                                    );
                                    let interpolated = start.slerp(end, factor);
                                    let mut wlock_instance = instance.write().unwrap();
                                    wlock_instance.rotation = interpolated;
                                } else {
                                    warn!("Rotation quaternions do not have exactly 4 elements.");
                                }
                            }
                            model::Keyframes::Scale(_) => {
                                // Handle scale interpolation if necessary
                            }
                            model::Keyframes::Other => {
                                warn!("Other animations are not supported yet!")
                            }
                        }
                    } else {
                        // If the AnimationQueue is emtpy
                        // ! Do not remove, causes deadlock if the lock is held for more
                        {
                            let mut wlock_instance = instance.write().unwrap();
                            let rlock_pos3 = pos3.read().unwrap();

                            wlock_instance.position = rlock_pos3.pos;
                            wlock_instance.rotation = rlock_pos3.rot;
                        }
                    }
                } else {
                    // If there is no AnimationQueue
                    // ! Do not remove, causes deadlock if the lock is held for more
                    {
                        let mut wlock_instance = instance.write().unwrap();
                        let rlock_pos3 = pos3.read().unwrap();

                        wlock_instance.position = rlock_pos3.pos;
                        wlock_instance.rotation = rlock_pos3.rot;
                    }
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

    fn update_physics_system(&mut self, dt: time::Duration) {
        let dt = dt.as_secs_f32();
        let mut physics_bodies = Vec::new();

        if let Some(player) = self.player_entity {
            let ecs_lock = self.ecs.lock().unwrap();
            physics_bodies.push((
                player,
                ecs_lock
                    .get_component_from_entity::<components::physics::RigidBody>(player)
                    .unwrap(),
                ecs_lock
                    .get_component_from_entity::<components::transforms::Pos3>(player)
                    .unwrap(),
            ));
        }

        if let Some(physics_entities) = &self.physics_entities {
            for entity in physics_entities {
                // TODO add an animation queue for physics entities as well
                let ecs_lock = self.ecs.lock().unwrap();

                let physics_body = ecs_lock
                    .get_component_from_entity::<components::physics::RigidBody>(*entity)
                    .unwrap();

                let instance = ecs_lock
                    .get_component_from_entity::<instance::Instance>(*entity)
                    .unwrap();
                let buffer = ecs_lock
                    .get_component_from_entity::<wgpu::Buffer>(*entity)
                    .unwrap();
                let pos3 = ecs_lock
                    .get_component_from_entity::<components::transforms::Pos3>(*entity)
                    .unwrap();

                {
                    let mut wlock_instance = instance.write().unwrap();
                    let mut rlock_pos3 = pos3.read().unwrap();

                    wlock_instance.position = rlock_pos3.pos;
                    wlock_instance.rotation = rlock_pos3.rot
                }

                let instance_raw = instance.read().unwrap().to_raw();
                self.queue.write_buffer(
                    &buffer.write().unwrap(),
                    0,
                    bytemuck::cast_slice(&[instance_raw]),
                );

                physics_bodies.push((*entity, physics_body, pos3));
            }
        }

        // Update positions and velocities based on acceleration
        for (entity, physics_body, pos3) in &physics_bodies {
            let mut wlock_physics_body = physics_body.write().unwrap();
            let mut wlock_pos3 = pos3.write().unwrap();

            wlock_physics_body.update_pos(&mut wlock_pos3, dt);
        }

        // Check for collisions and resolve them
        for i in 0..physics_bodies.len() {
            for j in (i + 1)..physics_bodies.len() {
                let (_entity_a, physics_body_a, pos3_a) = &physics_bodies[i];
                let (_entity_b, physics_body_b, pos3_b) = &physics_bodies[j];

                let mut wlock_physics_body_a = physics_body_a.write().unwrap();
                let mut wlock_physics_body_b = physics_body_b.write().unwrap();
                let mut wlock_pos3_a = pos3_a.write().unwrap();
                let mut wlock_pos3_b = pos3_b.write().unwrap();

                components::physics::RigidBody::check_and_resolve_collision(
                    &mut wlock_physics_body_a,
                    &mut wlock_pos3_a,
                    &mut wlock_physics_body_b,
                    &mut wlock_pos3_b,
                );
            }
        }
    }

    /// Render the scene. This function is called every frame to render the scene.
    /// It is responsible for rendering the models, the lights, the camera, etc.
    ///
    /// # Returns
    ///
    /// A result indicating if the rendering was successful or not.
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

            // ! Render models if any are present
            if let Some(model_entities) = &self.drawable_entities {
                render_pass.set_model_pipeline(
                    &self.render_pipeline,
                    &self.camera_bind_group,
                    &self.light_bind_group,
                );

                for entity in model_entities {
                    let ecs_lock = self.ecs.lock().unwrap();

                    let model = ecs_lock
                        .get_component_from_entity::<model::Model>(*entity)
                        .unwrap();
                    let instance_buffer = ecs_lock
                        .get_component_from_entity::<wgpu::Buffer>(*entity)
                        .unwrap();

                    let rlock_model = model.read().unwrap();
                    let rlock_instance_buffer = instance_buffer.read().unwrap();

                    render_pass.draw_model(&rlock_model, &rlock_instance_buffer);
                }
            }

            // ! Render collision boxes if enabled
            if self.draw_colliders {
                if let Some(physics_entities) = &self.physics_entities {
                    render_pass.set_wireframe_pipeline(
                        &self.collider_render_pipeline,
                        &self.camera_bind_group,
                    );

                    for entity in physics_entities {
                        let ecs_lock = self.ecs.lock().unwrap();

                        let wireframe = ecs_lock
                            .get_component_from_entity::<model::WireframeMesh>(*entity)
                            .unwrap();
                        // Use the same instance buffer that's used for the physics body
                        let instance_buffer = ecs_lock
                            .get_component_from_entity::<wgpu::Buffer>(*entity)
                            .unwrap();

                        // Lock and read components
                        let wireframe = wireframe.read().unwrap();
                        let instance_buffer = instance_buffer.read().unwrap();

                        render_pass.draw_wireframe_mesh(&wireframe, &instance_buffer);
                    }
                }
            }
        }

        // ! Egui render pass for the custom UI windows
        if (!self.egui_windows.is_empty()) {
            // * if a custom ui is present
            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: self.window.scale_factor() as f32,
            };

            self.egui_renderer.draw_ui_full(
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
