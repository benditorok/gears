use super::{animation, model, texture};
use anyhow::Context;
use gltf::Gltf;
use log::{info, warn};
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};
use wgpu::util::DeviceExt;

fn get_resource_path(file_path: &str) -> PathBuf {
    Path::new(env!("RES_DIR")).join(file_path)
}

pub(crate) async fn load_string(file_path: &str) -> anyhow::Result<String> {
    let path = get_resource_path(file_path);
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
    let path = get_resource_path(file_path);
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
    let model_root_dir = file_path.parent().context(format!(
        "No parent directory for path {}.",
        file_path.display()
    ))?;
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
    let mut animation_clips = Vec::new();
    for gltf_animation in gltf.animations() {
        let animation_name = gltf_animation.name().unwrap_or("Default").to_string();
        log::debug!("Loading GLTF animation: {}", animation_name);

        let mut clip = animation::AnimationClip::new(&animation_name);
        let mut max_duration = 0.0f32;

        for channel in gltf_animation.channels() {
            let reader = channel.reader(|buffer| Some(&buffer_data[buffer.index()]));

            // Read timestamps
            let timestamps = if let Some(inputs) = reader.read_inputs() {
                match inputs {
                    gltf::accessor::Iter::Standard(times) => {
                        let times: Vec<f32> = times.collect();
                        log::debug!("Animation '{}' times: {:?}", animation_name, &times);
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
                log::warn!(
                    "No input timestamps found for animation '{}'",
                    animation_name
                );
                Vec::new()
            };

            // Determine animation target
            let target = match channel.target().property() {
                gltf::animation::Property::Translation => {
                    log::debug!("Found translation track in animation '{}'", animation_name);
                    animation::AnimationTarget::Translation
                }
                gltf::animation::Property::Rotation => {
                    log::debug!("Found rotation track in animation '{}'", animation_name);
                    animation::AnimationTarget::Rotation
                }
                gltf::animation::Property::Scale => {
                    log::debug!("Found scale track in animation '{}'", animation_name);
                    animation::AnimationTarget::Scale
                }
                _ => {
                    log::warn!(
                        "Skipping unsupported animation property in '{}'",
                        animation_name
                    );
                    continue;
                }
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
                        let translation_count = translations.len();
                        log::debug!(
                            "Reading {} translation keyframes for '{}'",
                            translation_count,
                            animation_name
                        );
                        for (time, translation) in timestamps.iter().zip(translations) {
                            let value = animation::AnimationValue::Vector3(cgmath::Vector3::new(
                                translation[0],
                                translation[1],
                                translation[2],
                            ));
                            track.add_keyframe(animation::Keyframe::new(*time, value));
                            max_duration = max_duration.max(*time);
                            log::debug!(
                                "Added translation keyframe at {}: [{}, {}, {}]",
                                time,
                                translation[0],
                                translation[1],
                                translation[2]
                            );
                        }
                    }
                    gltf::animation::util::ReadOutputs::Rotations(rotations) => {
                        let rotations_f32 = rotations.into_f32();
                        let rotation_count = rotations_f32.len();
                        log::debug!(
                            "Reading {} rotation keyframes for '{}'",
                            rotation_count,
                            animation_name
                        );
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
                            log::debug!(
                                "Added rotation keyframe at {}: [{}, {}, {}, {}]",
                                time,
                                rotation[0],
                                rotation[1],
                                rotation[2],
                                rotation[3]
                            );
                        }
                    }
                    gltf::animation::util::ReadOutputs::Scales(scales) => {
                        let scale_count = scales.len();
                        log::debug!(
                            "Reading {} scale keyframes for '{}'",
                            scale_count,
                            animation_name
                        );
                        for (time, scale) in timestamps.iter().zip(scales) {
                            let value = animation::AnimationValue::Vector3(cgmath::Vector3::new(
                                scale[0], scale[1], scale[2],
                            ));
                            track.add_keyframe(animation::Keyframe::new(*time, value));
                            max_duration = max_duration.max(*time);
                            log::debug!(
                                "Added scale keyframe at {}: [{}, {}, {}]",
                                time,
                                scale[0],
                                scale[1],
                                scale[2]
                            );
                        }
                    }
                    _ => {
                        warn!("Unsupported animation output type for '{}'", animation_name);
                        continue;
                    }
                }
            } else {
                log::warn!(
                    "No output data found for animation track in '{}'",
                    animation_name
                );
            }

            // Add track to clip
            clip.add_track(target.clone(), track);
            log::debug!(
                "Added track for target {:?} to animation '{}'",
                target,
                animation_name
            );
        }

        // Set clip duration
        clip.duration = max_duration;
        log::debug!(
            "Animation '{}' total duration: {}",
            animation_name,
            max_duration
        );

        // Convert to old format for compatibility (temporary)
        // We need to create proper legacy data instead of placeholder
        let legacy_clip = if let Some(translation_track) =
            clip.get_track(&animation::AnimationTarget::Translation)
        {
            log::debug!(
                "Converting translation track to legacy format for '{}'",
                animation_name
            );
            let mut translation_frames = Vec::new();
            let mut timestamps = Vec::new();

            for keyframe in &translation_track.keyframes {
                if let Some(vec3) = keyframe.value.as_vector3() {
                    translation_frames.push(vec![vec3.x, vec3.y, vec3.z]);
                    timestamps.push(keyframe.time);
                }
            }

            model::AnimationClip {
                name: clip.name.clone(),
                keyframes: model::Keyframes::Translation(translation_frames),
                timestamps,
            }
        } else if let Some(rotation_track) = clip.get_track(&animation::AnimationTarget::Rotation) {
            log::debug!(
                "Converting rotation track to legacy format for '{}'",
                animation_name
            );
            let mut rotation_frames = Vec::new();
            let mut timestamps = Vec::new();

            for keyframe in &rotation_track.keyframes {
                if let Some(quat) = keyframe.value.as_quaternion() {
                    rotation_frames.push(vec![quat.v.x, quat.v.y, quat.v.z, quat.s]);
                    timestamps.push(keyframe.time);
                }
            }

            model::AnimationClip {
                name: clip.name.clone(),
                keyframes: model::Keyframes::Rotation(rotation_frames),
                timestamps,
            }
        } else {
            log::warn!(
                "No supported tracks found in animation '{}', using placeholder",
                animation_name
            );
            model::AnimationClip {
                name: clip.name.clone(),
                keyframes: model::Keyframes::Other,
                timestamps: Vec::new(),
            }
        };

        animation_clips.push(legacy_clip);
    }

    // Load materials
    let mut materials = Vec::new();
    for material in gltf.materials() {
        let pbr = material.pbr_metallic_roughness();
        let texture_source = &pbr
            .base_color_texture()
            .map(|tex| tex.texture().source().source());

        let texture_source = match texture_source {
            Some(source) => source,
            None => {
                warn!("No texture source found for material {:?}", material.name());

                println!("Continue? (y/n)");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();

                if input.trim() == "y" {
                    continue;
                } else {
                    panic!(
                        "Aborting due to missing texture source for material {:?}",
                        material.name()
                    );
                }
            }
        };

        match texture_source {
            // Removed mime_type
            gltf::image::Source::View { view, .. } => {
                let texture = texture::Texture::from_bytes(
                    device,
                    queue,
                    &buffer_data[view.buffer().index()],
                    file_name,
                )?;
                // Removed the invalid cloning line:
                // texture.view = texture.view.clone();

                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&texture.sampler),
                        },
                    ],
                    label: None,
                });

                materials.push(model::Material {
                    name: material.name().unwrap_or("Default Material").to_string(),
                    diffuse_texture: texture,
                    bind_group,
                });
            }
            // Removed mime_type
            gltf::image::Source::Uri { uri, .. } => {
                let uri_path = get_resource_path(model_root_dir.join(uri).to_str().unwrap());
                let diffuse_texture = load_texture_path(uri_path, device, queue).await?;
                // Removed the invalid cloning line:
                // diffuse_texture.view = diffuse_texture.view.clone();

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

            if let Some(mesh) = node.mesh() {
                let mesh_name = mesh.name().unwrap_or("Unnamed Mesh").to_string(); // Capture mesh name

                for primitive in mesh.primitives() {
                    let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

                    // Read positions
                    let positions: Vec<[f32; 3]> = reader
                        .read_positions()
                        .ok_or_else(|| anyhow::anyhow!("Missing positions"))?
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

                    // Construct vertices using indices
                    let vertices: Vec<model::ModelVertex> = indices
                        .iter()
                        .map(|&i| model::ModelVertex {
                            position: positions[i as usize],
                            normal: normals[i as usize],
                            tex_coords: tex_coords[i as usize],
                        })
                        .collect();

                    // Use deduplicated indices
                    let unique_vertices = vertices.clone();
                    let unique_indices: Vec<u32> = (0..unique_vertices.len() as u32).collect();

                    let vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&format!("{:?} Vertex Buffer", mesh_name)),
                            contents: bytemuck::cast_slice(&unique_vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                    let index_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&format!("{:?} Index Buffer", mesh_name)),
                            contents: bytemuck::cast_slice(&unique_indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                    log::info!("Mesh: {}", mesh_name);
                    meshes.push(model::Mesh {
                        name: mesh_name.clone(),
                        vertex_buffer,
                        index_buffer,
                        num_elements: unique_indices.len() as u32,
                        material: primitive.material().index().unwrap_or(0),
                    });
                }
            } else {
                warn!("Node {} has no mesh", node.index());
            }
        }
    }

    Ok(model::Model {
        meshes,
        materials,
        animations: animation_clips,
    })
}
