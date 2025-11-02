use cgmath::{InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Vector3, perspective};
use gears_core::OPENGL_TO_WGPU_MATRIX;
use gears_ecs::components;

/// Uniform data sent to the GPU for camera transformations.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CameraUniform {
    /// The camera position in world space.
    pub view_pos: [f32; 4],
    /// The combined view-projection matrix.
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    /// Creates a new camera uniform with identity matrices.
    ///
    /// # Returns
    ///
    /// A new [`CameraUniform`] instance.
    pub fn new() -> Self {
        Self {
            view_pos: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    /// Updates the view-projection matrix from camera components.
    ///
    /// # Arguments
    ///
    /// * `pos3` - The position component of the camera.
    /// * `controller` - The view controller component of the camera.
    /// * `projection` - The projection settings to use.
    pub fn update_view_proj(
        &mut self,
        pos3: &components::transforms::Pos3,
        controller: &components::controllers::ViewController,
        projection: &Projection,
    ) {
        self.view_pos = [pos3.pos.x, pos3.pos.y, pos3.pos.z, 1.0];
        self.view_proj =
            (projection.calc_matrix() * CameraUniform::calc_matrix(controller, pos3)).into();
    }

    /// Calculates the view matrix from controller and position data.
    ///
    /// # Arguments
    ///
    /// * `controller` - The view controller component.
    /// * `pos3` - The position component.
    ///
    /// # Returns
    ///
    /// The calculated 4x4 view matrix.
    fn calc_matrix(
        controller: &components::controllers::ViewController,
        pos3: &components::transforms::Pos3,
    ) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = controller.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = controller.yaw.0.sin_cos();

        // Add the head offset to the position only for the view calculation
        let view_position =
            Point3::new(pos3.pos.x, pos3.pos.y + controller.head_offset, pos3.pos.z);

        Matrix4::look_to_rh(
            view_position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }
}

/// Projection settings for the camera.
pub struct Projection {
    /// The aspect ratio (width / height).
    aspect: f32,
    /// The field of view in the y direction.
    fovy: Rad<f32>,
    /// The near clipping plane distance.
    znear: f32,
    /// The far clipping plane distance.
    zfar: f32,
}

impl Projection {
    /// Creates a new projection with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `width` - The viewport width in pixels.
    /// * `height` - The viewport height in pixels.
    /// * `fovy` - The field of view angle in the y direction.
    /// * `znear` - The near clipping plane distance.
    /// * `zfar` - The far clipping plane distance.
    ///
    /// # Returns
    ///
    /// A new [`Projection`] instance.
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    /// Updates the aspect ratio when the viewport is resized.
    ///
    /// # Arguments
    ///
    /// * `width` - The new viewport width in pixels.
    /// * `height` - The new viewport height in pixels.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    /// Calculates the projection matrix.
    ///
    /// # Returns
    ///
    /// The calculated 4x4 projection matrix.
    fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}
