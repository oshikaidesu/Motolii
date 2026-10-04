// Echo plates: the dry picture at t-6 and t-12 as boards in the one camera.
// The board's edge fades out (premultiplied over), so the past hangs in the space without a frame.
struct Uniforms { vp: mat4x4<f32>, model: mat4x4<f32>, fade: f32 };
@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};
@vertex
fn vs(@location(0) pos: vec3<f32>, @location(1) uv: vec2<f32>) -> VsOut {
    var out: VsOut;
    out.clip = u.vp * u.model * vec4<f32>(pos, 1.0);
    out.uv = uv;
    return out;
}

const EDGE: f32 = 0.16;

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    let picture = textureSample(tex, samp, in.uv).rgb;
    let to_edge = min(min(in.uv.x, 1.0 - in.uv.x), min(in.uv.y, 1.0 - in.uv.y));
    let keep = smoothstep(0.0, EDGE, to_edge) * u.fade;
    return vec4<f32>(picture * keep, keep);
}
