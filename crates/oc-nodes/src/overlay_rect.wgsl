struct RectUniform {
    min_px: vec2<f32>,
    max_px: vec2<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> rect: RectUniform;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    let p = positions[vertex_index];
    var out: VsOut;
    out.pos = vec4<f32>(p, 0.0, 1.0);
    out.uv = p * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let base = textureSample(input_tex, input_sampler, in.uv);
    let inside =
        in.pos.x >= rect.min_px.x &&
        in.pos.x < rect.max_px.x &&
        in.pos.y >= rect.min_px.y &&
        in.pos.y < rect.max_px.y;
    if inside {
        return rect.color;
    }
    return base;
}
