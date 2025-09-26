mod init;
mod resources;

use super::model::{self, DrawModelMesh, DrawWireframeMesh, Vertex};
use super::{camera, instance, light, texture};
use crate::{BufferComponent, errors::RendererError};
use egui::mutex::Mutex;
use egui_wgpu::ScreenDescriptor;
use gears_core::config::Config;
use gears_ecs::components::physics::{AABBCollisionBox, CollisionBox, RigidBody};
use gears_ecs::components::transforms::Pos3;
use gears_ecs::query_system::{ComponentQuery, WorldQueryExt};
use gears_ecs::{self, Entity, World, components};
use gears_gui::{EguiRenderer, EguiWindowCallback};
use image::imageops::FilterType::Triangle;
use std::any::Any;
use std::iter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{self};
use wgpu::util::DeviceExt;
use winit::event::*;
use winit::window::CursorGrabMode;
use winit::{
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

/// Global state of the application. This is where all rendering related data is stored.
///
/// The State is responsible for handling the rendering pipeline, the camera, the lights,
/// the models, the window, etc.
pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    pub queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    movement_controller: Option<Arc<RwLock<components::controllers::MovementController>>>,
    pub view_controller: Option<Arc<RwLock<components::controllers::ViewController>>>,
    player_entity: Option<Entity>,
    camera_owner_entity: Option<Entity>,
    camera_projection: camera::Projection,
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    light_entities: Option<Vec<Entity>>,
    pub light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    light_bind_group_layout: wgpu::BindGroupLayout,
    depth_texture: texture::Texture,
    window: Arc<Window>,
    world: Arc<World>,
    mouse_pressed: bool,
    pub draw_colliders: bool,
    egui_renderer: EguiRenderer,
    egui_windows: Arc<Mutex<Vec<EguiWindowCallback>>>,
    is_state_paused: AtomicBool,
    is_cursor_grabbed: AtomicBool,
    collider_render_pipeline: wgpu::RenderPipeline,
    target_entities: Option<Vec<Entity>>,
}

impl State {
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
    pub async fn new(window: Arc<Window>, world: Arc<World>) -> State {
        // * Initializing the backend
        // The instance is a handle to the GPU. BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU.
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
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
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features,
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: Default::default(),
            })
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

        // Choose present mode with vsync (Fifo) as default, fallback to first available
        let present_mode = surface_caps
            .present_modes
            .iter()
            .copied()
            .find(|&mode| mode == wgpu::PresentMode::AutoVsync)
            .unwrap_or(surface_caps.present_modes[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
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
                source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/shader.wgsl").into()),
            };
            resources::create_render_pipeline(
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
        let egui_renderer = EguiRenderer::new(&device, surface_format, None, 1, &window);
        let egui_windows = Arc::new(Mutex::new(Vec::new()));

        // * Wireframe render pipeline
        let collider_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Collider Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Collider Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../shaders/wireframe.wgsl").into(),
                ),
            };

            resources::create_render_pipeline(
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
            light_bind_group_layout,
            depth_texture,
            window,
            world,
            mouse_pressed: false,
            draw_colliders: false,
            egui_renderer,
            egui_windows,
            is_state_paused: AtomicBool::new(false),
            is_cursor_grabbed: AtomicBool::new(false),
            collider_render_pipeline,
            player_entity: None,
            target_entities: None,
        }
    }

    /// Toggle the debug mode.
    pub fn toggle_debug(&mut self) {
        self.draw_colliders = !self.draw_colliders;
    }

    pub fn is_paused(&self) -> bool {
        self.is_state_paused.load(Ordering::Relaxed)
    }

    pub fn add_windows(&mut self, egui_windows: Vec<EguiWindowCallback>) {
        self.egui_windows.lock().extend(egui_windows);
    }

    pub fn grab_cursor(&self) {
        self.window
            .set_cursor_grab(CursorGrabMode::Confined)
            .or_else(|_e| self.window.set_cursor_grab(CursorGrabMode::Locked))
            .ok();
        self.window.set_cursor_visible(false);
    }

    pub fn release_cursor(&self) {
        self.window.set_cursor_grab(CursorGrabMode::None).ok();
        self.window.set_cursor_visible(true);
    }

    pub fn toggle_cursor(&self) {
        if self.is_cursor_grabbed.load(Ordering::Relaxed) {
            self.release_cursor();
            self.is_cursor_grabbed.store(false, Ordering::Relaxed);
        } else {
            self.grab_cursor();
            self.is_cursor_grabbed.store(true, Ordering::Relaxed);
        }
    }

    /// Initialize the components which can be rendered.
    pub async fn init_components(&mut self) -> Result<(), RendererError> {
        if !init::player(self) {
            init::camera(self);
        }

        init::targets(self);
        init::models(
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
            &self.world,
        )
        .await;
        init::physics_models(
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
            &self.world,
        )
        .await;

        Ok(())
    }

    /// Get a reference to the window used by the state.
    ///
    /// # Returns
    ///
    /// A reference to the window.
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Resize the window when the size changes.
    ///
    /// # Arguments
    ///
    /// * `new_size` - The new size of the window.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
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
    pub fn input(&mut self, event: &WindowEvent) -> bool {
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
            let is_running = self.is_state_paused.load(Ordering::Relaxed);
            self.is_state_paused.store(!is_running, Ordering::Relaxed);

            return true;
        }

        // * Capture the input for the custom windows
        if self.egui_renderer.handle_input(&self.window, event) {
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
                match key {
                    KeyCode::AltLeft | KeyCode::AltRight => {
                        // Toggle on release
                        if !state.is_pressed() {
                            self.toggle_cursor();
                        }
                    }
                    KeyCode::F1 => {
                        // Toggle on release
                        if !state.is_pressed() {
                            self.toggle_debug();
                        }
                    }
                    _ => {
                        if let Some(movement_controller) = &self.movement_controller {
                            let mut wlock_movement_controller =
                                movement_controller.write().unwrap();
                            wlock_movement_controller.process_keyboard(*key, *state);
                        }
                    }
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
    pub async fn update(&mut self, dt: time::Duration) -> Result<(), RendererError> {
        // ! Update the camera (view controller). If the camera is a player, then update the movement controller as well.
        if let Some(view_controller) = &self.view_controller {
            let camera_entity = self.camera_owner_entity.unwrap();

            // Build query for camera components
            let mut query = ComponentQuery::new().write::<Pos3>(vec![camera_entity]);

            // Check if we also need rigidbody access
            let has_rigidbody = self
                .world
                .get_entities_with_component::<RigidBody<AABBCollisionBox>>()
                .contains(&camera_entity);

            if has_rigidbody {
                query = query.write::<RigidBody<AABBCollisionBox>>(vec![camera_entity]);
            }

            if let Some(resources) = self.world.acquire_query(query) {
                if let Some(pos3_comp) = resources.get::<Pos3>(camera_entity) {
                    let mut wlock_pos3 = pos3_comp.write().unwrap();
                    let mut wlock_view_controller = view_controller.write().unwrap();
                    wlock_view_controller.update_rot(&mut wlock_pos3, dt.as_secs_f32());

                    if let Some(movement_controller) = &self.movement_controller {
                        let mut wlock_movement_controller = movement_controller.write().unwrap();

                        if has_rigidbody {
                            if let Some(rigid_body_comp) =
                                resources.get::<RigidBody<AABBCollisionBox>>(camera_entity)
                            {
                                let mut wlock_rigid_body = rigid_body_comp.write().unwrap();
                                wlock_movement_controller.update_pos(
                                    &wlock_view_controller,
                                    &mut wlock_pos3,
                                    Some(&mut wlock_rigid_body),
                                    dt.as_secs_f32(),
                                );
                            }
                        } else {
                            wlock_movement_controller.update_pos(
                                &wlock_view_controller,
                                &mut wlock_pos3,
                                None::<
                                    &mut gears_ecs::components::physics::RigidBody<
                                        AABBCollisionBox,
                                    >,
                                >,
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
            }
        }

        Ok(())
    }

    /// Render the scene. This function is called every frame to render the scene.
    /// It is responsible for rendering the models, the lights, the camera, etc.
    ///
    /// # Returns
    ///
    /// A result indicating if the rendering was successful or not.
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
            let models = self.world.get_entities_with_component::<model::Model>();
            if !models.is_empty() {
                render_pass.set_model_pipeline(
                    &self.render_pipeline,
                    &self.camera_bind_group,
                    &self.light_bind_group,
                );

                for entity in models {
                    let query = ComponentQuery::new()
                        .read::<model::Model>(vec![entity])
                        .read::<BufferComponent>(vec![entity]);

                    if let Some(resources) = self.world.acquire_query(query) {
                        if let (Some(model_comp), Some(buffer_comp)) = (
                            resources.get::<model::Model>(entity),
                            resources.get::<BufferComponent>(entity),
                        ) {
                            let rlock_model = model_comp.read().unwrap();
                            let rlock_instance_buffer = buffer_comp.read().unwrap();

                            render_pass.draw_model(&rlock_model, &rlock_instance_buffer);
                        }
                    }
                }
            }

            // ! Render collision boxes if enabled
            if self.draw_colliders {
                let wireframes = self
                    .world
                    .get_entities_with_component::<model::WireframeMesh>();

                if !wireframes.is_empty() {
                    render_pass.set_wireframe_pipeline(
                        &self.collider_render_pipeline,
                        &self.camera_bind_group,
                    );

                    for entity in wireframes {
                        let query = ComponentQuery::new()
                            .read::<model::WireframeMesh>(vec![entity])
                            .read::<BufferComponent>(vec![entity]);

                        if let Some(resources) = self.world.acquire_query(query) {
                            if let (Some(wireframe_comp), Some(buffer_comp)) = (
                                resources.get::<model::WireframeMesh>(entity),
                                resources.get::<BufferComponent>(entity),
                            ) {
                                let wireframe = wireframe_comp.read().unwrap();
                                let instance_buffer = buffer_comp.read().unwrap();

                                render_pass.draw_wireframe_mesh(&wireframe, &instance_buffer);
                            }
                        }
                    }
                }
            }
        }

        // ! Egui render pass for the custom UI windows
        let mut egui_windows = self.egui_windows.lock();
        if !egui_windows.is_empty() {
            // * if a custom ui is present
            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: self.window.scale_factor() as f32,
            };

            self.egui_renderer.draw_ui_full(
                &self.device,
                &self.queue,
                &mut encoder,
                &self.window,
                &view,
                &screen_descriptor,
                &mut egui_windows,
            );
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
