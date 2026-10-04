// The last pass: the shown picture into the presented surface. Gamma space, as the rest.
// A soft vignette, a touch of contrast, and a hash dither so 8-bit gradients do not band.
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};
@vertex
fn vs(@builtin(vertex_index) index: u32) -> VsOut {
    var pos = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    var uv = array<vec2<f32>, 3>(vec2<f32>(0.0, 1.0), vec2<f32>(2.0, 1.0), vec2<f32>(0.0, -1.0));
    var out: VsOut;
    out.clip = vec4<f32>(pos[index], 0.0, 1.0);
    out.uv = uv[index];
    return out;
}

fn hash(p: vec2<f32>) -> f32 {
    let q = fract(p * vec2<f32>(0.1031, 0.1030));
    let r = q + dot(q, q.yx + 33.33);
    return fract((r.x + r.y) * r.x);
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    var rgb = textureSample(tex, samp, in.uv).rgb;
    // Contrast around mid grey, kept gentle so the video's blacks and whites stay where they were.
    rgb = (rgb - 0.5) * 1.06 + 0.5;
    // Vignette: wide and soft, darker toward the corners.
    let d = (in.uv - 0.5) * vec2<f32>(1.0, 0.85);
    let vig = 1.0 - smoothstep(0.35, 1.05, length(d) * 1.35);
    rgb *= 0.78 + 0.22 * vig;
    // Dither: one LSB of noise, static per pixel.
    rgb += (hash(in.clip.xy) - 0.5) / 255.0;
    return vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
