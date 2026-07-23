struct ScreenUniform {
    size: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(0) var<uniform> screen: ScreenUniform;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) rect: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) shape: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local: vec2<f32>,
    @location(2) @interpolate(flat) shape: u32,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let corners = array<vec2<f32>, 6>(
        vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0),
        vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
    );
    let local = corners[input.vertex_index];
    let pixel = input.rect.xy + local * input.rect.zw;
    let clip = vec2(pixel.x / screen.size.x * 2.0 - 1.0, 1.0 - pixel.y / screen.size.y * 2.0);
    var output: VertexOutput;
    output.position = vec4(clip, 0.0, 1.0);
    output.color = input.color;
    output.local = local;
    output.shape = input.shape;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    if input.shape == 1u && abs(input.local.x - 0.5) + abs(input.local.y - 0.5) > 0.5 {
        discard;
    }
    if input.shape == 2u && input.local.y < abs(input.local.x - 0.5) * 1.8 {
        discard;
    }
    return input.color;
}
