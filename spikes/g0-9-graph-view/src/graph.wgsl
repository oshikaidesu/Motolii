struct ScreenUniform {
    size: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(0) var<uniform> screen: ScreenUniform;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) bounds: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) extra: vec4<f32>,
    @location(3) shape: u32,
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
    var pixel = input.bounds.xy + local * input.bounds.zw;
    if input.shape == 3u {
        let p0 = input.bounds.xy;
        let p1 = input.extra.xy;
        let half_width = input.extra.z;
        let delta = p1 - p0;
        let direction = delta / max(length(delta), 0.0001);
        let normal = vec2(-direction.y, direction.x) * half_width;
        let line_corners = array<vec2<f32>, 6>(
            p0 - normal, p1 - normal, p0 + normal,
            p0 + normal, p1 - normal, p1 + normal,
        );
        pixel = line_corners[input.vertex_index];
    }
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
    if input.shape == 1u && distance(input.local, vec2(0.5, 0.5)) > 0.5 {
        discard;
    }
    if input.shape == 2u && abs(input.local.x - 0.5) + abs(input.local.y - 0.5) > 0.5 {
        discard;
    }
    return input.color;
}
