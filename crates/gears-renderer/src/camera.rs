use crate::{ecs::components, OPENGL_TO_WGPU_MATRIX};
use cgmath::{perspective, InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Vector3};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CameraUniform {
    pub view_pos: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_pos: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

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

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}
