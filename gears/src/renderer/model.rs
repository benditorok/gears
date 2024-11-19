use wgpu::util::DeviceExt;

use crate::ecs::components;

use super::texture;
use std::ops::{Div, Range};

pub(crate) trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ColliderVertex {
    pub position: [f32; 3],
    pub dimensions: [f32; 3],
}

impl Vertex for ColliderVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ColliderVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub(crate) struct Material {
    #[allow(unused)]
    pub name: String,
    #[allow(unused)]
    pub diffuse_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

pub(crate) struct Mesh {
    #[allow(unused)]
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

pub(crate) struct WireframeMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl WireframeMesh {
    pub fn new(device: &wgpu::Device, rigid_body: &components::physics::RigidBody) -> Self {
        let collision_box = &rigid_body.collision_box;

        // Define vertices at their actual positions relative to the origin
        let vertices = [
            // Front face corners
            [
                collision_box.min.x,
                collision_box.min.y,
                collision_box.min.z,
            ],
            [
                collision_box.max.x,
                collision_box.min.y,
                collision_box.min.z,
            ],
            [
                collision_box.max.x,
                collision_box.max.y,
                collision_box.min.z,
            ],
            [
                collision_box.min.x,
                collision_box.max.y,
                collision_box.min.z,
            ],
            // Back face corners
            [
                collision_box.min.x,
                collision_box.min.y,
                collision_box.max.z,
            ],
            [
                collision_box.max.x,
                collision_box.min.y,
                collision_box.max.z,
            ],
            [
                collision_box.max.x,
                collision_box.max.y,
                collision_box.max.z,
            ],
            [
                collision_box.min.x,
                collision_box.max.y,
                collision_box.max.z,
            ],
        ];

        // Calculate actual dimensions
        let dimensions = [
            (collision_box.max.x - collision_box.min.x).abs().div(2.0),
            (collision_box.max.y - collision_box.min.y).abs().div(2.0),
            (collision_box.max.z - collision_box.min.z).abs().div(2.0),
        ];

        let vertex_data: Vec<ColliderVertex> = vertices
            .iter()
            .map(|pos| ColliderVertex {
                position: *pos,
                dimensions,
            })
            .collect();

        // Indices for drawing lines between corners (12 lines = 24 indices)
        let indices: Vec<u32> = vec![
            // Front face
            0, 1, 1, 2, 2, 3, 3, 0, // Back face
            4, 5, 5, 6, 6, 7, 7, 4, // Connecting lines
            0, 4, 1, 5, 2, 6, 3, 7,
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Wireframe Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Wireframe Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}

pub(crate) enum Keyframes {
    Translation(Vec<Vec<f32>>),
    Rotation(Vec<Vec<f32>>), // Added Rotation variant
    Scale(Vec<Vec<f32>>),    // Added Scale variant
    Other,
}

pub(crate) struct AnimationClip {
    pub name: String,
    pub keyframes: Keyframes,
    pub timestamps: Vec<f32>,
}

pub(crate) struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub animations: Vec<AnimationClip>,
}

impl Model {
    pub fn get_animation(&self, name: &str) -> anyhow::Result<&AnimationClip> {
        self.animations
            .iter()
            .find(|clip| clip.name == name)
            .ok_or_else(|| anyhow::anyhow!("Animation with name {} not found in model", name))
    }
}

pub(crate) trait DrawModel<'a> {
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_model(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.set_bind_group(2, light_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model(
        &mut self,
        model: &'b Model,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_model_instanced(model, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(
                mesh,
                material,
                instances.clone(),
                camera_bind_group,
                light_bind_group,
            );
        }
    }
}
