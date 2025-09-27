struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}

struct Light {
    position: vec3<f32>,
    light_type: u32,
    color: vec3<f32>,
    radius: f32,
    direction: vec3<f32>,
    intensity: f32,
}

struct LightData {
    lights: array<Light, 20>,
    num_lights: u32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
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
    @location(1) world_position: vec3<f32>,
    @location(2) tangent_matrix_0: vec3<f32>,
    @location(3) tangent_matrix_1: vec3<f32>,
    @location(4) tangent_matrix_2: vec3<f32>,
    @location(5) tangent_view_position: vec3<f32>,
}

// Diffuse texture map
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

// Normal texture map
@group(0) @binding(2)
var t_normal: texture_2d<f32>;

@group(0) @binding(3)
var s_normal: sampler;

// Camera
@group(1) @binding(0)
var<uniform> camera: Camera;

// Lights
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

    // Construct the tangent matrix
    let world_normal = normalize(normal_matrix * model.normal);
    let world_tangent = normalize(normal_matrix * model.tangent);
    let world_bitangent = normalize(normal_matrix * model.bitangent);
    let tangent_matrix = transpose(mat3x3<f32>(
        world_tangent,
        world_bitangent,
        world_normal,
    ));

    let world_position = model_matrix * vec4<f32>(model.position, 1.0);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_position;
    out.tex_coords = model.tex_coords;
    out.world_position = world_position.xyz;
    // Pass tangent matrix components to fragment shader
    out.tangent_matrix_0 = tangent_matrix[0];
    out.tangent_matrix_1 = tangent_matrix[1];
    out.tangent_matrix_2 = tangent_matrix[2];
    out.tangent_view_position = tangent_matrix * camera.view_pos.xyz;
    return out;
}


// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let object_normal: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);

    // Reconstruct tangent matrix
    let tangent_matrix = mat3x3<f32>(
        in.tangent_matrix_0,
        in.tangent_matrix_1,
        in.tangent_matrix_2,
    );

    let tangent_position = tangent_matrix * in.world_position;

    var result_color: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);

    for (var i = 0u; i < light_data.num_lights; i = i + 1u) {
        let light = light_data.lights[i];

        if light.light_type == 0u { // Point light
            // Transform light position to tangent space
            let tangent_light_position = tangent_matrix * light.position;
            let distance = length(tangent_light_position - tangent_position);
            let attenuation = clamp(1.0 - (distance / light.radius) * (distance / light.radius), 0.0, 1.0);

            if attenuation > 0.0 {
                let tangent_normal = object_normal.xyz * 2.0 - 1.0;
                let light_dir = normalize(tangent_light_position - tangent_position);
                let view_dir = normalize(in.tangent_view_position - tangent_position);
                let half_dir = normalize(view_dir + light_dir);

                // Diffuse component
                let diffuse_strength = max(dot(tangent_normal, light_dir), 0.0);
                let diffuse_color = light.color * light.intensity * diffuse_strength * attenuation;

                // Specular component
                let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0);
                let specular_color = light.color * light.intensity * specular_strength * attenuation;

                result_color = result_color + (diffuse_color + specular_color) * mix(object_color.xyz, light.color, 0.3);
            }
        } else if light.light_type == 1u { // Ambient light
            let ambient_color = light.color * light.intensity;
            result_color = result_color + ambient_color * object_color.xyz;
        } else if light.light_type == 2u { // Directional light
            // Transform light direction to tangent space
            let tangent_light_direction = normalize(tangent_matrix * light.direction);

            let tangent_normal = object_normal.xyz * 2.0 - 1.0;
            let view_dir = normalize(in.tangent_view_position - tangent_position);
            let half_dir = normalize(view_dir + (-tangent_light_direction));

            // Diffuse component
            let diffuse_strength = max(dot(tangent_normal, -tangent_light_direction), 0.0);
            let diffuse_color = light.color * light.intensity * diffuse_strength;

            // Specular component
            let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0);
            let specular_color = light.color * light.intensity * specular_strength;

            result_color = result_color + (diffuse_color + specular_color) * object_color.xyz;
        }
    }

    return vec4<f32>(result_color, object_color.a);
}
