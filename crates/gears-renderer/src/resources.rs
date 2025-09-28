use crate::errors::RendererError;

use super::{animation, model, texture};
use cgmath::InnerSpace;
use gltf::Gltf;
use log::{info, warn};
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};
use wgpu::util::DeviceExt;

fn get_resource_path(file_path: &str) -> PathBuf {
    Path::new(env!("RES_DIR")).join(file_path)
}

pub(crate) async fn load_string(file_path: &str) -> Result<String, RendererError> {
    let path = get_resource_path(file_path);
    let txt = std::fs::read_to_string(&path).map_err(|_err| {
        RendererError::ResourceLoadingFailed(format!("Failed to read file: {}", path.display()))
    })?;

    Ok(txt)
}

pub(crate) async fn load_string_path(path: PathBuf) -> Result<String, RendererError> {
    let txt = std::fs::read_to_string(&path).map_err(|_err| {
        RendererError::ResourceLoadingFailed(format!(
            "Failed to read file to string: {}",
            path.display()
        ))
    })?;

    Ok(txt)
}

pub(crate) async fn load_binary(file_path: &str) -> Result<Vec<u8>, RendererError> {
    let path = get_resource_path(file_path);
    let data = std::fs::read(&path).map_err(|_err| {
        RendererError::ResourceLoadingFailed(format!(
            "Failed to read file to binary: {}",
            path.display()
        ))
    })?;

    Ok(data)
}

pub(crate) async fn load_binary_path(path: PathBuf) -> Result<Vec<u8>, RendererError> {
    let data = std::fs::read(&path).map_err(|_err| {
        RendererError::ResourceLoadingFailed(format!(
            "Failed to read file to binary: {}",
            path.display()
        ))
    })?;

    Ok(data)
}

pub(crate) async fn load_texture(
    file_path: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    is_normal_map: bool,
) -> Result<texture::Texture, RendererError> {
    let data = load_binary(file_path).await?;

    texture::Texture::from_bytes(device, queue, &data, file_path, is_normal_map)
}

pub(crate) async fn load_texture_path(
    path: PathBuf,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    is_normal_map: bool,
) -> Result<texture::Texture, RendererError> {
    let data = load_binary_path(path.clone()).await?;

    texture::Texture::from_bytes(
        device,
        queue,
        &data,
        path.file_name().unwrap().to_str().unwrap(),
        is_normal_map,
    )
}

// TODO ! use the example from the tobj crate's documentation
pub(crate) async fn load_model_obj(
    file_path: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> Result<model::Model, RendererError> {
    let path = Path::new(file_path);
    let model_root_dir = path.parent().unwrap();
    let file_name = model_root_dir.file_name().unwrap().to_str().unwrap();

    let obj_text = load_string(file_path).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut async_obj_reader = tokio::io::BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::tokio::load_obj_buf(
        &mut async_obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(model_root_dir.join(&p).to_str().unwrap())
                .await
                .unwrap_or_default();

            let mut async_mat_text_reader = tokio::io::BufReader::new(Cursor::new(mat_text));
            tobj::tokio::load_mtl_buf(&mut async_mat_text_reader).await
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        // let diffuse_texture = load_texture(
        //     model_root_dir
        //         .join(m.diffuse_texture.as_ref().expect(
        //             format!("Diffuse texture is required for material {}", &m.name).as_str(),
        //         ))
        //         .to_str()
        //         .unwrap(),
        //     device,
        //     queue,
        //     false,
        // )
        // .await?;

        // Try to load the diffuse texture if it exists for this material
        let diffuse_texture = if let Some(diffuse_texture) = m.diffuse_texture {
            load_texture(
                model_root_dir.join(diffuse_texture).to_str().unwrap(),
                device,
                queue,
                true,
            )
            .await?
        } else {
            // Default white texture if none is provided
            texture::Texture::default_white(device, queue)
        };

        // Try to load the normal texture if it exists for this material
        let normal_texture = if let Some(normal_texture) = m.normal_texture {
            load_texture(
                model_root_dir.join(normal_texture).to_str().unwrap(),
                device,
                queue,
                true,
            )
            .await?
        } else {
            // Default normal texture if none is provided
            texture::Texture::default_normal(device, queue)
        };

        materials.push(model::Material::new(
            device,
            &m.name,
            diffuse_texture,
            normal_texture,
            layout,
        ));
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let mut vertices = (0..m.mesh.positions.len() / 3)
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
                            tangent: [0.0; 3],
                            bitangent: [0.0; 3],
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
                            tangent: [0.0; 3],
                            bitangent: [0.0; 3],
                        }
                    }
                })
                .collect::<Vec<_>>();

            let indices = &m.mesh.indices;
            let mut triangles_included = vec![0; vertices.len()];

            // Calculate tangents and bitangets. We're going to
            // use the triangles, so we need to loop through the
            // indices in chunks of 3
            for c in indices.chunks(3) {
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                let pos0: cgmath::Vector3<_> = v0.position.into();
                let pos1: cgmath::Vector3<_> = v1.position.into();
                let pos2: cgmath::Vector3<_> = v2.position.into();

                let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
                let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
                let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

                // Calculate the edges of the triangle
                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                // This will give us a direction to calculate the
                // tangent and bitangent
                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;
                // Solving the following system of equations will
                // give us the tangent and bitangent.
                //     delta_pos1 = delta_uv1.x * T + delta_uv1.y * B
                //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                let denominator = delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x;

                // Check for degenerate UV coordinates
                if denominator.abs() < 1e-6 {
                    // Use fallback tangent/bitangent based on normal
                    let normal: cgmath::Vector3<f32> = v0.normal.into();
                    let up = if normal.y.abs() < 0.9 {
                        cgmath::Vector3::unit_y()
                    } else {
                        cgmath::Vector3::unit_x()
                    };
                    let tangent = normal.cross(up).normalize();
                    let bitangent = normal.cross(tangent);

                    // Apply to all vertices in triangle
                    vertices[c[0] as usize].tangent =
                        (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
                    vertices[c[1] as usize].tangent =
                        (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
                    vertices[c[2] as usize].tangent =
                        (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();
                    vertices[c[0] as usize].bitangent = (bitangent
                        + cgmath::Vector3::from(vertices[c[0] as usize].bitangent))
                    .into();
                    vertices[c[1] as usize].bitangent = (bitangent
                        + cgmath::Vector3::from(vertices[c[1] as usize].bitangent))
                    .into();
                    vertices[c[2] as usize].bitangent = (bitangent
                        + cgmath::Vector3::from(vertices[c[2] as usize].bitangent))
                    .into();
                } else {
                    let r = 1.0 / denominator;
                    let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                    let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

                    // We'll use the same tangent/bitangent for each vertex in the triangle
                    vertices[c[0] as usize].tangent =
                        (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
                    vertices[c[1] as usize].tangent =
                        (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
                    vertices[c[2] as usize].tangent =
                        (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();
                    vertices[c[0] as usize].bitangent = (bitangent
                        + cgmath::Vector3::from(vertices[c[0] as usize].bitangent))
                    .into();
                    vertices[c[1] as usize].bitangent = (bitangent
                        + cgmath::Vector3::from(vertices[c[1] as usize].bitangent))
                    .into();
                    vertices[c[2] as usize].bitangent = (bitangent
                        + cgmath::Vector3::from(vertices[c[2] as usize].bitangent))
                    .into();
                }

                // Used to average the tangents/bitangents
                triangles_included[c[0] as usize] += 1;
                triangles_included[c[1] as usize] += 1;
                triangles_included[c[2] as usize] += 1;
            }

            // Average and orthogonalize the tangents/bitangents
            for (i, n) in triangles_included.into_iter().enumerate() {
                let denom = 1.0 / n as f32;
                let v = &mut vertices[i];

                // Average the accumulated values
                let tangent = cgmath::Vector3::from(v.tangent) * denom;
                let bitangent = cgmath::Vector3::from(v.bitangent) * denom;
                let normal: cgmath::Vector3<f32> = v.normal.into();

                // Gram-Schmidt orthogonalize
                let orthogonal_tangent = (tangent - normal * normal.dot(tangent)).normalize();
                let orthogonal_bitangent = (bitangent
                    - normal * normal.dot(bitangent)
                    - orthogonal_tangent * orthogonal_tangent.dot(bitangent))
                .normalize();

                v.tangent = orthogonal_tangent.into();
                v.bitangent = orthogonal_bitangent.into();
            }

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
        animations: Vec::new(), // Simple OBJ files don't have animations
    })
}

pub(crate) async fn load_model_gltf(
    gltf_path: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> Result<model::Model, RendererError> {
    let file_path = Path::new(gltf_path);
    let model_root_dir = file_path.parent().ok_or_else(|| {
        RendererError::InvalidPath(format!(
            "No parent directory for path {}.",
            file_path.display()
        ))
    })?;
    let file_name = model_root_dir.file_name().unwrap().to_str().unwrap();

    let string_path = get_resource_path(gltf_path);
    let gltf_text = load_string_path(string_path).await?;
    let gltf_cursor = Cursor::new(gltf_text);
    let gltf_reader = BufReader::new(gltf_cursor);
    let gltf = Gltf::from_reader(gltf_reader)?;

    // Load buffers
    let mut buffer_data = Vec::new();
    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                // Handle binary buffer if necessary
                // if let Some(blob) = gltf.blob.as_deref() {
                //     buffer_data.push(blob.into());
                //     println!("Found a bin, saving");
                // };
            }
            gltf::buffer::Source::Uri(uri) => {
                let uri_path = get_resource_path(model_root_dir.join(uri).to_str().unwrap());
                let bin = load_binary_path(uri_path).await?;
                buffer_data.push(bin);
            }
        }
    }

    // Load animations using the new animation system
    let mut animation_clips: Vec<animation::AnimationClip> = Vec::new();
    for gltf_animation in gltf.animations() {
        let animation_name = gltf_animation.name().unwrap_or("Default").to_string();
        log::info!("Loading GLTF animation: {}", animation_name);

        let mut clip = animation::AnimationClip::new(&animation_name);
        let mut max_duration = 0.0f32;

        for channel in gltf_animation.channels() {
            let reader = channel.reader(|buffer| Some(&buffer_data[buffer.index()]));

            // Read timestamps
            let timestamps = if let Some(inputs) = reader.read_inputs() {
                match inputs {
                    gltf::accessor::Iter::Standard(times) => {
                        let times: Vec<f32> = times.collect();
                        times
                    }
                    gltf::accessor::Iter::Sparse(_) => {
                        warn!(
                            "Sparse keyframes not supported for animation '{}'",
                            animation_name
                        );
                        Vec::new()
                    }
                }
            } else {
                Vec::new()
            };

            // Determine animation target
            let target = match channel.target().property() {
                gltf::animation::Property::Translation => animation::AnimationTarget::Translation,
                gltf::animation::Property::Rotation => animation::AnimationTarget::Rotation,
                gltf::animation::Property::Scale => animation::AnimationTarget::Scale,
                _ => continue,
            };

            // Create animation track
            let mut track = if target == animation::AnimationTarget::Rotation {
                animation::AnimationTrack::new_rotation_track()
            } else {
                animation::AnimationTrack::new()
            };

            // Read keyframe data
            if let Some(outputs) = reader.read_outputs() {
                match outputs {
                    gltf::animation::util::ReadOutputs::Translations(translations) => {
                        for (time, translation) in timestamps.iter().zip(translations) {
                            let value = animation::AnimationValue::Vector3(cgmath::Vector3::new(
                                translation[0],
                                translation[1],
                                translation[2],
                            ));
                            track.add_keyframe(animation::Keyframe::new(*time, value));
                            max_duration = max_duration.max(*time);
                        }
                    }
                    gltf::animation::util::ReadOutputs::Rotations(rotations) => {
                        let rotations_f32 = rotations.into_f32();
                        for (time, rotation) in timestamps.iter().zip(rotations_f32) {
                            let value =
                                animation::AnimationValue::Quaternion(cgmath::Quaternion::new(
                                    rotation[3],
                                    rotation[0],
                                    rotation[1],
                                    rotation[2],
                                ));
                            track.add_keyframe(animation::Keyframe::new(*time, value));
                            max_duration = max_duration.max(*time);
                        }
                    }
                    gltf::animation::util::ReadOutputs::Scales(scales) => {
                        for (time, scale) in timestamps.iter().zip(scales) {
                            let value = animation::AnimationValue::Vector3(cgmath::Vector3::new(
                                scale[0], scale[1], scale[2],
                            ));
                            track.add_keyframe(animation::Keyframe::new(*time, value));
                            max_duration = max_duration.max(*time);
                        }
                    }
                    _ => {
                        warn!("Unsupported animation output type for '{}'", animation_name);
                        continue;
                    }
                }
            } else {
                continue;
            }

            // Add track to clip
            clip.add_track(target.clone(), track);
        }

        // Set clip duration
        clip.duration = max_duration;

        // Store new animation clip directly
        animation_clips.push(clip);
    }

    // Load materials
    let mut materials = Vec::new();
    for material in gltf.materials() {
        let pbr = material.pbr_metallic_roughness();
        let texture_source = &pbr
            .base_color_texture()
            .map(|tex| tex.texture().source().source());

        let (diffuse_texture, normal_texture) = match texture_source {
            Some(texture_source) => {
                let diffuse_texture = match texture_source {
                    // Removed mime_type
                    gltf::image::Source::View { view, .. } => texture::Texture::from_bytes(
                        device,
                        queue,
                        &buffer_data[view.buffer().index()],
                        file_name,
                        false,
                    )?,
                    // Removed mime_type
                    gltf::image::Source::Uri { uri, .. } => {
                        let uri_path =
                            get_resource_path(model_root_dir.join(uri).to_str().unwrap());
                        load_texture_path(uri_path, device, queue, false).await?
                    }
                };

                // Try to load normal texture if it exists
                let normal_texture = if let Some(normal_info) = material.normal_texture() {
                    let normal_texture_source = normal_info.texture().source().source();
                    match normal_texture_source {
                        gltf::image::Source::View { view, .. } => texture::Texture::from_bytes(
                            device,
                            queue,
                            &buffer_data[view.buffer().index()],
                            file_name,
                            true,
                        )?,
                        gltf::image::Source::Uri { uri, .. } => {
                            let uri_path =
                                get_resource_path(model_root_dir.join(uri).to_str().unwrap());
                            load_texture_path(uri_path, device, queue, true).await?
                        }
                    }
                } else {
                    // Default normal texture if none is provided
                    texture::Texture::default_normal(device, queue)
                };

                (diffuse_texture, normal_texture)
            }
            None => {
                log::error!(
                    "No texture source found for material {:?}, using default white texture",
                    material.name()
                );
                (
                    texture::Texture::default_white(device, queue),
                    texture::Texture::default_normal(device, queue),
                )
            }
        };

        materials.push(model::Material::new(
            device,
            material.name().unwrap_or("Default Material"),
            diffuse_texture,
            normal_texture,
            layout,
        ));
    }

    let mut meshes = Vec::new();

    // Recursive function to traverse all nodes in the scene hierarchy
    fn traverse_node(
        node: gltf::Node,
        meshes: &mut Vec<model::Mesh>,
        buffer_data: &[Vec<u8>],
        device: &wgpu::Device,
        file_name: &str,
    ) -> Result<(), RendererError> {
        info!("Node {}", node.index());

        if let Some(mesh) = node.mesh() {
            let mesh_name = mesh.name().unwrap_or("Unnamed Mesh").to_string();

            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

                // Read positions
                let positions: Vec<[f32; 3]> = reader
                    .read_positions()
                    .ok_or_else(|| RendererError::MissingData("Missing positions".to_string()))?
                    .collect();

                // Read normals or generate default
                let normals: Vec<[f32; 3]> = reader
                    .read_normals()
                    .map(|n| n.collect())
                    .unwrap_or_else(|| {
                        warn!("No normals found for mesh {}", mesh_name);
                        positions.iter().map(|_| [0.0, 0.0, 0.0]).collect()
                    });

                // Read tex_coords or generate default
                let tex_coords: Vec<[f32; 2]> = reader
                    .read_tex_coords(0)
                    .map(|v| {
                        v.into_f32()
                            .map(|mut tex_coord| {
                                // Flip the V-component of the texture coordinate
                                tex_coord[1] = 1.0 - tex_coord[1];
                                tex_coord
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_else(|| {
                        warn!("No texture coordinates found for mesh {}", mesh_name);
                        positions.iter().map(|_| [0.0, 0.0]).collect()
                    });

                // Read indices or generate sequential
                let indices: Vec<u32> = reader
                    .read_indices()
                    .map(|i| i.into_u32().collect())
                    .unwrap_or_else(|| (0..positions.len() as u32).collect());

                // Create vertices with all attributes
                let mut vertices = (0..positions.len())
                    .map(|i| model::ModelVertex {
                        position: positions[i],
                        normal: normals[i],
                        tex_coords: tex_coords[i],
                        tangent: [0.0; 3],
                        bitangent: [0.0; 3],
                    })
                    .collect::<Vec<_>>();

                let mut triangles_included = vec![0; vertices.len()];

                // Calculate tangents and bitangents using the original triangle indices
                for c in indices.chunks(3) {
                    let i0 = c[0] as usize;
                    let i1 = c[1] as usize;
                    let i2 = c[2] as usize;

                    let v0 = vertices[i0];
                    let v1 = vertices[i1];
                    let v2 = vertices[i2];

                    let pos0: cgmath::Vector3<_> = v0.position.into();
                    let pos1: cgmath::Vector3<_> = v1.position.into();
                    let pos2: cgmath::Vector3<_> = v2.position.into();

                    let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
                    let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
                    let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

                    // Calculate the edges of the triangle
                    let delta_pos1 = pos1 - pos0;
                    let delta_pos2 = pos2 - pos0;

                    // This will give us a direction to calculate the
                    // tangent and bitangent
                    let delta_uv1 = uv1 - uv0;
                    let delta_uv2 = uv2 - uv0;

                    let denominator = delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x;

                    // Check for degenerate UV coordinates
                    if denominator.abs() < 1e-6 {
                        // Use fallback tangent/bitangent based on normal
                        let normal: cgmath::Vector3<f32> = v0.normal.into();
                        let up = if normal.y.abs() < 0.9 {
                            cgmath::Vector3::unit_y()
                        } else {
                            cgmath::Vector3::unit_x()
                        };
                        let tangent = normal.cross(up).normalize();
                        let bitangent = normal.cross(tangent);

                        // Apply to all vertices in triangle
                        vertices[i0].tangent =
                            (tangent + cgmath::Vector3::from(vertices[i0].tangent)).into();
                        vertices[i1].tangent =
                            (tangent + cgmath::Vector3::from(vertices[i1].tangent)).into();
                        vertices[i2].tangent =
                            (tangent + cgmath::Vector3::from(vertices[i2].tangent)).into();
                        vertices[i0].bitangent =
                            (bitangent + cgmath::Vector3::from(vertices[i0].bitangent)).into();
                        vertices[i1].bitangent =
                            (bitangent + cgmath::Vector3::from(vertices[i1].bitangent)).into();
                        vertices[i2].bitangent =
                            (bitangent + cgmath::Vector3::from(vertices[i2].bitangent)).into();
                    } else {
                        let r = 1.0 / denominator;
                        let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                        let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

                        // Apply to all vertices in triangle
                        vertices[i0].tangent =
                            (tangent + cgmath::Vector3::from(vertices[i0].tangent)).into();
                        vertices[i1].tangent =
                            (tangent + cgmath::Vector3::from(vertices[i1].tangent)).into();
                        vertices[i2].tangent =
                            (tangent + cgmath::Vector3::from(vertices[i2].tangent)).into();
                        vertices[i0].bitangent =
                            (bitangent + cgmath::Vector3::from(vertices[i0].bitangent)).into();
                        vertices[i1].bitangent =
                            (bitangent + cgmath::Vector3::from(vertices[i1].bitangent)).into();
                        vertices[i2].bitangent =
                            (bitangent + cgmath::Vector3::from(vertices[i2].bitangent)).into();
                    }

                    // Used to average the tangents/bitangents
                    triangles_included[i0] += 1;
                    triangles_included[i1] += 1;
                    triangles_included[i2] += 1;
                }

                // Average and orthogonalize the tangents/bitangents
                for (i, n) in triangles_included.into_iter().enumerate() {
                    if n > 0 {
                        let denom = 1.0 / n as f32;
                        let v = &mut vertices[i];

                        // Average the accumulated values
                        let tangent = cgmath::Vector3::from(v.tangent) * denom;
                        let bitangent = cgmath::Vector3::from(v.bitangent) * denom;
                        let normal: cgmath::Vector3<f32> = v.normal.into();

                        // Gram-Schmidt orthogonalize
                        let orthogonal_tangent =
                            (tangent - normal * normal.dot(tangent)).normalize();
                        let orthogonal_bitangent = (bitangent
                            - normal * normal.dot(bitangent)
                            - orthogonal_tangent * orthogonal_tangent.dot(bitangent))
                        .normalize();

                        v.tangent = orthogonal_tangent.into();
                        v.bitangent = orthogonal_bitangent.into();
                    }
                }

                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Vertex Buffer", mesh_name)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Index Buffer", mesh_name)),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

                log::info!("Mesh: {}", mesh_name);
                meshes.push(model::Mesh {
                    name: mesh_name.clone(),
                    vertex_buffer,
                    index_buffer,
                    num_elements: indices.len() as u32,
                    material: primitive.material().index().unwrap_or(0),
                });
            }
        }

        // Recursively traverse child nodes
        for child in node.children() {
            traverse_node(child, meshes, buffer_data, device, file_name)?;
        }

        Ok(())
    }

    for scene in gltf.scenes() {
        for node in scene.nodes() {
            traverse_node(node, &mut meshes, &buffer_data, device, file_name)?;
        }
    }

    Ok(model::Model {
        meshes,
        materials,
        animations: animation_clips,
    })
}
