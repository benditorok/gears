//! Renderer state management and rendering loop.

mod init;
mod pipeline;
mod resources;

use super::instance;
use super::model::{self, DrawModelMesh, DrawWireframeMesh};
use crate::{BufferComponent, errors::RendererError};
use egui::mutex::Mutex;
use egui_wgpu::ScreenDescriptor;
use gears_ecs::components::physics::{AABBCollisionBox, RigidBody};
use gears_ecs::components::transforms::Pos3;
use gears_ecs::query::{ComponentQuery, WorldQueryExt};
use gears_ecs::{self, Entity, World, components};
use gears_gui::{EguiRenderer, EguiWindowCallback};
use std::iter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{self};
use winit::dpi::PhysicalSize;
use winit::event::*;
use winit::window::CursorGrabMode;
use winit::{
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

/// Global state of the application containing all rendering data.
/// Responsible for managing pipelines, camera, lights, models, and the window.
pub struct State {
    /// The GPU surface for rendering output.
    surface: wgpu::Surface<'static>,
    /// The GPU device for resource creation.
    device: wgpu::Device,
    /// The GPU command queue.
    queue: wgpu::Queue,
    /// The surface configuration settings.
    config: wgpu::SurfaceConfiguration,
    /// The window size in pixels.
    size: winit::dpi::PhysicalSize<u32>,
    /// The base rendering pipeline for models.
    base_pipeline: pipeline::base::BasePipeline,
    /// The HDR post-processing pipeline.
    hdr_pipeline: pipeline::hdr::HdrPipeline,
    /// The wireframe rendering pipeline for debug visualization.
    wireframe_pipeline: pipeline::wireframe::WireframePipeline,
    /// The crosshair overlay pipeline.
    crosshair_pipeline: pipeline::crosshair::CrosshairPipeline,
    /// Optional movement controller for the active camera.
    movement_controller: Option<Arc<RwLock<components::controllers::MovementController>>>,
    /// View controller for the active camera.
    view_controller: Option<Arc<RwLock<components::controllers::ViewController>>>,
    /// Optional player entity reference.
    player_entity: Option<Entity>,
    /// Entity that owns the active camera.
    camera_owner_entity: Option<Entity>,
    /// The window being rendered to.
    window: Arc<Window>,
    /// The ECS world containing all entities and components.
    world: Arc<World>,
    /// Whether the mouse button is currently pressed.
    mouse_pressed: bool,
    /// Whether to draw collision box wireframes.
    pub draw_colliders: bool,
    /// The egui renderer for UI elements.
    egui_renderer: EguiRenderer,
    /// Registered egui window callbacks.
    egui_windows: Arc<Mutex<Vec<EguiWindowCallback>>>,
    /// Whether the state is paused.
    is_state_paused: AtomicBool,
    /// Whether the cursor is grabbed.
    is_cursor_grabbed: AtomicBool,
    /// Optional list of target entities for gameplay.
    target_entities: Option<Vec<Entity>>,
}

impl State {
    /// Creates a new renderer state instance.
    ///
    /// # Arguments
    ///
    /// * `window` - The window to render to.
    /// * `ecs` - The ECS manager.
    ///
    /// # Returns
    ///
    /// A new [`State`] instance.
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

        // * Initializing the egui renderer
        let egui_renderer = EguiRenderer::new(&device, surface_format, None, 1, &window);
        let egui_windows = Arc::new(Mutex::new(Vec::new()));

        // ! HDR Pipeline
        let hdr_pipeline = pipeline::hdr::HdrPipeline::new(&device, &config);

        // ! Base pipeline
        let base_pipeline =
            pipeline::base::BasePipeline::new(&device, &config, hdr_pipeline.format());

        // ! Wireframe pipeline
        let wireframe_pipeline = pipeline::wireframe::WireframePipeline::new(
            &device,
            &config,
            &base_pipeline.camera_layout(),
            &hdr_pipeline,
        );

        // ! Crosshair pipeline
        let crosshair_pipeline = pipeline::crosshair::CrosshairPipeline::new(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            base_pipeline,
            hdr_pipeline,
            wireframe_pipeline,
            crosshair_pipeline,
            movement_controller: None,
            view_controller: None,
            camera_owner_entity: None,
            window,
            world,
            mouse_pressed: false,
            draw_colliders: false,
            egui_renderer,
            egui_windows,
            is_state_paused: AtomicBool::new(false),
            is_cursor_grabbed: AtomicBool::new(false),
            player_entity: None,
            target_entities: None,
        }
    }

    /// Exposes the base pipeline.
    ///
    /// # Returns
    ///
    /// A reference to the base pipeline.
    pub fn base_pipeline(&self) -> &pipeline::base::BasePipeline {
        &self.base_pipeline
    }

    /// Exposes the GPU device.
    ///
    /// # Returns
    ///
    /// A reference to the GPU device.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Exposes the view controller.
    ///
    /// # Returns
    ///
    /// An optional reference to the view controller.
    pub fn view_controller(&self) -> Option<&Arc<RwLock<components::controllers::ViewController>>> {
        self.view_controller.as_ref()
    }

    /// Sets the view controller.
    ///
    /// # Arguments
    ///
    /// * `view_controller` - The new view controller to set.
    pub fn set_view_controller(
        &mut self,
        view_controller: Option<Arc<RwLock<components::controllers::ViewController>>>,
    ) {
        self.view_controller = view_controller;
    }

    /// Exposes the movement controller.
    ///
    /// # Returns
    ///
    /// An optional reference to the movement controller.
    pub fn movement_controller(
        &self,
    ) -> Option<&Arc<RwLock<components::controllers::MovementController>>> {
        self.movement_controller.as_ref()
    }

    /// Sets the movement controller.
    ///
    /// # Arguments
    ///
    /// * `movement_controller` - The new movement controller to set.
    pub fn set_movement_controller(
        &mut self,
        movement_controller: Option<Arc<RwLock<components::controllers::MovementController>>>,
    ) {
        self.movement_controller = movement_controller;
    }

    /// Exposes the current window size.
    ///
    /// # Returns
    ///
    /// A reference to the physical size of the window.
    pub fn size(&self) -> &PhysicalSize<u32> {
        &self.size
    }

    /// Sets the debug mode for rendering.
    ///
    /// # Arguments
    ///
    /// * `debug` - Whether to enable debug mode.
    pub fn set_debug(&mut self, debug: bool) {
        self.draw_colliders = debug;
    }

    /// Toggle the debug mode.
    pub fn toggle_debug(&mut self) {
        self.draw_colliders = !self.draw_colliders;
    }

    /// Toggles the crosshair visibility.
    pub fn toggle_crosshair(&mut self) {
        self.crosshair_pipeline.toggle_visible();
    }

    /// Sets the crosshair visibility.
    ///
    /// # Arguments
    ///
    /// * `visible` - Whether the crosshair should be visible.
    pub fn set_crosshair_visible(&mut self, visible: bool) {
        self.crosshair_pipeline.set_visible(visible);
    }

    /// Configures the crosshair appearance.
    ///
    /// # Arguments
    ///
    /// * `gap` - Distance from center to crosshair lines.
    /// * `length` - Length of each crosshair line.
    /// * `thickness` - Thickness of the crosshair lines.
    /// * `color` - RGBA color of the crosshair.
    pub fn configure_crosshair(&self, gap: f32, length: f32, thickness: f32, color: [f32; 4]) {
        self.crosshair_pipeline.update_uniforms(
            &self.queue,
            self.config.width,
            self.config.height,
            gap,
            length,
            thickness,
            color,
        );
    }

    /// Check if the mouse button is currently pressed.
    ///
    /// # Returns
    ///
    /// `true` if the left mouse button is pressed, `false` otherwise.
    pub fn is_mouse_pressed(&self) -> bool {
        self.mouse_pressed
    }

    /// Check if the state is paused.
    ///
    /// # Returns
    ///
    /// `true` if the state is paused.
    pub fn is_paused(&self) -> bool {
        self.is_state_paused.load(Ordering::Relaxed)
    }

    /// Add custom egui windows to be rendered.
    ///
    /// # Arguments
    ///
    /// * `egui_windows` - A vector of egui window callbacks to add.
    pub fn add_windows(&mut self, egui_windows: Vec<EguiWindowCallback>) {
        self.egui_windows.lock().extend(egui_windows);
    }

    /// Grab the cursor for first-person camera control.
    pub fn grab_cursor(&self) {
        self.window
            .set_cursor_grab(CursorGrabMode::Confined)
            .or_else(|_e| self.window.set_cursor_grab(CursorGrabMode::Locked))
            .ok();
        self.window.set_cursor_visible(false);
    }

    /// Release the cursor to allow normal interaction.
    pub fn release_cursor(&self) {
        self.window.set_cursor_grab(CursorGrabMode::None).ok();
        self.window.set_cursor_visible(true);
    }

    /// Toggle the cursor between grabbed and released states.
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
    ///
    /// # Returns
    ///
    /// A result indicating if the initialization was successful or not.
    pub async fn init_components(&mut self) -> Result<(), RendererError> {
        if !init::player(self) {
            init::camera(self);
        }

        init::targets(self);
        init::models(
            &self.device,
            &self.queue,
            &self.base_pipeline.texture_layout(),
            &self.world,
        )
        .await;
        init::physics_models(
            &self.device,
            &self.queue,
            &self.base_pipeline.texture_layout(),
            &self.world,
        )
        .await;

        Ok(())
    }

    /// Get the player entity if it exists.
    ///
    /// # Returns
    ///
    /// An optional reference to the player entity.
    pub fn player_entity(&self) -> Option<Entity> {
        self.player_entity
    }

    /// Get a reference to the window used by the state.
    ///
    /// # Returns
    ///
    /// A reference to the window.
    pub fn window(&self) -> &Window {
        &self.window
    }

    // Reconfigure the surface with the same size.
    pub fn resize_self(&mut self) {
        self.resize(self.size);
    }

    /// Resize the window when the size changes.
    ///
    /// # Arguments
    ///
    /// * `new_size` - The new size of the window.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.base_pipeline
            .camera_projection_mut()
            .resize(new_size.width, new_size.height);

        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.size = new_size;
            //self.camera.aspect = self.config.width as f32 / self.config.height as f32;
            self.surface.configure(&self.device, &self.config);

            self.base_pipeline
                .resize(&self.device, new_size.width, new_size.height);
            self.hdr_pipeline
                .resize(&self.device, new_size.width, new_size.height);
            self.crosshair_pipeline
                .resize(&self.queue, new_size.width, new_size.height);
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
    /// A result indicating if the update was successful or not.
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

                    // Update the camera's view
                    self.base_pipeline
                        .update_camera_view_proj(&wlock_pos3, &wlock_view_controller);

                    self.queue.write_buffer(
                        &self.base_pipeline.camera_buffer(),
                        0,
                        bytemuck::cast_slice(&[self.base_pipeline.camera_uniform().clone()]),
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
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.config.format.add_srgb_suffix()),
            ..Default::default()
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("State::render_encoder"),
            });

        // ! Graphical render pass
        {
            // Run the render pass
            let mut render_pass = self
                .base_pipeline
                .begin(&mut encoder, &self.hdr_pipeline.view());

            // ! Render models if any are present
            let models = self.world.get_entities_with_component::<model::Model>();
            if !models.is_empty() {
                render_pass.set_model_pipeline(
                    &self.base_pipeline.pipeline(),
                    &self.base_pipeline.camera_bind_group(),
                    &self.base_pipeline.light_bind_group(),
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
                        &self.wireframe_pipeline.pipeline(),
                        &self.base_pipeline.camera_bind_group(),
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

        // ! Apply tonemapping
        self.hdr_pipeline.process(&mut encoder, &view);

        // ! Render crosshair overlay
        self.crosshair_pipeline.render(&mut encoder, &view);

        // ! Egui render pass for the custom UI windows
        let mut egui_windows = self.egui_windows.lock();
        if !egui_windows.is_empty() {
            // * If a custom ui is present
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
