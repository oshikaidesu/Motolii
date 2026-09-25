// Scratch Lighting Pack (BB theater). Surface: each picture is a world quad; its coverage is its alpha.
// Light: a key light (the environment's sun) with contact-hardening shadows cone-traced through the
// pictures' coverage, pictures that give light as textured area lights, and one bounce gathered by
// cones from every picture in the world. Image formation: exposure, AgX. History-free: every frame
// is computed from the frame's pictures alone.

struct Globals {
    clip_from_world: mat4x4f,
    eye: vec4f,
    sun_dir: vec4f,   // xyz toward the sun, w: the environment's sun share
    sun_color: vec4f, // rgb
    params: vec4f,    // x quad count, y lit, z film
    window: vec4f,    // xy size, z has background
    background: vec4f,
    grid_min: vec4f,  // the light field's first probe
    grid_cell: vec4f, // and the step between probes
    world_from_clip: mat4x4f,
};
struct Quad {
    corner: vec4f, // w slot
    u: vec4f,      // w |u|
    v: vec4f,      // w |v|
    n: vec4f,      // w flags: 1 blocks the key light, 2 gives light
    info: vec4f,   // x decode, y opacity, z emission
};

@group(0) @binding(0) var<uniform> g: Globals;
@group(0) @binding(1) var<storage, read> quads: array<Quad>;
@group(0) @binding(2) var atlas: texture_2d<f32>;
@group(0) @binding(3) var lin: sampler;
@group(1) @binding(0) var picture: texture_2d<f32>;

// ---- look (the pack's own taste; tuned by eye) ------------------------------------------------
const KEY_IRRADIANCE: f32 = 8.0;       // the key light on a surface facing it
const KEY_TAN: f32 = 0.055;            // the key light's size (tan of its radius): penumbra grows with distance
const SKY_UP: vec3f = vec3f(0.27, 0.33, 0.44);  // cool   // the room's light from above (radiance)
const SKY_DOWN: vec3f = vec3f(0.15, 0.12, 0.09); // warm // and from below
const EMISSION_SCALE: f32 = 8.0;       // Glow intensity -> radiance it throws on the room
const EMISSION_SELF: f32 = 0.3;        // and how bright the picture itself looks
const EXPOSURE: f32 = 1.55;
const LOOK_POWER: f32 = 1.12;           // AgX look: a little contrast
const LOOK_SATURATION: f32 = 1.12;      // and colour
const PROBE_CONES: u32 = 48u;        // per probe, over the whole sphere
const PROBE_CONE_TAN: f32 = 0.36;
const PROBES: vec3u = vec3u(22u, 11u, 22u);
const AO_CONES: u32 = 6u;              // contact shadow near a surface, where the field is too coarse
const AO_TAN: f32 = 0.6;
const AO_REACH: f32 = 160.0;
const AIR: f32 = 2.5e-4;               // how much the room's air scatters (per px of path)
const AIR_STEPS: u32 = 16u;
const AIR_GLOW: f32 = 6.0e-5;          // and around the pictures that give light

const PI: f32 = 3.14159265;
const SLOT: f32 = 128.0;
const ATLAS: f32 = 2048.0;

fn quad_count() -> u32 { return u32(g.params.x); }
fn has_flag(q: Quad, bit: u32) -> bool { return (u32(q.n.w) & bit) != 0u; }

fn srgb_to_linear(c: vec3f) -> vec3f {
    return select(pow((c + 0.055) / 1.055, vec3f(2.4)), c / 12.92, c <= vec3f(0.04045));
}

/// A picture's texel as premultiplied scene-linear colour, by how the host uploaded it.
fn decode(c: vec4f, mode: f32) -> vec4f {
    if mode > 1.5 { return c; }
    if mode > 0.5 { return vec4f(srgb_to_linear(c.rgb) * c.a, c.a); }
    return c;
}

// ---- atlas ------------------------------------------------------------------------------------
fn atlas_sample(slot: f32, uv: vec2f, lod: f32) -> vec4f {
    let s = u32(slot);
    let origin = vec2f(f32(s % 16u), f32(s / 16u)) * SLOT;
    let half_texel = 0.5 * exp2(lod) / SLOT;
    let local = clamp(uv, vec2f(half_texel), vec2f(1.0 - half_texel));
    return textureSampleLevel(atlas, lin, (origin + local * SLOT) / ATLAS, lod);
}

struct VOut { @builtin(position) clip: vec4f, @location(0) uv: vec2f, @location(1) world: vec3f, @location(2) @interpolate(flat) index: u32 };

@vertex fn vs_slot(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    let p = vec2f(f32((vi << 1u) & 2u), f32(vi & 2u));
    var o: VOut;
    o.clip = vec4f(p * vec2f(2.0, -2.0) + vec2f(-1.0, 1.0), 0.0, 1.0);
    o.uv = p;
    o.index = ii;
    return o;
}
@fragment fn fs_blit(in: VOut) -> @location(0) vec4f {
    let q = quads[in.index];
    return decode(textureSampleLevel(picture, lin, in.uv, 0.0), q.info.x) * q.info.y;
}

@vertex fn vs_full(@builtin(vertex_index) vi: u32) -> VOut {
    let p = vec2f(f32((vi << 1u) & 2u), f32(vi & 2u));
    var o: VOut;
    o.clip = vec4f(p * vec2f(2.0, -2.0) + vec2f(-1.0, 1.0), 0.0, 1.0);
    o.uv = p;
    return o;
}
@group(0) @binding(0) var level_in: texture_2d<f32>;
@group(0) @binding(1) var level_sampler: sampler;
@fragment fn fs_downsample(in: VOut) -> @location(0) vec4f {
    return textureSampleLevel(level_in, level_sampler, in.uv, 0.0);
}

// ---- rays through the pictures ----------------------------------------------------------------
struct Hit { t: f32, cover: f32, index: u32, uv: vec2f, lod: f32 };

/// Where a cone of half-width `tan_half` along `d` meets picture `i`, and how much of it the picture covers there.
fn cone_hit(i: u32, o: vec3f, d: vec3f, tan_half: f32, t_max: f32) -> Hit {
    var h: Hit;
    h.cover = 0.0;
    let q = quads[i];
    let denom = dot(d, q.n.xyz);
    if abs(denom) < 1e-5 { return h; }
    let t = dot(q.corner.xyz - o, q.n.xyz) / denom;
    if t <= 0.5 || t >= t_max { return h; }
    let local = o + d * t - q.corner.xyz;
    let uv = vec2f(dot(local, q.u.xyz) / (q.u.w * q.u.w), dot(local, q.v.xyz) / (q.v.w * q.v.w));
    let r = t * tan_half + 1.0;
    let outside = max(max(-uv.x, uv.x - 1.0) * q.u.w, max(-uv.y, uv.y - 1.0) * q.v.w);
    if outside > r { return h; }
    let lod = clamp(log2(max(2.0 * r * SLOT / min(q.u.w, q.v.w), 1.0)), 0.0, 7.0);
    let edge = 1.0 - smoothstep(-r, r, outside);
    h.t = t;
    h.index = i;
    h.uv = uv;
    h.lod = lod;
    h.cover = atlas_sample(q.corner.w, uv, lod).a * edge;
    return h;
}

/// How much of the key light (or an emitter) gets from `o` along `d`: the pictures that block light, cone-filtered.
fn transmittance(o: vec3f, d: vec3f, t_max: f32, tan_half: f32, skip: u32, skip2: u32) -> f32 {
    var through = 1.0;
    for (var i = 0u; i < quad_count(); i++) {
        if i == skip || i == skip2 || !has_flag(quads[i], 1u) { continue; }
        let h = cone_hit(i, o, d, tan_half, t_max);
        through *= 1.0 - h.cover;
    }
    return through;
}

fn sky(d: vec3f) -> vec3f {
    // world y is down: -y is up
    return mix(SKY_DOWN, SKY_UP, smoothstep(-0.4, 0.4, -d.y));
}

fn key_dir() -> vec3f { return normalize(g.sun_dir.xyz); }
fn key_color() -> vec3f { return g.sun_color.rgb * KEY_IRRADIANCE; }

/// Irradiance at `p` (normal `n`) from the pictures that give light: each an area light of its mean colour.
fn emitted_irradiance(p: vec3f, n: vec3f, skip: u32, visibility: bool) -> vec3f {
    var e = vec3f(0.0);
    for (var i = 0u; i < quad_count(); i++) {
        let q = quads[i];
        if i == skip || !has_flag(q, 2u) { continue; }
        let mean = atlas_sample(q.corner.w, vec2f(0.5), 7.0);
        let radiance = mean.rgb * q.info.z * EMISSION_SCALE;
        let c0 = q.corner.xyz - p;
        let v0 = normalize(c0);
        let v1 = normalize(c0 + q.u.xyz);
        let v2 = normalize(c0 + q.u.xyz + q.v.xyz);
        let v3 = normalize(c0 + q.v.xyz);
        var f = vec3f(0.0);
        f += acos(clamp(dot(v0, v1), -1.0, 1.0)) * normalize(cross(v0, v1));
        f += acos(clamp(dot(v1, v2), -1.0, 1.0)) * normalize(cross(v1, v2));
        f += acos(clamp(dot(v2, v3), -1.0, 1.0)) * normalize(cross(v2, v3));
        f += acos(clamp(dot(v3, v0), -1.0, 1.0)) * normalize(cross(v3, v0));
        f *= 0.5;
        let centre = c0 + 0.5 * (q.u.xyz + q.v.xyz);
        if dot(f, centre) < 0.0 { f = -f; }
        let form = max(dot(f, n), 0.0);
        if form <= 0.0 { continue; }
        var vis = 1.0;
        if visibility {
            let dist = length(centre);
            let size = 0.5 * max(q.u.w, q.v.w);
            vis = transmittance(p + n * 0.5, centre / dist, dist - 1.0, size / dist, skip, i);
        }
        e += radiance * form * vis;
    }
    return e;
}

/// Light arriving at a surface point straight from the key light and the pictures that give light.
fn direct(p: vec3f, n: vec3f, skip: u32, visibility: bool) -> vec3f {
    var e = vec3f(0.0);
    let l = key_dir();
    let ndl = dot(n, l);
    if ndl > 0.0 {
        e += key_color() * ndl * transmittance(p + n * 0.5, l, 1e7, KEY_TAN, skip, skip);
    }
    return e + emitted_irradiance(p, n, skip, visibility);
}

struct Surface { albedo: vec3f, p: vec3f, n: vec3f };

/// The picture a cone hit, at the hit: its colour there (filtered to the cone), where, and facing the ray.
fn hit_surface(h: Hit, o: vec3f, d: vec3f) -> Surface {
    let q = quads[h.index];
    let texel = atlas_sample(q.corner.w, h.uv, h.lod);
    var n = q.n.xyz;
    if dot(n, d) > 0.0 { n = -n; }
    return Surface(texel.rgb / max(texel.a, 1e-4), o + d * h.t, n);
}

struct Two { first: Hit, second: Hit };

/// The nearest two pictures a cone meets.
fn nearest_two(p: vec3f, d: vec3f, tan_half: f32, skip: u32) -> Two {
    var first: Hit; first.cover = 0.0; first.t = 1e9;
    var second: Hit; second.cover = 0.0; second.t = 1e9;
    for (var i = 0u; i < quad_count(); i++) {
        if i == skip { continue; }
        let h = cone_hit(i, p, d, tan_half, 1e7);
        if h.cover <= 0.01 { continue; }
        if h.t < first.t { second = first; first = h; } else if h.t < second.t { second = h; }
    }
    return Two(first, second);
}

/// Light leaving a hit picture towards the probe, first bounce: the key light and the emitters only.
fn bounce_first(h: Hit, o: vec3f, d: vec3f) -> vec3f {
    let s = hit_surface(h, o, d);
    return s.albedo * direct(s.p, s.n, h.index, false) / PI;
}

fn cone_radiance_first(p: vec3f, d: vec3f) -> vec3f {
    let two = nearest_two(p, d, PROBE_CONE_TAN, 0xffffffffu);
    var l = sky(d);
    if two.second.cover > 0.0 { l = mix(l, bounce_first(two.second, p, d), two.second.cover); }
    if two.first.cover > 0.0 { l = mix(l, bounce_first(two.first, p, d), two.first.cover); }
    return l;
}

struct ProbeOut { @location(0) r: vec4f, @location(1) g: vec4f, @location(2) b: vec4f };

fn probe_position(frag: vec4f) -> vec3f {
    let texel = vec2u(frag.xy);
    let index = vec3u(texel.x % PROBES.x, texel.x / PROBES.x, texel.y);
    return g.grid_min.xyz + vec3f(index) * g.grid_cell.xyz;
}

fn probe_direction(k: u32) -> vec3f {
    let z = 1.0 - (2.0 * f32(k) + 1.0) / f32(PROBE_CONES);
    let rad = sqrt(max(1.0 - z * z, 0.0));
    let phi = f32(k) * 2.39996323;
    return vec3f(rad * cos(phi), z, rad * sin(phi));
}

fn project(l: vec3f, d: vec3f, o: ptr<function, ProbeOut>) {
    let basis = vec4f(0.282095, 0.488603 * d.x, 0.488603 * d.y, 0.488603 * d.z) * (4.0 * PI / f32(PROBE_CONES));
    (*o).r += l.r * basis; (*o).g += l.g * basis; (*o).b += l.b * basis;
}

/// The first light field: cones over the sphere (a fixed Fibonacci set), projected on L1 harmonics.
@fragment fn fs_probe_first(in: VOut) -> ProbeOut {
    let p = probe_position(in.clip);
    var o = ProbeOut(vec4f(0.0), vec4f(0.0), vec4f(0.0));
    for (var k = 0u; k < PROBE_CONES; k++) {
        let d = probe_direction(k);
        project(cone_radiance_first(p, d), d, &o);
    }
    return o;
}

@group(2) @binding(0) var probe_r: texture_2d<f32>;
@group(2) @binding(1) var probe_g: texture_2d<f32>;
@group(2) @binding(2) var probe_b: texture_2d<f32>;

/// Irradiance from the light field at `p` facing `n`: the eight probes around, none from behind the surface.
fn field_irradiance(p: vec3f, n: vec3f) -> vec3f {
    let q = p + n * 12.0;
    let f = clamp((q - g.grid_min.xyz) / g.grid_cell.xyz, vec3f(0.0), vec3f(PROBES - 1u) - 1e-3);
    let base = vec3u(floor(f));
    let frac = f - floor(f);
    var sum = vec3f(0.0);
    var weight = 0.0;
    for (var c = 0u; c < 8u; c++) {
        let o = vec3u(c & 1u, (c >> 1u) & 1u, (c >> 2u) & 1u);
        let idx = min(base + o, PROBES - 1u);
        let probe_p = g.grid_min.xyz + vec3f(idx) * g.grid_cell.xyz;
        let tri = select(1.0 - frac, frac, vec3<bool>(o == vec3u(1u)));
        let to_probe = probe_p - p;
        let facing = dot(n, to_probe) / max(length(to_probe), 1e-3);
        let w = tri.x * tri.y * tri.z * smoothstep(-0.05, 0.15, facing);
        if w <= 0.0 { continue; }
        let t = vec2i(i32(idx.x + idx.y * PROBES.x), i32(idx.z));
        let sr = textureLoad(probe_r, t, 0); let sg = textureLoad(probe_g, t, 0); let sb = textureLoad(probe_b, t, 0);
        let a0 = PI * 0.282095;
        let a1 = 2.0 * PI / 3.0 * 0.488603;
        let e = vec3f(a0 * sr.x + a1 * dot(sr.yzw, n), a0 * sg.x + a1 * dot(sg.yzw, n), a0 * sb.x + a1 * dot(sb.yzw, n));
        sum += max(e, vec3f(0.0)) * w;
        weight += w;
    }
    return sum / max(weight, 1e-4);
}

/// Light leaving a hit picture, second bounce: lit also by the first light field.
fn bounce_field(h: Hit, o: vec3f, d: vec3f) -> vec3f {
    let s = hit_surface(h, o, d);
    return s.albedo * (direct(s.p, s.n, h.index, false) + field_irradiance(s.p, s.n)) / PI;
}

fn cone_radiance_field(p: vec3f, d: vec3f) -> vec3f {
    let two = nearest_two(p, d, PROBE_CONE_TAN, 0xffffffffu);
    var l = sky(d);
    if two.second.cover > 0.0 { l = mix(l, bounce_field(two.second, p, d), two.second.cover); }
    if two.first.cover > 0.0 { l = mix(l, bounce_field(two.first, p, d), two.first.cover); }
    return l;
}

/// The second light field, read through the first: light that has bounced twice.
@fragment fn fs_probe(in: VOut) -> ProbeOut {
    let p = probe_position(in.clip);
    var o = ProbeOut(vec4f(0.0), vec4f(0.0), vec4f(0.0));
    for (var k = 0u; k < PROBE_CONES; k++) {
        let d = probe_direction(k);
        project(cone_radiance_field(p, d), d, &o);
    }
    return o;
}

/// How open the space right around a surface is (1 = nothing near): short cones against the pictures.
fn contact_openness(p: vec3f, n: vec3f, skip: u32) -> f32 {
    let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(n.y) > 0.9);
    let t = normalize(cross(up, n));
    let b = cross(n, t);
    var open = 0.0;
    for (var k = 0u; k < AO_CONES; k++) {
        let fk = (f32(k) + 0.5) / f32(AO_CONES);
        let r = sqrt(fk);
        let phi = f32(k) * 2.39996323;
        let d = normalize(t * (r * cos(phi)) + b * (r * sin(phi)) + n * sqrt(max(1.0 - fk, 0.0)));
        var through = 1.0;
        for (var i = 0u; i < quad_count(); i++) {
            if i == skip { continue; }
            let h = cone_hit(i, p + n * 0.5, d, AO_TAN, AO_REACH);
            through *= 1.0 - h.cover * (1.0 - smoothstep(0.3 * AO_REACH, AO_REACH, h.t));
        }
        open += through;
    }
    return open / f32(AO_CONES);
}

// ---- the air -------------------------------------------------------------------------------------
/// Interleaved gradient noise: a fixed per-pixel offset for the air's samples (no history).
fn ign(px: vec2f) -> f32 { return fract(52.9829189 * fract(dot(px, vec2f(0.06711056, 0.00583715)))); }

/// The key light at `x` only where it came through a picture that blocks light (a window, a gobo):
/// the air glows in the shafts a picture's holes let through, and stays clear in the open.
fn shaft(x: vec3f) -> f32 {
    let l = key_dir();
    var through = 1.0;
    var crossed = false;
    for (var i = 0u; i < quad_count(); i++) {
        let q = quads[i];
        if !has_flag(q, 1u) { continue; }
        let denom = dot(l, q.n.xyz);
        if abs(denom) < 1e-5 { continue; }
        let t = dot(q.corner.xyz - x, q.n.xyz) / denom;
        if t <= 0.0 { continue; }
        let local = x + l * t - q.corner.xyz;
        let uv = vec2f(dot(local, q.u.xyz) / (q.u.w * q.u.w), dot(local, q.v.xyz) / (q.v.w * q.v.w));
        if any(uv < vec2f(0.0)) || any(uv > vec2f(1.0)) { continue; }
        crossed = true;
        let r = t * KEY_TAN + 1.0;
        let lod = clamp(log2(max(2.0 * r * SLOT / min(q.u.w, q.v.w), 1.0)), 0.0, 7.0);
        through *= 1.0 - atlas_sample(q.corner.w, uv, lod).a;
    }
    return select(0.0, through, crossed);
}

/// Light the air scatters towards the eye between the eye and `p`: the key light where it reaches
/// the air (shafts through the window), and the glow around the pictures that give light.
fn air_light(p: vec3f, px: vec2f) -> vec3f {
    let eye = g.eye.xyz;
    let seg = p - eye;
    let len = length(seg);
    let d = seg / len;
    let dt = len / f32(AIR_STEPS);
    let j = ign(px);
    var key = 0.0;
    for (var k = 0u; k < AIR_STEPS; k++) {
        let x = eye + d * ((f32(k) + j) * dt);
        key += shaft(x);
    }
    let sum = key_color() * key * dt;
    var glow = vec3f(0.0);
    for (var i = 0u; i < quad_count(); i++) {
        let q = quads[i];
        if !has_flag(q, 2u) { continue; }
        let mean = atlas_sample(q.corner.w, vec2f(0.5), 7.0).rgb * q.info.z * EMISSION_SCALE;
        let centre = q.corner.xyz + 0.5 * (q.u.xyz + q.v.xyz);
        // a large soft emitter (a view outside) lights the room, but only a small one glows in the air
        let intensity = mean * min(q.u.w * q.v.w, 2.0e5);
        let t0 = dot(centre - eye, d);
        let h = max(length(centre - eye - d * t0), 0.25 * min(q.u.w, q.v.w));
        // seen from where the ray passes closest: a wall between keeps the glow on its side
        let x0 = eye + d * clamp(t0, 0.0, len);
        let to_c = centre - x0;
        let dist = length(to_c);
        let vis = transmittance(x0, to_c / max(dist, 1e-3), dist - 1.0, 0.5 * max(q.u.w, q.v.w) / max(dist, 1.0), i, i);
        glow += intensity * vis * (atan((len - t0) / h) - atan(-t0 / h)) / h;
    }
    return (sum * AIR + glow * AIR_GLOW) / (4.0 * PI);
}

@group(3) @binding(0) var near_light: texture_2d<f32>;
@group(3) @binding(1) var near_sampler: sampler;

// ---- the run's pictures -----------------------------------------------------------------------
@vertex fn vs_quad(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VOut {
    var corners = array<vec2f, 6>(vec2f(0.0, 0.0), vec2f(1.0, 0.0), vec2f(0.0, 1.0), vec2f(0.0, 1.0), vec2f(1.0, 0.0), vec2f(1.0, 1.0));
    let c = corners[vi];
    let q = quads[ii];
    let p = q.corner.xyz + q.u.xyz * c.x + q.v.xyz * c.y;
    var o: VOut;
    o.clip = g.clip_from_world * vec4f(p, 1.0);
    o.uv = c;
    o.world = p;
    o.index = ii;
    return o;
}

fn surface(in: VOut) -> vec4f {
    let q = quads[in.index];
    return decode(textureSample(picture, lin, in.uv), q.info.x) * q.info.y;
}

/// The front surface: its normal (facing the eye) and which picture it is (+1; 0 = none).
@fragment fn fs_depth(in: VOut) -> @location(0) vec4f {
    if surface(in).a < 0.5 { discard; }
    var n = quads[in.index].n.xyz;
    if dot(n, g.eye.xyz - in.world) < 0.0 { n = -n; }
    return vec4f(n, f32(in.index + 1u));
}

@group(1) @binding(0) var front: texture_2d<f32>;
@group(1) @binding(1) var front_depth: texture_depth_2d;

/// At half size, from the front surfaces: how open the space around them is (a), and the light the
/// air between them and the eye scatters (rgb). Both change slowly across the picture.
@fragment fn fs_screen(in: VOut) -> @location(0) vec4f {
    let px = vec2i(in.clip.xy) * 2;
    let f = textureLoad(front, px, 0);
    if f.w < 0.5 { return vec4f(0.0, 0.0, 0.0, 1.0); }
    let depth = textureLoad(front_depth, px, 0);
    let size = g.window.xy;
    let ndc = vec2f((f32(px.x) + 0.5) / size.x * 2.0 - 1.0, 1.0 - (f32(px.y) + 0.5) / size.y * 2.0);
    let h = g.world_from_clip * vec4f(ndc, depth, 1.0);
    let p = h.xyz / h.w;
    let index = u32(f.w + 0.5) - 1u;
    return vec4f(air_light(p, in.clip.xy), contact_openness(p, f.xyz, index));
}

@fragment fn fs_color(in: VOut) -> @location(0) vec4f {
    let c = surface(in);
    if c.a < 1.0 / 255.0 { discard; }
    let q = quads[in.index];
    let albedo = c.rgb / c.a;
    if g.params.y < 0.5 {
        return c;
    }
    var n = q.n.xyz;
    if dot(n, g.eye.xyz - in.world) < 0.0 { n = -n; }
    let e = direct(in.world, n, in.index, true);
    let near = textureSampleLevel(near_light, near_sampler, in.clip.xy / g.window.xy, 0.0);
    let around = field_irradiance(in.world, n) * near.a;
    var rgb = albedo * (e + around) / PI;
    rgb += near.rgb;
    rgb += albedo * q.info.z * EMISSION_SELF;
    return vec4f(rgb * c.a, c.a);
}

// ---- image formation --------------------------------------------------------------------------
@group(0) @binding(1) var hdr_in: texture_2d<f32>;
@group(0) @binding(2) var hdr_sampler: sampler;

fn agx_contrast(x: vec3f) -> vec3f {
    let x2 = x * x;
    let x4 = x2 * x2;
    return 15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4 - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - 0.00232;
}
fn agx(c: vec3f) -> vec3f {
    let inset = mat3x3f(0.842479062253094, 0.0423282422610123, 0.0423756549057051,
                        0.0784335999999992, 0.878468636469772, 0.0784336,
                        0.0792237451477643, 0.0791661274605434, 0.879142973793104);
    let outset = mat3x3f(1.19687900512017, -0.0528968517574562, -0.0529716355144438,
                         -0.0980208811401368, 1.15190312990417, -0.0980434501171241,
                         -0.0990297440797205, -0.0989611768448433, 1.15107367264116);
    let min_ev = -12.47393;
    let max_ev = 4.026069;
    var v = inset * max(c, vec3f(1e-10));
    v = clamp(log2(v), vec3f(min_ev), vec3f(max_ev));
    v = (v - min_ev) / (max_ev - min_ev);
    v = agx_contrast(v);
    // look (AgX's "punchy" family, gentler)
    v = pow(max(v, vec3f(0.0)), vec3f(LOOK_POWER));
    let luma = dot(v, vec3f(0.2126, 0.7152, 0.0722));
    v = luma + LOOK_SATURATION * (v - luma);
    v = outset * v;
    return pow(max(v, vec3f(0.0)), vec3f(2.2));
}

@fragment fn fs_formation(in: VOut) -> @location(0) vec4f {
    let c = textureSampleLevel(hdr_in, hdr_sampler, in.uv, 0.0);
    var rgb = vec3f(0.0);
    if c.a > 0.0 {
        // clamp: the host's own image formation today (saturate the scene-linear value, no exposure)
        let straight = c.rgb / c.a;
        rgb = select(clamp(straight, vec3f(0.0), vec3f(1.0)), agx(straight * EXPOSURE), g.params.z > 0.5) * c.a;
    }
    var a = c.a;
    if g.window.z > 0.5 {
        let bg = srgb_to_linear(g.background.rgb) * g.background.a;
        rgb += (1.0 - a) * bg;
        a += (1.0 - a) * g.background.a;
    }
    return vec4f(rgb, a);
}
