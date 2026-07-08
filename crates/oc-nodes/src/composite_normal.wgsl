@group(0) @binding(0) var background_tex: texture_2d<f32>;
@group(0) @binding(1) var tex_sampler: sampler;
@group(0) @binding(2) var foreground_tex: texture_2d<f32>;

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
    let bg = textureSample(background_tex, tex_sampler, in.uv);
    let fg = textureSample(foreground_tex, tex_sampler, in.uv);
    let inv_a = 1.0 - fg.a;
    return vec4<f32>(
        fg.rgb + bg.rgb * inv_a,
        fg.a + bg.a * inv_a
    );
}
