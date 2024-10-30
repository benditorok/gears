use super::{model, texture};
use anyhow::Context;
use gltf::Gltf;
use image::GenericImageView;
use log::{info, warn};
use std::fmt::format;
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};
use wgpu::util::DeviceExt;

pub(crate) async fn load_string(file_path: &str) -> anyhow::Result<String> {
    let path = std::path::Path::new(env!("OUT_DIR")).join(file_path);
    let txt = std::fs::read_to_string(&path).context(format!(
        "Failed to read file to string: {}",
        &path.display()
    ))?;

    Ok(txt)
}

pub(crate) async fn load_string_path(path: PathBuf) -> anyhow::Result<String> {
    let txt = std::fs::read_to_string(&path).context(format!(
        "Failed to read file to string: {}",
        &path.display()
    ))?;

    Ok(txt)
}

pub(crate) async fn load_binary(file_path: &str) -> anyhow::Result<Vec<u8>> {
    let path = std::path::Path::new(env!("OUT_DIR")).join(file_path);
    let data = std::fs::read(&path).context(format!(
        "Failed to read file to binary: {}",
        &path.display()
    ))?;

    Ok(data)
}

pub(crate) async fn load_binary_path(path: PathBuf) -> anyhow::Result<Vec<u8>> {
    let data = std::fs::read(&path).context(format!(
        "Failed to read file to binary: {}",
        &path.display()
    ))?;

    Ok(data)
}

pub(crate) async fn load_texture(
    file_path: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(file_path).await?;

    texture::Texture::from_bytes(device, queue, &data, file_path)
}

pub(crate) async fn load_texture_path(
    path: PathBuf,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary_path(path.clone()).await?;

    texture::Texture::from_bytes(
        device,
        queue,
        &data,
        path.file_name().unwrap().to_str().unwrap(),
    )
}

// TODO ! use the example from the tobj crate's documentation
pub(crate) async fn load_model_obj(
    file_path: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let path = Path::new(file_path);
    let model_root_dir = path.parent().unwrap();
    let file_name = model_root_dir.file_name().unwrap().to_str().unwrap();

    let obj_text = load_string(file_path).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(model_root_dir.join(&p).to_str().unwrap())
                .await
                .unwrap_or_default();

            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        let diffuse_texture = load_texture(
            model_root_dir
                .join(m.diffuse_texture.as_ref().unwrap())
                .to_str()
                .unwrap(),
            device,
            queue,
        )
        .await?;
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
            ],
            label: None,
        });

        materials.push(model::Material {
            name: m.name,
            diffuse_texture,
            bind_group,
        })
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    if m.mesh.normals.is_empty() {
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [
                                m.mesh.texcoords[i * 2],
                                1.0 - m.mesh.texcoords[i * 2 + 1],
                            ],
                            normal: [0.0, 0.0, 0.0],
                        }
                    } else {
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3],
                                m.mesh.positions[i * 3 + 1],
                                m.mesh.positions[i * 3 + 2],
                            ],
                            tex_coords: [
                                m.mesh.texcoords[i * 2],
                                1.0 - m.mesh.texcoords[i * 2 + 1],
                            ],
                            normal: [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ],
                        }
                    }
                })
                .collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            log::info!("Mesh: {}", m.name);
            model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model {
        meshes,
        materials,
        animations: Vec::new(),
    })
}

pub(crate) async fn load_model_gltf(
    gltf_path: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let file_path = Path::new(gltf_path);
    let model_root_dir = file_path.parent().unwrap();
    let file_name = model_root_dir.file_name().unwrap().to_str().unwrap();

    let string_path = Path::new(env!("OUT_DIR")).join(gltf_path);
    let gltf_text = load_string_path(string_path).await?;
    let gltf_cursor = Cursor::new(gltf_text);
    let gltf_reader = BufReader::new(gltf_cursor);
    let gltf = Gltf::from_reader(gltf_reader)?;

    // Load buffers
    let mut buffer_data = Vec::new();
    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                // if let Some(blob) = gltf.blob.as_deref() {
                //     buffer_data.push(blob.into());
                //     println!("Found a bin, saving");
                // };
            }
            gltf::buffer::Source::Uri(uri) => {
                let uri_path = Path::new(env!("OUT_DIR")).join(model_root_dir).join(uri);
                let bin = load_binary_path(uri_path).await?;
                buffer_data.push(bin);
            }
        }
    }

    // Load animations
    let mut animation_clips = Vec::new();
    for animation in gltf.animations() {
        for channel in animation.channels() {
            let reader = channel.reader(|buffer| Some(&buffer_data[buffer.index()]));
            let timestamps = if let Some(inputs) = reader.read_inputs() {
                match inputs {
                    gltf::accessor::Iter::Standard(times) => {
                        let times: Vec<f32> = times.collect();
                        info!("Times: {:?}", &times);
                        times
                    }
                    gltf::accessor::Iter::Sparse(_) => {
                        warn!("Sparse keyframes not supported");
                        Vec::new()
                    }
                }
            } else {
                Vec::new()
            };

            let keyframes = if let Some(outputs) = reader.read_outputs() {
                match outputs {
                    gltf::animation::util::ReadOutputs::Translations(translation) => {
                        let translation_vec =
                            translation.map(|tr| tr.into()).collect::<Vec<Vec<f32>>>();
                        model::Keyframes::Translation(translation_vec)
                    }
                    gltf::animation::util::ReadOutputs::Rotations(rotation) => {
                        let rotation_vec = rotation
                            .into_f32()
                            .map(|rot| rot.into())
                            .collect::<Vec<Vec<f32>>>();
                        model::Keyframes::Rotation(rotation_vec)
                    }
                    gltf::animation::util::ReadOutputs::Scales(scale) => {
                        let scale_vec = scale.map(|s| s.into()).collect::<Vec<Vec<f32>>>();
                        model::Keyframes::Scale(scale_vec)
                    }
                    _ => model::Keyframes::Other,
                }
            } else {
                model::Keyframes::Other
            };

            animation_clips.push(model::AnimationClip {
                name: animation.name().unwrap_or("Default").to_string(),
                keyframes,
                timestamps,
            });
        }
    }

    // Load materials
    let mut materials = Vec::new();
    for material in gltf.materials() {
        println!("Looping thru materials");
        let pbr = material.pbr_metallic_roughness();
        let base_color_texture = &pbr.base_color_texture();
        let texture_source = &pbr
            .base_color_texture()
            .map(|tex| {
                // println!("Grabbing diffuse tex");
                // dbg!(&tex.texture().source());
                tex.texture().source().source()
            })
            .expect("texture");

        match texture_source {
            gltf::image::Source::View { view, mime_type } => {
                let diffuse_texture = texture::Texture::from_bytes(
                    device,
                    queue,
                    &buffer_data[view.buffer().index()],
                    file_name,
                )
                .expect("Couldn't load diffuse");
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
                    ],
                    label: None,
                });

                materials.push(model::Material {
                    name: material.name().unwrap_or("Default Material").to_string(),
                    diffuse_texture,
                    bind_group,
                });
            }
            gltf::image::Source::Uri { uri, mime_type } => {
                let uri_path = Path::new(env!("OUT_DIR")).join(model_root_dir).join(uri);
                let diffuse_texture = load_texture_path(uri_path, device, queue).await?;
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
                    ],
                    label: None,
                });

                materials.push(model::Material {
                    name: material.name().unwrap_or("Default Material").to_string(),
                    diffuse_texture,
                    bind_group,
                });
            }
        };
    }

    let mut meshes = Vec::new();

    for scene in gltf.scenes() {
        for node in scene.nodes() {
            println!("Node {}", node.index());
            // dbg!(node);

            let mesh = node.mesh().expect("Got mesh");
            let primitives = mesh.primitives();
            primitives.for_each(|primitive| {
                // dbg!(primitive);

                let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

                let mut vertices = Vec::new();
                if let Some(vertex_attribute) = reader.read_positions() {
                    vertex_attribute.for_each(|vertex| {
                        // dbg!(vertex);
                        vertices.push(model::ModelVertex {
                            position: vertex,
                            tex_coords: Default::default(),
                            normal: Default::default(),
                        })
                    });
                }
                if let Some(normal_attribute) = reader.read_normals() {
                    let mut normal_index = 0;
                    normal_attribute.for_each(|normal| {
                        // dbg!(normal);
                        vertices[normal_index].normal = normal;

                        normal_index += 1;
                    });
                }
                if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
                    let mut tex_coord_index = 0;
                    tex_coord_attribute.for_each(|tex_coord| {
                        // dbg!(tex_coord);
                        vertices[tex_coord_index].tex_coords = tex_coord;

                        tex_coord_index += 1;
                    });
                }

                let mut indices = Vec::new();
                if let Some(indices_raw) = reader.read_indices() {
                    // dbg!(indices_raw);
                    indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
                }
                // dbg!(indices);

                // println!("{:#?}", &indices.expect("got indices").data_type());
                // println!("{:#?}", &indices.expect("got indices").index());
                // println!("{:#?}", &material);

                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Vertex Buffer", file_name)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Index Buffer", file_name)),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

                meshes.push(model::Mesh {
                    name: file_name.to_string(),
                    vertex_buffer,
                    index_buffer,
                    num_elements: indices.len() as u32,
                    // material: m.mesh.material_id.unwrap_or(0),
                    material: 0,
                });
            });
        }
    }

    Ok(model::Model {
        meshes,
        materials,
        animations: animation_clips,
    })
}
