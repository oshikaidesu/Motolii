// A community-style effect: one file, one fragment function, parameters are the uniform struct's members.
@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var smp: sampler;

struct Params {
    threshold: f32,
    intensity: f32,
    radius: f32,
};
@group(1) @binding(0) var<uniform> params: Params;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(src));
    let uv = pos.xy / dims;
    let base = textureSampleLevel(src, smp, uv, 0.0);
    var glow = vec3<f32>(0.0);
    var wsum = 0.0;
    for (var i = 0; i < 64; i++) {
        let a = f32(i) * 2.399963;
        let r = sqrt((f32(i) + 0.5) / 64.0);
        let o = vec2<f32>(cos(a), sin(a)) * r * params.radius / dims;
        let c = textureSampleLevel(src, smp, uv + o, 0.0).rgb;
        let k = max(max(c.r, max(c.g, c.b)) - params.threshold, 0.0);
        let w = 1.0 - r * 0.85;
        glow += c * k * w;
        wsum += w;
    }
    glow = glow / wsum * params.intensity * 8.0;
    return vec4<f32>(base.rgb + glow, base.a);
}
