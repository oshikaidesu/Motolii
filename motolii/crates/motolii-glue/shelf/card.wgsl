// The video card: NV12 planes sampled in the one camera. 8-bit BT.709 limited range.
struct Uniforms { vp: mat4x4<f32>, model: mat4x4<f32> };
@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var y_tex: texture_2d<f32>;
@group(0) @binding(2) var uv_tex: texture_2d<f32>;
@group(0) @binding(3) var samp: sampler;
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
    let y_raw = textureSample(y_tex, samp, in.uv).r;
    let uv_raw = textureSample(uv_tex, samp, in.uv).rg;
    let y = (y_raw - 16.0 / 255.0) * (255.0 / 219.0);
    let cu = (uv_raw.x - 128.0 / 255.0) * (255.0 / 224.0);
    let cv = (uv_raw.y - 128.0 / 255.0) * (255.0 / 224.0);
    let rgb = vec3<f32>(y + 1.5748 * cv, y - 0.1873 * cu - 0.4681 * cv, y + 1.8556 * cu);
    return vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
