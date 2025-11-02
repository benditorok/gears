struct CrosshairUniforms {
    screen_width: f32,
    screen_height: f32,
    gap: f32,
    length: f32,
    thickness: f32,
    _padding0: f32,
    _padding1: f32,
    _padding2: f32,
    color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: CrosshairUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Vertex shader - generates a fullscreen triangle
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // Generate fullscreen triangle
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);

    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);

    return out;
}

// Fragment shader - procedurally generates the crosshair
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert UV to screen pixel coordinates
    let screen_pos = vec2<f32>(
        in.uv.x * uniforms.screen_width,
        in.uv.y * uniforms.screen_height
    );

    // Center of the screen
    let center = vec2<f32>(
        uniforms.screen_width * 0.5,
        uniforms.screen_height * 0.5
    );

    // Distance from center
    let dist_from_center = screen_pos - center;
    let abs_dist = abs(dist_from_center);

    // Horizontal line (left and right from center)
    let is_horizontal = abs_dist.y < uniforms.thickness * 0.5 && abs_dist.x > uniforms.gap && abs_dist.x < uniforms.gap + uniforms.length;

    // Vertical line (top and bottom from center)
    let is_vertical = abs_dist.x < uniforms.thickness * 0.5 && abs_dist.y > uniforms.gap && abs_dist.y < uniforms.gap + uniforms.length;

    // Center dot (optional)
    let is_center_dot = length(dist_from_center) < uniforms.thickness * 0.5;

    // Combine all parts
    if is_horizontal || is_vertical {
        return uniforms.color;
    }

    // Discard pixels that aren't part of the crosshair (transparent)
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
