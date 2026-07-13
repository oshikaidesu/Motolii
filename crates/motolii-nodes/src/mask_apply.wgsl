struct MaskUniform { mode: u32, _pad0: u32, _pad1: u32, _pad2: u32, }
@group(0) @binding(0) var content_tex: texture_2d<f32>;
@group(0) @binding(1) var content_samp: sampler;
@group(0) @binding(2) var mask_tex: texture_2d<f32>;
@group(0) @binding(3) var mask_samp: sampler;
@group(0) @binding(4) var<uniform> u: MaskUniform;
fn luminance(c: vec3<f32>) -> f32 { return dot(c, vec3(0.2126, 0.7152, 0.0722)); }
fn mask_factor(mask: vec4<f32>) -> f32 {
    var f: f32;
    switch (u.mode) { case 0u: { f = mask.a; } case 1u: { f = luminance(mask.rgb); } case 2u: { f = 1.0 - mask.a; } default: { f = 1.0 - luminance(mask.rgb); } }
    return clamp(f, 0.0, 1.0);
}
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32>, }
@vertex fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var p = array<vec2<f32>, 3>(vec2(-1.0,-1.0), vec2(3.0,-1.0), vec2(-1.0,3.0));
    var o: VsOut; o.pos = vec4(p[vi],0.0,1.0); o.uv = p[vi]*vec2(0.5,-0.5)+vec2(0.5,0.5); return o;
}
@fragment fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(content_tex, content_samp, in.uv) * mask_factor(textureSample(mask_tex, mask_samp, in.uv));
}
