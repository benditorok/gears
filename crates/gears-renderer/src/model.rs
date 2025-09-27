use std::fmt::Debug;

use super::texture;
use gears_ecs::{
    Component,
    components::{self, physics::AABBCollisionBox},
};
use gears_macro::Component;
use wgpu::util::DeviceExt;

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
}

impl Vertex for ColliderVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ColliderVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

#[derive(Debug)]
pub(crate) struct Material {
    #[allow(unused)]
    pub name: String,
    #[allow(unused)]
    pub diffuse_texture: texture::Texture,
    #[allow(unused)]
    pub normal_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}
impl Material {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        diffuse_texture: texture::Texture,
        normal_texture: texture::Texture,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
            label: Some(name),
        });

        Self {
            name: String::from(name),
            diffuse_texture,
            normal_texture,
            bind_group,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Mesh {
    #[allow(unused)]
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

#[derive(Component, Debug)]
pub(crate) struct WireframeMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl WireframeMesh {
    pub fn new(
        device: &wgpu::Device,
        rigid_body: &components::physics::RigidBody<AABBCollisionBox>,
    ) -> Self {
        let collision_box = &rigid_body.collision_box;
        // New indices to include diagonals of each face and through the cube
        let indices: Vec<u32> = vec![
            // Front face edges
            0, 1, 1, 2, 2, 3, 3, 0, // Back face edges
            4, 5, 5, 6, 6, 7, 7, 4, // Connecting edges
            0, 4, 1, 5, 2, 6, 3, 7, // Front face diagonal
            0, 2, // Back face diagonal
            4, 6, // Top face diagonal
            2, 7, // Bottom face diagonal
            0, 5, // Left face diagonal
            0, 7, // Right face diagonal
            1, 6, // Front face edges
        ];

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

        // Pass position without computing dimensions - the shader will use the positions directly
        let vertex_data: Vec<ColliderVertex> = vertices
            .iter()
            .map(|pos| ColliderVertex { position: *pos })
            .collect();

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

pub(crate) trait DrawWireframeMesh {
    fn set_wireframe_pipeline(
        &mut self,
        pipeline: &wgpu::RenderPipeline,
        camera_bind_group: &wgpu::BindGroup,
    );

    fn draw_wireframe_mesh(&mut self, mesh: &WireframeMesh, instance_buffer: &wgpu::Buffer);
}

impl DrawWireframeMesh for wgpu::RenderPass<'_> {
    fn set_wireframe_pipeline(
        &mut self,
        pipeline: &wgpu::RenderPipeline,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        self.set_pipeline(pipeline);
        self.set_bind_group(0, camera_bind_group, &[]);
    }

    fn draw_wireframe_mesh(&mut self, mesh: &WireframeMesh, instance_buffer: &wgpu::Buffer) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_vertex_buffer(1, instance_buffer.slice(..));
        self.draw_indexed(0..mesh.num_indices, 0, 0..1);
    }
}

#[derive(Component)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub animations: Vec<super::animation::AnimationClip>,
}

impl Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("meshes", &self.meshes)
            .field("materials", &self.materials)
            .field("animations", &self.animations)
            .finish()
    }
}

impl Model {
    pub fn get_animation(&self, name: &str) -> Result<&super::animation::AnimationClip, String> {
        self.animations
            .iter()
            .find(|clip| clip.name == name)
            .ok_or_else(|| format!("Animation with name {} not found in model", name))
    }
}

pub(crate) trait DrawModelMesh {
    fn set_model_pipeline(
        &mut self,
        pipeline: &wgpu::RenderPipeline,
        camera_bind_group: &wgpu::BindGroup,
        light_bind_group: &wgpu::BindGroup,
    );

    fn draw_model(&mut self, model: &Model, instance_buffer: &wgpu::Buffer);
}

// TODO lifetime??
impl DrawModelMesh for wgpu::RenderPass<'_> {
    fn set_model_pipeline(
        &mut self,
        pipeline: &wgpu::RenderPipeline,
        camera_bind_group: &wgpu::BindGroup,
        light_bind_group: &wgpu::BindGroup,
    ) {
        self.set_pipeline(pipeline);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.set_bind_group(2, light_bind_group, &[]);
    }

    fn draw_model(&mut self, model: &Model, instance_buffer: &wgpu::Buffer) {
        self.set_vertex_buffer(1, instance_buffer.slice(..));

        for mesh in &model.meshes {
            self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            self.set_bind_group(0, &model.materials[mesh.material].bind_group, &[]);
            self.draw_indexed(0..mesh.num_elements, 0, 0..1);
        }
    }
}
