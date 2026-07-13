struct AffineUniform {
    // UV空間の逆アフィン: uv_src = M * [u, v, 1]
    m00: f32,
    m01: f32,
    m02: f32,
    _pad0: f32,
    m10: f32,
    m11: f32,
    m12: f32,
    _pad1: f32,
}

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var input_samp: sampler;
@group(0) @binding(2) var<uniform> u: AffineUniform;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var p = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    var o: VsOut;
    o.pos = vec4(p[vi], 0.0, 1.0);
    o.uv = p[vi] * vec2(0.5, -0.5) + vec2(0.5, 0.5);
    return o;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let src = vec2(
        u.m00 * in.uv.x + u.m01 * in.uv.y + u.m02,
        u.m10 * in.uv.x + u.m11 * in.uv.y + u.m12,
    );
    // フレーム外は透明(premul 0)。
    if (src.x < 0.0 || src.x > 1.0 || src.y < 0.0 || src.y > 1.0) {
        return vec4(0.0);
    }
    return textureSample(input_tex, input_samp, src);
}
