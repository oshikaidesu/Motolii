// Glass: the usual screen-space refraction, thick and colored. The scene is already in `curr`.
// Per-channel refraction (dispersion), Beer–Lambert absorption by thickness, Schlick Fresnel,
// a screen-space reflection of the scene, and a Blinn–Phong highlight from a fixed key light.
// Not a ray march; one pass, a handful of samples.
struct Glass { center: vec2<f32>, radius: f32, aspect: f32 };
@group(0) @binding(0) var curr: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> g: Glass;
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

const IOR: f32 = 1.52;
const DISPERSION: f32 = 0.035;
const BEND: f32 = 0.11;
// Absorption per unit thickness, per channel. Low red, high blue: a deep amber body.
const ABSORB: vec3<f32> = vec3<f32>(0.10, 0.48, 0.95);
const TINT: vec3<f32> = vec3<f32>(1.0, 0.62, 0.22);

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    let scene = textureSample(curr, samp, in.uv).rgb;
    let p = (in.uv - g.center) * vec2<f32>(g.aspect, 1.0);
    let r2 = dot(p, p);
    let dist = length(p);
    let aa = max(fwidth(dist), 0.0001);
    let cover = 1.0 - smoothstep(g.radius - aa, g.radius + aa, dist);
    if (cover <= 0.0) { return vec4<f32>(scene, 1.0); }
    let z = sqrt(max(g.radius * g.radius - r2, 0.0));
    let n = normalize(vec3<f32>(p, z));
    let view = vec3<f32>(0.0, 0.0, 1.0);
    let ndv = clamp(n.z, 0.0, 1.0);

    // Refraction, one sample per channel so the edges split into color.
    let bent_r = refract(-view, n, 1.0 / (IOR - DISPERSION));
    let bent_g = refract(-view, n, 1.0 / IOR);
    let bent_b = refract(-view, n, 1.0 / (IOR + DISPERSION));
    let scale = BEND * vec2<f32>(1.0 / g.aspect, 1.0);
    let lo = vec2<f32>(0.002);
    let hi = vec2<f32>(0.998);
    let red = textureSample(curr, samp, clamp(in.uv + bent_r.xy * scale, lo, hi)).r;
    let green = textureSample(curr, samp, clamp(in.uv + bent_g.xy * scale, lo, hi)).g;
    let blue = textureSample(curr, samp, clamp(in.uv + bent_b.xy * scale, lo, hi)).b;
    let through = vec3<f32>(red, green, blue);

    // Thickness through the sphere along the view, absorbed by color.
    let thickness = z * 2.0 / max(g.radius, 0.001);
    let absorb = exp(-ABSORB * thickness);
    var body = through * absorb;
    body += TINT * 0.16 * (1.0 - absorb.g);

    // The scene mirrored by the surface, as the reflection of the room.
    let refl = reflect(-view, n);
    let mirror_uv = clamp(in.uv - refl.xy * 0.22 * vec2<f32>(1.0 / g.aspect, 1.0), lo, hi);
    let mirrored = textureSample(curr, samp, mirror_uv).rgb;
    let fresnel = 0.04 + 0.96 * pow(1.0 - ndv, 5.0);
    var rgb = mix(body, mirrored * 0.9 + vec3<f32>(0.08), fresnel);

    // Key light up-left, Blinn–Phong; a wide soft lobe and a tight hot one.
    let light = normalize(vec3<f32>(-0.55, 0.75, 0.6));
    let half = normalize(light + view);
    let ndh = clamp(dot(n, half), 0.0, 1.0);
    let hot = pow(ndh, 420.0) * 1.4;
    let soft = pow(ndh, 24.0) * 0.18;
    rgb += vec3<f32>(1.0, 0.97, 0.92) * (hot + soft);

    // Contact darkening at the rim: the sphere sits in the picture, it does not float as a sticker.
    rgb *= 0.72 + 0.28 * smoothstep(0.0, 0.3, ndv);
    return vec4<f32>(mix(scene, rgb, cover), 1.0);
}
