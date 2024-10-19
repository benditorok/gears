struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}

struct Light {
    position: vec3<f32>,
    light_type: u32,
    color: vec3<f32>,
    radius: f32,
}

struct LightData {
    lights: array<Light, 20>,
    num_lights: u32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0)@binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> camera: Camera;

@group(2) @binding(0)
var<uniform> light_data: LightData;

// Vertex shader

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.world_normal = normal_matrix * model.normal;
    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    var result_color: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);

    for (var i = 0u; i < light_data.num_lights; i = i + 1u) {
        let light = light_data.lights[i];

        let ambient_strength = 0.1;
        let ambient_color = light.color * ambient_strength;

        if (light.light_type == 0u) { // Point light
            let distance = length(light.position - in.world_position);
            let attenuation = clamp(1.0 - distance / light.radius, 0.0, 1.0);

            if (attenuation > 0.0) {
                let light_dir = normalize(light.position - in.world_position);
                let view_dir = normalize(camera.view_pos.xyz - in.world_position);
                let half_dir = normalize(view_dir + light_dir);

                let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
                let diffuse_color = light.color * diffuse_strength * attenuation;

                let specular_strength = pow(max(dot(in.world_normal, half_dir), 0.0), 32.0);
                let specular_color = specular_strength * light.color * attenuation;

                result_color = result_color + (ambient_color + diffuse_color + specular_color) * object_color.xyz;
            }
        } else if (light.light_type == 1u) { // Ambient light
            result_color = result_color + ambient_color * object_color.xyz;
        } else if (light.light_type == 2u) { // Directional light
            let light_dir = normalize(light.position - in.world_position);
            let view_dir = normalize(camera.view_pos.xyz - in.world_position);
            let half_dir = normalize(view_dir + light_dir);

            let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
            let diffuse_color = light.color * diffuse_strength;

            let specular_strength = pow(max(dot(in.world_normal, half_dir), 0.0), 32.0);
            let specular_color = specular_strength * light.color;

            result_color = result_color + (ambient_color + diffuse_color + specular_color) * object_color.xyz;
        }
    }

    return vec4<f32>(result_color, object_color.a);
}