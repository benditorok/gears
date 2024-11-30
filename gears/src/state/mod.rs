mod init;
mod update;

use crate::ecs::traits::Marker;
use crate::ecs::{self, components};
use crate::gui::EguiRenderer;
use crate::renderer::model::{self, DrawModelMesh, DrawWireframeMesh, Vertex};
use crate::renderer::{camera, instance, light, resources, texture};
use cgmath::prelude::*;
use egui_wgpu::ScreenDescriptor;
use log::warn;
use std::future::Future;
use std::iter;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{self, Instant};
use wgpu::util::DeviceExt;
use winit::event::*;
use winit::{
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

/// Global state of the application. This is where all rendering related data is stored.
///
/// The State is responsible for handling the rendering pipeline, the camera, the lights,
/// the models, the window, etc.
pub(crate) struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    movement_controller: Option<Arc<RwLock<components::controllers::MovementController>>>,
    pub(crate) view_controller: Option<Arc<RwLock<components::controllers::ViewController>>>,
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
    pub(crate) egui_windows: Vec<Box<dyn FnMut(&egui::Context)>>,
    is_state_paused: AtomicBool,
    time: Instant,
    collider_render_pipeline: wgpu::RenderPipeline,
    target_entities: Option<Vec<ecs::Entity>>,
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
    pub(crate) async fn new(window: &'a Window, ecs: Arc<Mutex<ecs::Manager>>) -> State<'a> {
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
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
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
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/wireframe.wgsl").into()),
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
            target_entities: None,
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

    pub fn is_paused(&self) -> bool {
        self.is_state_paused.load(Ordering::Relaxed)
    }

    /// Initialize the components which can be rendered.
    pub(crate) async fn init_components(&mut self) -> anyhow::Result<()> {
        if !init::player(self) {
            init::camera(self);
        }

        init::targets(self);
        init::lights(self);

        // Load models and handle potential errors
        let model_entities = init::models(
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
            &self.ecs,
        )
        .await;

        let physics_entities = init::physics_models(
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
            &self.ecs,
        )
        .await;

        self.static_model_entities = Some(model_entities.clone());
        self.physics_entities = Some(physics_entities.clone());

        let mut drawable_entities = model_entities;
        drawable_entities.extend(physics_entities);
        self.drawable_entities = Some(drawable_entities);

        Ok(())
    }

    /// Get a reference to the window used by the state.
    ///
    /// # Returns
    ///
    /// A reference to the window.
    pub(crate) fn window(&self) -> &Window {
        self.window
    }

    /// Resize the window when the size changes.
    ///
    /// # Arguments
    ///
    /// * `new_size` - The new size of the window.
    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
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
    pub(crate) fn input(&mut self, event: &WindowEvent) -> bool {
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
    pub(crate) async fn update(&mut self, dt: time::Duration) -> anyhow::Result<()> {
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
        update::update_lights(self);
        update::update_models(self);

        Ok(())
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
                    let rlock_pos3 = pos3.read().unwrap();

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
    pub(crate) fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
        if !self.egui_windows.is_empty() {
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
