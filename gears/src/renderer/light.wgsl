struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}
// TODO max szamot is kintrol kapja be?
struct LightData {
    lights: array<Light, 20>,
    num_lights: u32,
}

// Vertex shader

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> light_data: LightData;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
   let scale = 0.25;
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position * scale, 1.0);
    out.color = vec3<f32>(0.0, 0.0, 0.0);
    
    for (var i = 0u; i < light_data.num_lights; i = i + 1u) {
        let light = light_data.lights[i];
        let light_dir = normalize(light.position - model.position);
        let diff = max(dot(light_dir, vec3<f32>(0.0, 0.0, 1.0)), 0.0);
        out.color = out.color + light.color * diff;
    }
    return out;
}

// Fragment shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}