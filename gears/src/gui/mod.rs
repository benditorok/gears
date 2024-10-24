use egui::Context;
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, StoreOp, TextureFormat, TextureView};
use egui_wgpu::{wgpu, Renderer, ScreenDescriptor};
use egui_winit::State;
use winit::event::WindowEvent;
use winit::window::Window;

/// A wrapper around the egui-wgpu renderer that handles the egui context and renderer.
///
/// This struct is responsible for handling events on the custom windows, and provides
/// methods to interact with the egui context and renderer.
pub struct EguiRenderer {
    state: State,
    renderer: Renderer,
    frame_started: bool,
}

impl EguiRenderer {
    /// Create a new EguiRenderer.
    ///
    /// # Arguments
    ///
    /// * `device` - The wgpu device.
    /// * `output_color_format` - The texture format for the output color.
    /// * `output_depth_format` - The texture format for the output depth.
    /// * `msaa_samples` - The number of samples for multisampling.
    /// * `window` - The window to render to.
    pub fn new(
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
        window: &Window,
    ) -> EguiRenderer {
        let egui_context = Context::default();

        let egui_state = egui_winit::State::new(
            egui_context,
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(2 * 1024), // default dimension is 2048
        );
        let egui_renderer = Renderer::new(
            device,
            output_color_format,
            output_depth_format,
            msaa_samples,
            true,
        );

        EguiRenderer {
            state: egui_state,
            renderer: egui_renderer,
            frame_started: false,
        }
    }

    /// Get a reference to the egui context.
    ///
    /// # Returns
    ///
    /// A reference to the egui context.
    pub fn context(&self) -> &Context {
        self.state.egui_ctx()
    }

    /// Handle input events on the window.
    /// This method should be called when a window event is received.
    /// This method will return true if the event was consumed by the egui context.
    ///
    /// # Arguments
    ///
    /// * `window` - The window that received the event.
    /// * `event` - The event that was received.
    ///
    /// # Returns
    ///
    /// True if the event was consumed by the egui context.
    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let response = self.state.on_window_event(window, event);
        response.consumed
    }

    /// Set the pixels per point for the egui context.
    ///
    /// # Arguments
    ///
    /// * `v` - The pixels per point value.
    pub fn ppp(&mut self, v: f32) {
        self.context().set_pixels_per_point(v);
    }

    /// Begin a new frame.
    ///
    /// # Arguments
    ///
    /// * `window` - The window to render to.
    pub fn begin_frame(&mut self, window: &Window) {
        let raw_input = self.state.take_egui_input(window);
        self.state.egui_ctx().begin_pass(raw_input);
        self.frame_started = true;
    }

    /// End the current frame and draw the egui context to the window.
    /// This method must be called after begin_frame.
    ///
    /// # Arguments
    ///
    /// * `device` - The wgpu device.
    /// * `queue` - The wgpu queue.
    /// * `encoder` - The wgpu command encoder.
    /// * `window` - The window to render to.
    /// * `window_surface_view` - The texture view for the window surface.
    /// * `screen_descriptor` - The screen descriptor for the window.
    ///
    /// # Panics
    ///
    /// This method will panic if begin_frame has not been called before end_frame_and_draw.
    pub fn end_frame_and_draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        window: &Window,
        window_surface_view: &TextureView,
        screen_descriptor: ScreenDescriptor,
    ) {
        if !self.frame_started {
            panic!("begin_frame must be called before end_frame_and_draw can be called!");
        }

        self.ppp(screen_descriptor.pixels_per_point);

        let full_output = self.state.egui_ctx().end_pass();

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .state
            .egui_ctx()
            .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }
        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);
        let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: window_surface_view,
                resolve_target: None,
                ops: egui_wgpu::wgpu::Operations {
                    load: egui_wgpu::wgpu::LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            label: Some("egui main render pass"),
            occlusion_query_set: None,
        });

        self.renderer
            .render(&mut rpass.forget_lifetime(), &tris, &screen_descriptor);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }

        self.frame_started = false;
    }

    /// Draw a custom UI to the window context.
    /// This method will handle the entire UI rendering process.
    ///
    /// # Arguments
    ///
    /// * `device` - The wgpu device.
    /// * `queue` - The wgpu queue.
    /// * `encoder` - The wgpu command encoder.
    /// * `window` - The window to render to.
    /// * `window_surface_view` - The texture view for the window surface.
    /// * `screen_descriptor` - The screen descriptor for the window.
    /// * `run_ui` - A closure that will be called to run the UI.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_ui_full(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        window: &Window,
        window_surface_view: &TextureView,
        screen_descriptor: &ScreenDescriptor,
        run_ui: &mut impl FnMut(&egui::Context),
    ) {
        let raw_input = self.state.take_egui_input(window);
        self.ppp(screen_descriptor.pixels_per_point);

        self.frame_started = true;

        let full_output = self.state.egui_ctx().run(raw_input, |ui| {
            run_ui(ui);
        });

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .state
            .egui_ctx()
            .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }
        self.renderer
            .update_buffers(device, queue, encoder, &tris, screen_descriptor);
        let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: window_surface_view,
                resolve_target: None,
                ops: egui_wgpu::wgpu::Operations {
                    load: egui_wgpu::wgpu::LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            label: Some("egui main render pass"),
            occlusion_query_set: None,
        });

        self.renderer
            .render(&mut rpass.forget_lifetime(), &tris, screen_descriptor);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }

        self.frame_started = false;
    }
}
