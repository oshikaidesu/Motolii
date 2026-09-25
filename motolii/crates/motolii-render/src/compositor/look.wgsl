// U79 spike: the Look pass — one HDR bright-pass pyramid feeds bloom, halation, a star and an
// anamorphic streak; the composite adds them to the view and splits colour where it means something.
// Bloom chain after Jimenez (SIGGRAPH 2014) as Bevy writes it (bloom.wesl, MIT/Apache-2.0): 13-tap
// Karis-weighted prefilter, 13-tap downsample, 3x3 tent upsample. Star after Kawase (GDC 2003),
// streak after KinoStreak (MIT), halation after utility-dctls Halation (MIT): ideas, not code.

struct Look {
    // threshold, knee, bloom, halation
    p0: vec4f,
    // star, streak, lateral chromatic aberration (px at the frame's corner), plate misregistration (px)
    p1: vec4f,
    // source texel size (xy), target size (zw)
    p2: vec4f,
    // star angle (rad), spectral tint of the glare (0..1), star length (texels), highlight dispersion (px)
    p3: vec4f,
};

@group(0) @binding(0) var<uniform> look: Look;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var src2: texture_2d<f32>;
@group(0) @binding(3) var src3: texture_2d<f32>;
@group(0) @binding(4) var lin: sampler;

struct VsOut {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

@vertex
fn vs_full(@builtin(vertex_index) i: u32) -> VsOut {
    let p = vec2f(f32((i << 1u) & 2u), f32(i & 2u));
    var out: VsOut;
    out.position = vec4f(p * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2f(p.x, 1.0 - p.y);
    return out;
}

fn max3(c: vec3f) -> f32 { return max(c.r, max(c.g, c.b)); }
fn at(t: texture_2d<f32>, uv: vec2f) -> vec3f { return textureSampleLevel(t, lin, uv, 0.0).rgb; }

/// Only light above the threshold (Bevy's soft knee on the brightest channel): white paper stays out.
fn bright(c: vec3f) -> vec3f {
    let br = max3(c);
    let t = look.p0.x;
    let knee = max(look.p0.y, 1e-4);
    var rq = clamp(br - t + knee, 0.0, 2.0 * knee);
    rq = rq * rq / (4.0 * knee);
    return c * max(rq, br - t) / max(br, 1e-4);
}
fn karis(c: vec3f) -> f32 { return 1.0 / (1.0 + max3(c) * 0.25); }

/// Jimenez's 13 taps: five overlapping 2x2 boxes (the centre one twice as heavy).
fn thirteen(t: texture_2d<f32>, uv: vec2f, first: bool) -> vec3f {
    let d = look.p2.xy;
    let a = at(t, uv + d * vec2f(-2.0, -2.0)); let b = at(t, uv + d * vec2f(0.0, -2.0)); let c = at(t, uv + d * vec2f(2.0, -2.0));
    let dd = at(t, uv + d * vec2f(-2.0, 0.0)); let e = at(t, uv); let f = at(t, uv + d * vec2f(2.0, 0.0));
    let g = at(t, uv + d * vec2f(-2.0, 2.0)); let h = at(t, uv + d * vec2f(0.0, 2.0)); let i = at(t, uv + d * vec2f(2.0, 2.0));
    let j = at(t, uv + d * vec2f(-1.0, -1.0)); let k = at(t, uv + d * vec2f(1.0, -1.0));
    let l = at(t, uv + d * vec2f(-1.0, 1.0)); let m = at(t, uv + d * vec2f(1.0, 1.0));
    if first {
        // Karis average per box keeps one hot pixel from becoming a flickering disc.
        var g0 = bright((j + k + l + m) * 0.25); var g1 = bright((a + b + dd + e) * 0.25);
        var g2 = bright((b + c + e + f) * 0.25); var g3 = bright((dd + e + g + h) * 0.25);
        var g4 = bright((e + f + h + i) * 0.25);
        let w0 = karis(g0) * 0.5; let w1 = karis(g1) * 0.125; let w2 = karis(g2) * 0.125;
        let w3 = karis(g3) * 0.125; let w4 = karis(g4) * 0.125;
        return (g0 * w0 + g1 * w1 + g2 * w2 + g3 * w3 + g4 * w4) / (w0 + w1 + w2 + w3 + w4);
    }
    return e * 0.125 + (a + c + g + i) * 0.03125 + (b + dd + f + h) * 0.0625 + (j + k + l + m) * 0.125;
}

@fragment
fn fs_prefilter(in: VsOut) -> @location(0) vec4f {
    return vec4f(thirteen(src, in.uv, true), 1.0);
}

@fragment
fn fs_down(in: VsOut) -> @location(0) vec4f {
    return vec4f(thirteen(src, in.uv, false), 1.0);
}

/// This level plus the wider one below it, spread by a 3x3 tent.
@fragment
fn fs_up(in: VsOut) -> @location(0) vec4f {
    let d = look.p2.xy;
    var s = at(src2, in.uv) * 4.0;
    s += (at(src2, in.uv + vec2f(-d.x, 0.0)) + at(src2, in.uv + vec2f(d.x, 0.0)) + at(src2, in.uv + vec2f(0.0, -d.y)) + at(src2, in.uv + vec2f(0.0, d.y))) * 2.0;
    s += at(src2, in.uv - d) + at(src2, in.uv + d) + at(src2, in.uv + vec2f(d.x, -d.y)) + at(src2, in.uv + vec2f(-d.x, d.y));
    return vec4f(at(src, in.uv) + s / 16.0, 1.0);
}

/// A hue along a streak (0 = the core): diffraction spreads blue in, red out.
fn spectrum(t: f32) -> vec3f {
    let c = clamp(abs(fract(vec3f(0.62, 0.29, 0.95) - t * 0.9) * 6.0 - 3.0) - 1.0, vec3f(0.0), vec3f(1.0));
    return c * c;
}

/// What a star grows from: a light brighter than its surroundings (a sparkle, a glint), not a lit
/// area (a neon word, a softbox seen in chrome), which only glows.
fn point(uv: vec2f) -> vec3f {
    return max(at(src, uv) - at(src3, uv) * 1.5, vec3f(0.0));
}

/// The star (a six-bladed aperture's spikes) from the eighth-size bright level, and the anamorphic
/// streak from the sixteenth-size one. Gathered: each pixel collects what shines along the blades, one
/// source texel per tap so a blade is a line, not a string of beads.
@fragment
fn fs_star(in: VsOut) -> @location(0) vec4f {
    let d = vec2f(1.0) / vec2f(textureDimensions(src));
    let spectral = look.p3.y;
    var star = vec3f(0.0);
    if look.p1.x > 0.0 {
        let taps = i32(clamp(look.p3.z, 2.0, 48.0));
        for (var axis = 0; axis < 3; axis += 1) {
            let a = look.p3.x + f32(axis) * 1.0471976;
            let dir = vec2f(cos(a), sin(a)) * d;
            for (var i = 1; i <= taps; i += 1) {
                let t = f32(i) / f32(taps);
                // Metal's fast math: i / taps can land a hair above 1, and pow of a negative is NaN.
                let w = pow(max(1.0 - t, 0.0), 1.6);
                let tint = mix(vec3f(1.0), spectrum(t) * 2.2, spectral * 0.7 * smoothstep(0.3, 0.9, t));
                star += (point(in.uv + dir * f32(i)) + point(in.uv - dir * f32(i))) * w * tint;
            }
        }
        star *= look.p1.x * 3.0 / f32(taps);
    }
    var streak = vec3f(0.0);
    if look.p1.y > 0.0 {
        let d2 = vec2f(1.0) / vec2f(textureDimensions(src2));
        // Only from point-like lights too: a lit area does not smear a line across the frame.
        for (var i = -24; i <= 24; i += 1) {
            let w = exp(-abs(f32(i)) / 6.0);
            let uv = in.uv + vec2f(f32(i) * d2.x, 0.0);
            streak += max(at(src2, uv) - at(src3, uv) * 1.2, vec3f(0.0)) * w;
        }
        streak *= look.p1.y / 6.0 * vec3f(0.55, 0.75, 1.25);
    }
    return vec4f(star + streak, 1.0);
}

fn luma(c: vec3f) -> f32 { return dot(c, vec3f(0.2126, 0.7152, 0.0722)); }

/// The view with its glare, and colour split where it means something: a lens's lateral aberration
/// (grows toward the corners, only visible at edges), misregistered print plates (a constant offset per
/// ink), and the glare itself dispersed (white highlights open into a rainbow rim).
@fragment
fn fs_composite(in: VsOut) -> @location(0) vec4f {
    let size = look.p2.zw;
    let px = 1.0 / size;
    let aspect = size.x / size.y;
    var r = in.uv - 0.5;
    r.x *= aspect;
    let radial = r / (0.5 * sqrt(aspect * aspect + 1.0));
    let lateral = radial * dot(radial, radial) * look.p1.z;
    let plate = look.p1.w;
    let o_r = (lateral + vec2f(0.8, 0.35) * plate) * px;
    let o_b = (-lateral + vec2f(-0.3, -0.8) * plate) * px;
    let o_g = vec2f(-0.45, 0.4) * plate * px;
    let centre = textureSampleLevel(src, lin, in.uv + o_g, 0.0);
    let scene = vec3f(textureSampleLevel(src, lin, in.uv + o_r, 0.0).r, centre.g, textureSampleLevel(src, lin, in.uv + o_b, 0.0).b);

    // Glare: bloom (the pyramid), its red film halation, the star and streak; dispersed a little.
    let disp = (normalize(r + vec2f(1e-5)) * look.p3.w) * px;
    let bloom = vec3f(at(src2, in.uv + disp * 2.0).r, at(src2, in.uv).g, at(src2, in.uv - disp * 2.0).b);
    // Film halation: red light scattered back from the base, seen around a highlight, not over it.
    let halation = luma(bloom) * vec3f(1.0, 0.24, 0.05) * (1.0 - smoothstep(0.5, 1.0, max3(scene)));
    let glint = vec3f(at(src3, in.uv + disp).r, at(src3, in.uv).g, at(src3, in.uv - disp).b);
    let glare = bloom * look.p0.z + halation * look.p0.w + glint;

    var out = scene + glare;
    // Above white a saturated colour keeps its hue (a neon word stays pink instead of clipping to a
    // tint), a near-neutral highlight goes to white (the faint tint of added glare is not magnified),
    // and a far brighter core burns toward white, the way film does.
    let peak = max3(out);
    if peak > 1.0 {
        let hue = out / peak;
        let saturation = 1.0 - min(hue.r, min(hue.g, hue.b));
        let white = max(1.0 - smoothstep(0.0, 0.5, saturation), smoothstep(3.0, 16.0, peak) * 0.7);
        out = mix(hue, vec3f(1.0), white);
    }
    let alpha = max(centre.a, clamp(max3(glare), 0.0, 1.0));
    return vec4f(out, alpha);
}
