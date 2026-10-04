// The text card: ThorVG's L texture (premultiplied) placed in the one camera.
struct Uniforms { vp: mat4x4<f32>, model: mat4x4<f32> };
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
@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    let sample = textureSample(tex, samp, in.uv);
    if (sample.a < 0.02) { discard; }
    return sample;
}
