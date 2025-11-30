use super::texture;
use crate::animation::clip::AnimationClip;
use gears_ecs::{
    Component,
    components::{self, physics::AABBCollisionBox},
};
use gears_macro::Component;
use std::fmt::Debug;
use wgpu::util::DeviceExt;

/// Trait for vertex types that can provide GPU buffer layout information.
pub(crate) trait Vertex {
    /// Returns the vertex buffer layout descriptor for the GPU.
    ///
    /// # Returns
    ///
    /// A [`wgpu::VertexBufferLayout`] describing the vertex format.
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

/// Vertex data for model rendering with position, UV, and tangent space basis.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ModelVertex {
    /// Vertex position in model space.
    pub position: [f32; 3],
    /// Texture coordinates.
    pub tex_coords: [f32; 2],
    /// Surface normal direction.
    pub normal: [f32; 3],
    /// Tangent vector parallel to the surface.
    pub tangent: [f32; 3],
    /// Bitangent vector perpendicular to the tangent.
    pub bitangent: [f32; 3],
}

impl Vertex for ModelVertex {
    /// Returns the vertex buffer layout for model vertices.
    ///
    /// # Returns
    ///
    /// A [`wgpu::VertexBufferLayout`] describing the model vertex format.
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
                // Tangent and bitangent
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// Simplified vertex data for wireframe collision box rendering.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ColliderVertex {
    /// Vertex position in model space.
    pub position: [f32; 3],
}

impl Vertex for ColliderVertex {
    /// Returns the vertex buffer layout for collider vertices.
    ///
    /// # Returns
    ///
    /// A [`wgpu::VertexBufferLayout`] describing the collider vertex format.
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

/// Material data containing textures and GPU bind group.
#[derive(Debug)]
pub(crate) struct Material {
    /// Material name for identification.
    #[allow(unused)]
    pub name: String,
    /// Diffuse/albedo texture.
    #[allow(unused)]
    pub diffuse_texture: texture::Texture,
    /// Normal map texture.
    #[allow(unused)]
    pub normal_texture: texture::Texture,
    /// GPU bind group for shader access.
    pub bind_group: wgpu::BindGroup,
}
impl Material {
    /// Creates a new material with the given textures.
    ///
    /// # Arguments
    ///
    /// * `device` - The GPU device for bind group creation.
    /// * `name` - The name of the material.
    /// * `diffuse_texture` - The diffuse texture for the material.
    /// * `normal_texture` - The normal map texture for the material.
    /// * `layout` - The bind group layout to use.
    ///
    /// # Returns
    ///
    /// A new [`Material`] instance.
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

/// A mesh containing geometry and material index.
#[derive(Debug)]
pub(crate) struct Mesh {
    /// Mesh name for identification.
    #[allow(unused)]
    pub name: String,
    /// GPU buffer containing vertex data.
    pub vertex_buffer: wgpu::Buffer,
    /// GPU buffer containing index data.
    pub index_buffer: wgpu::Buffer,
    /// Number of indices to draw.
    pub num_elements: u32,
    /// Index into the model's material array.
    pub material: usize,
}

/// Wireframe mesh for rendering collision boxes.
#[derive(Component, Debug)]
pub(crate) struct WireframeMesh {
    /// GPU buffer containing wireframe vertex data.
    pub vertex_buffer: wgpu::Buffer,
    /// GPU buffer containing wireframe index data.
    pub index_buffer: wgpu::Buffer,
    /// Number of indices in the wireframe.
    pub num_indices: u32,
}

impl WireframeMesh {
    /// Creates a new wireframe mesh from a rigid body's collision box.
    ///
    /// # Arguments
    ///
    /// * `device` - The GPU device for buffer creation.
    /// * `rigid_body` - The rigid body containing the AABB collision box.
    ///
    /// # Returns
    ///
    /// A new [`WireframeMesh`] instance.
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

/// Trait for drawing wireframe meshes with a render pass.
pub(crate) trait DrawWireframeMesh {
    /// Sets up the wireframe rendering pipeline and camera.
    ///
    /// # Arguments
    ///
    /// * `pipeline` - The wireframe render pipeline.
    /// * `camera_bind_group` - The bind group containing camera data.
    fn set_wireframe_pipeline(
        &mut self,
        pipeline: &wgpu::RenderPipeline,
        camera_bind_group: &wgpu::BindGroup,
    );

    /// Draws a wireframe mesh with the given instance buffer.
    ///
    /// # Arguments
    ///
    /// * `mesh` - The wireframe mesh to draw.
    /// * `instance_buffer` - The instance buffer for instanced rendering.
    fn draw_wireframe_mesh(&mut self, mesh: &WireframeMesh, instance_buffer: &wgpu::Buffer);
}

impl DrawWireframeMesh for wgpu::RenderPass<'_> {
    /// Sets up the wireframe rendering pipeline and camera.
    ///
    /// # Arguments
    ///
    /// * `pipeline` - The wireframe render pipeline.
    /// * `camera_bind_group` - The bind group containing camera data.
    fn set_wireframe_pipeline(
        &mut self,
        pipeline: &wgpu::RenderPipeline,
        camera_bind_group: &wgpu::BindGroup,
    ) {
        self.set_pipeline(pipeline);
        self.set_bind_group(0, camera_bind_group, &[]);
    }

    /// Draws a wireframe mesh with the given instance buffer.
    ///
    /// # Arguments
    ///
    /// * `mesh` - The wireframe mesh to draw.
    /// * `instance_buffer` - The instance buffer for instanced rendering.
    fn draw_wireframe_mesh(&mut self, mesh: &WireframeMesh, instance_buffer: &wgpu::Buffer) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_vertex_buffer(1, instance_buffer.slice(..));
        self.draw_indexed(0..mesh.num_indices, 0, 0..1);
    }
}

/// A 3D model containing meshes, materials, and animations.
#[derive(Component)]
pub struct Model {
    /// All meshes in the model.
    pub(crate) meshes: Vec<Mesh>,
    /// All materials used by the model.
    pub(crate) materials: Vec<Material>,
    /// Animation clips associated with the model.
    pub(crate) animations: Vec<AnimationClip>,
}

impl Debug for Model {
    /// Formats the model for debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write to.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("meshes", &self.meshes)
            .field("materials", &self.materials)
            .field("animations", &self.animations)
            .finish()
    }
}

impl Model {
    /// Retrieves an animation clip by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the animation clip to retrieve.
    ///
    /// # Returns
    ///
    /// A result containing a reference to the animation clip or an error message.
    pub fn get_animation(&self, name: &str) -> Result<&AnimationClip, String> {
        self.animations
            .iter()
            .find(|clip| clip.name == name)
            .ok_or_else(|| format!("Animation with name {} not found in model", name))
    }
}

/// Trait for drawing models with a render pass.
pub(crate) trait DrawModelMesh {
    /// Sets up the model rendering pipeline with camera and lights.
    ///
    /// # Arguments
    ///
    /// * `pipeline` - The model render pipeline.
    /// * `camera_bind_group` - The bind group containing camera data.
    /// * `light_bind_group` - The bind group containing light data.
    fn set_model_pipeline(
        &mut self,
        pipeline: &wgpu::RenderPipeline,
        camera_bind_group: &wgpu::BindGroup,
        light_bind_group: &wgpu::BindGroup,
    );

    /// Draws a model with the given instance buffer.
    ///
    /// # Arguments
    ///
    /// * `model` - The model to draw.
    /// * `instance_buffer` - The instance buffer for instanced rendering.
    fn draw_model(&mut self, model: &Model, instance_buffer: &wgpu::Buffer);
}

impl DrawModelMesh for wgpu::RenderPass<'_> {
    /// Sets up the model rendering pipeline with camera and lights.
    ///
    /// # Arguments
    ///
    /// * `pipeline` - The model render pipeline.
    /// * `camera_bind_group` - The bind group containing camera data.
    /// * `light_bind_group` - The bind group containing light data.
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

    /// Draws a model with the given instance buffer.
    ///
    /// # Arguments
    ///
    /// * `model` - The model to draw.
    /// * `instance_buffer` - The instance buffer for instanced rendering.
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
