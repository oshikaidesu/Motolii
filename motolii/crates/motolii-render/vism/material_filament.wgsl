// Motolii's lit model: Filament's (evaluateIBL + getPixelParams, google/filament@41f996de, Apache-2.0)
// as `shade_surface`. The BRDF / SH / refraction / tone functions come through naga
// (material_filament_naga.wgsl, prefix fil_); the DFG LUT and studio SH are baked (material_filament_baked.wgsl).
// This file is the glue: Filament's evaluateIBL() order, re-expressed over Motolii's frame bindings,
// plus the built-in studio environment used when no environment is bound.

const FIL_MIN_PERCEPTUAL_ROUGHNESS: f32 = 0.045;
/// Exposure of the built-in studio (the SH and the softboxes share it).
const STUDIO_EXPOSURE: f32 = 0.85;
/// Perceptual roughness of a clear coat (a sheet's Clearcoat sets how much).
const FIL_CLEAR_COAT_ROUGHNESS: f32 = 0.06;

// ---- Filament PrefilteredDFG_LUT(): the baked LUT, bilinear ----
fn fil_prefiltered_dfg(perceptual_roughness: f32, n_dot_v: f32) -> vec2f {
    let n = f32(DFG_SIZE);
    let p = clamp(vec2f(n_dot_v, perceptual_roughness) * n - 0.5, vec2f(0.0), vec2f(n - 1.0));
    let i = vec2u(floor(p));
    let j = min(i + 1u, vec2u(DFG_SIZE - 1u));
    let f = fract(p);
    let a = DFG_LUT[i.y * DFG_SIZE + i.x];
    let b = DFG_LUT[i.y * DFG_SIZE + j.x];
    let c = DFG_LUT[j.y * DFG_SIZE + i.x];
    let d = DFG_LUT[j.y * DFG_SIZE + j.x];
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

// ---- The built-in studio (view space: +x right, +y up, +z toward the camera). Mirrors bake.py. ----
fn studio_erf(x: f32) -> f32 {
    let a = 0.147;
    let x2 = x * x;
    return sign(x) * sqrt(1.0 - exp(-x2 * (1.2732395 + a * x2) / (1.0 + a * x2)));
}
fn studio_box1(x: f32, h: f32, s: f32) -> f32 {
    let k = 0.70710678 / s;
    return 0.5 * (studio_erf((h - x) * k) + studio_erf((h + x) * k));
}
fn studio_softbox(d: vec3f, c: vec3f, half_size: vec2f, sigma: f32) -> f32 {
    let z = dot(d, c);
    if z <= 0.0 { return 0.0; }
    var r = vec3f(1.0, 0.0, 0.0);
    if abs(c.y) < 0.999 { r = normalize(cross(vec3f(0.0, 1.0, 0.0), c)); }
    let u = cross(c, r);
    let p = vec2f(dot(d, r), dot(d, u)) / max(z, 1e-4);
    let s = 0.03 + sigma;
    return studio_box1(p.x, half_size.x, s) * studio_box1(p.y, half_size.y, s);
}
/// Radiance of the studio along a view-space direction, blurred by `sigma` (tangent-plane units).
fn studio_radiance(d: vec3f, sigma: f32) -> vec3f {
    let w = 0.08 + sigma;
    let t = smoothstep(-w, w, d.y);
    let sky = 0.10 + 0.10 * clamp(d.y, 0.0, 1.0);
    let floor_ = 0.11 - 0.05 * clamp(-d.y, 0.0, 1.0);
    var out = vec3f(mix(floor_, sky, t));
    out += studio_softbox(d, normalize(vec3f(-0.55, 0.55, 0.63)), vec2f(0.42, 0.55), sigma) * vec3f(3.4, 3.3, 3.1);
    out += studio_softbox(d, normalize(vec3f(0.0, 1.0, 0.1)), vec2f(1.1, 0.35), sigma) * vec3f(1.3);
    out += studio_softbox(d, normalize(vec3f(-0.9, 0.12, -0.42)), vec2f(0.1, 1.3), sigma) * vec3f(2.6, 2.7, 2.9);
    out += studio_softbox(d, normalize(vec3f(0.9, 0.2, -0.4)), vec2f(0.1, 1.3), sigma) * vec3f(2.6, 2.7, 2.9);
    out += studio_softbox(d, normalize(vec3f(0.85, 0.05, 0.55)), vec2f(0.5, 0.7), sigma) * vec3f(0.55, 0.57, 0.6);
    out += studio_softbox(d, normalize(vec3f(0.15, 0.3, 1.0)), vec2f(0.6, 0.45), sigma) * vec3f(1.0);
    return out * STUDIO_EXPOSURE;
}
fn view_space_direction(world_dir: vec3f) -> vec3f {
    return normalize(frame.view_from_world * vec4f(world_dir, 0.0));
}

// ---- Filament prefilteredRadiance() / diffuseIrradiance() over what the frame has ----
fn fil_prefiltered_radiance(world_dir: vec3f, perceptual_roughness: f32) -> vec3f {
    if frame.environment_present == 1u {
        return environment_specular_along(world_dir, perceptual_roughness);
    }
    let alpha = perceptual_roughness * perceptual_roughness;
    return studio_radiance(view_space_direction(world_dir), alpha * 1.6);
}
fn fil_diffuse_irradiance(world_normal: vec3f) -> vec3f {
    if frame.environment_present == 1u {
        return diffuse_shading(world_normal);
    }
    return fil_Irradiance_SphericalHarmonics(view_space_direction(world_normal), STUDIO_SH) * STUDIO_EXPOSURE;
}

/// Screen-space refraction (Filament REFRACTION_MODE_SCREEN_SPACE, REFRACTION_TYPE_SOLID): the
/// backdrop where the refracted ray leaves the solid; where it is bent far (the rim), the studio
/// along the exit direction (Filament's cubemap mode) — a flat backdrop alone gives glass no edge.
fn fil_refracted_sample(position: vec3f, direction: vec3f, view_dir: vec3f, lod: f32, perceptual_roughness: f32) -> vec3f {
    let uv = clamp(projected_surface_uv(position), vec2f(0.0), vec2f(1.0));
    let behind = textureSampleLevel(backdrop_texture, screen_sampler, uv, lod);
    let env = fil_prefiltered_radiance(direction, perceptual_roughness);
    let seen = behind.rgb + (1.0 - behind.a) * env;
    let bent = 1.0 - clamp(dot(direction, -view_dir), 0.0, 1.0);
    return mix(seen, env, smoothstep(0.08, 0.55, bent));
}

// ---- Jewel circuits: an internal path through a proxy of the solid's far side ----
/// Interfaces met inside after entering (0 = one exit through Filament's solid sphere).
const JEWEL_BOUNCES: i32 = 3;
/// 1: trace one path and split colours at each exit (hero wavelength: cheaper, finer fire);
/// 0: Filament's four wavelengths through the whole path; 2: four on faceted surfaces, hero elsewhere.
const JEWEL_HERO: i32 = 2;
/// Beer-Lambert: optical depth per thickness of a fully coloured body (albedo sets the colour).
const JEWEL_ABSORPTION: f32 = 1.0;

/// Exact unpolarised Fresnel reflectance for light inside (n = ior) meeting an interface to air at
/// cos_i (textbook form; 1 = total internal reflection).
fn jewel_fresnel_inside(cos_i: f32, ior: f32) -> f32 {
    let sin_t2 = ior * ior * (1.0 - cos_i * cos_i);
    if sin_t2 >= 1.0 { return 1.0; }
    let cos_t = sqrt(1.0 - sin_t2);
    let rs = (ior * cos_i - cos_t) / (ior * cos_i + cos_t);
    let rp = (cos_i - ior * cos_t) / (cos_i + ior * cos_t);
    return 0.5 * (rs * rs + rp * rp);
}

/// Small hard lights of the light tent (0 = the plain studio).
const JEWEL_TENT: f32 = 1.0;

/// The jeweller's light tent seen from inside a stone: the studio, darkened between lights, and two
/// rings of six small hard strip lights round the camera side (a stone's facets each catch a
/// different one: the sparkle). Only the nearest light of each ring can be seen along a direction.
fn jewel_tent(world_dir: vec3f, sigma: f32) -> vec3f {
    let d = view_space_direction(world_dir);
    var out = studio_radiance(d, sigma) * mix(1.0, 0.45, JEWEL_TENT);
    if JEWEL_TENT <= 0.0 || d.z < -0.2 { return out; }
    let step = 6.2831853 / 6.0;
    let az = atan2(d.y - 0.25, d.x);
    let el = acos(clamp(d.z, -1.0, 1.0));
    for (var ring = 0; ring < 2; ring += 1) {
        let offset = f32(ring) * 0.5 * step;
        let a = az - (round((az - offset) / step) * step + offset);
        let e = el - select(0.55, 1.05, ring == 1);
        let soft = 0.012 + sigma;
        let strip = smoothstep(0.05 + soft, 0.05 - soft, abs(a) * sin(max(el, 0.2))) * smoothstep(0.22 + soft, 0.22 - soft, abs(e));
        out += vec3f(5.0) * strip * JEWEL_TENT * STUDIO_EXPOSURE;
    }
    return out;
}

/// What leaves along `d` from `exit`: the backdrop where the ray carries on away from the eye,
/// the light tent (or the bound environment) where it turns back or aside.
fn jewel_seen(exit: vec3f, d: vec3f, v: vec3f, lod: f32, perceptual_roughness: f32) -> vec3f {
    if frame.environment_present == 1u {
        return fil_refracted_sample(exit, d, v, lod, perceptual_roughness);
    }
    let uv = clamp(projected_surface_uv(exit), vec2f(0.0), vec2f(1.0));
    let behind = textureSampleLevel(backdrop_texture, screen_sampler, uv, lod);
    let env = jewel_tent(d, perceptual_roughness * perceptual_roughness * 1.6);
    let seen = behind.rgb + (1.0 - behind.a) * env;
    let bent = 1.0 - clamp(dot(d, -v), 0.0, 1.0);
    return mix(seen, env, smoothstep(0.08, 0.55, bent));
}

/// Virtual back facets round the object's axis (0 = a smooth far side) and their tiers.
const JEWEL_FACETS: f32 = 16.0;
const JEWEL_TIERS: f32 = 3.0;
/// 1 while shading a surface whose normal is flat across the pixel (a facet); set by shade_surface.
var<private> jewel_faceted: f32 = 0.0;

fn quat_rotate(q: vec4f, v: vec3f) -> vec3f {
    let t = 2.0 * cross(q.xyz, v);
    return v + q.w * t + cross(q.xyz, t);
}
fn quat_unrotate(q: vec4f, v: vec3f) -> vec3f { return quat_rotate(vec4f(-q.xyz, q.w), v); }

/// Where the ray from `p` along `r` meets the far side, and its outward normal. A slab's far side is
/// parallel to its face; a solid's is its bounding ball round the object's centre, cut into virtual
/// facets when the stone is faceted (every facet sends the ray a different way: the kaleidoscope).
fn jewel_far_side(p: vec3f, r: vec3f, n: vec3f, v: vec3f, thickness: f32) -> vec4f {
    let slab = surface_solid_slab > 0.5 && surface_solid_slab < 1.5;
    let centre = surface_object[0].xyz;
    let radius = surface_object[0].w;
    if slab && radius > 0.0 {
        // The cap the ray is heading for: the slab's two faces are the planes ±half depth along its z.
        let axis = quat_rotate(surface_object[1], vec3f(0.0, 0.0, 1.0));
        let nb = select(-axis, axis, dot(r, axis) > 0.0);
        let h = max(surface_half_depth, 1e-3);
        let t = (h - dot(p - centre, nb)) / max(dot(r, nb), 0.05);
        return vec4f(nb, clamp(t, 0.0, 4.0 * h));
    }
    if radius <= 0.0 {
        let nb = normalize(n - 2.0 * dot(n, v) * v);
        return vec4f(nb, thickness / max(dot(r, nb), 0.2));
    }
    let o = p - centre;
    let b = dot(o, r);
    let c = dot(o, o) - radius * radius;
    let t = max(-b + sqrt(max(b * b - c, 0.0)), radius * 0.05);
    var nh = normalize(o + r * t);
    if JEWEL_FACETS > 0.0 && jewel_faceted > 0.5 {
        let q = surface_object[1];
        let local = quat_unrotate(q, nh);
        let az = atan2(local.z, local.x);
        let step = 6.2831853 / JEWEL_FACETS;
        let el = asin(clamp(local.y, -1.0, 1.0));
        let tier = 3.1415927 / JEWEL_TIERS;
        let a = (floor(az / step) + 0.5) * step;
        let e = clamp((floor(el / tier) + 0.5) * tier, -1.4, 1.4);
        nh = quat_rotate(q, vec3f(cos(a) * cos(e), sin(e), sin(a) * cos(e)));
    }
    return vec4f(nh, t);
}

/// The solid's own size along a ray (a slab's depth, a ball's diameter): absorption is per this.
fn jewel_size(thickness: f32) -> f32 {
    let slab = surface_solid_slab > 0.5 && surface_solid_slab < 1.5;
    let size = select(2.0 * surface_object[0].w, 2.0 * surface_half_depth, slab);
    return max(select(thickness, size, size > 0.0), 1e-3);
}

/// One wavelength's light through the solid, entered at `p` with normal `n`: each interface of the
/// far side splits the light by Fresnel into what leaves (seen along its exit) and what reflects on
/// inside; TIR keeps all of it.
fn jewel_path(p: vec3f, v: vec3f, n: vec3f, ior: f32, spread: f32, thickness: f32, tint: vec3f, lod: f32, pr: f32) -> vec3f {
    return jewel_path_n(p, v, n, ior, spread, thickness, tint, lod, pr, jewel_bounces);
}
fn jewel_path_n(p: vec3f, v: vec3f, n: vec3f, ior: f32, spread: f32, thickness: f32, tint: vec3f, lod: f32, pr: f32, bounces: i32) -> vec3f {
    var r = refract(-v, n, 1.0 / ior);
    if all(r == vec3f(0.0)) { return vec3f(0.0); }
    var pos = p;
    var w = vec3f(1.0);
    var out = vec3f(0.0);
    for (var k = 0; k < bounces; k += 1) {
        let far = jewel_far_side(pos, r, n, v, thickness);
        let nk = far.xyz;
        let travel = far.w;
        let cos_i = max(dot(r, nk), 1e-3);
        pos += r * travel;
        w *= exp(-JEWEL_ABSORPTION * (1.0 - tint) * travel / jewel_size(thickness));
        let f = jewel_fresnel_inside(cos_i, ior);
        if f < 1.0 {
            let d = refract(r, -nk, ior);
            var seen = jewel_seen(pos, d, v, lod, pr);
            if spread > 0.0 {
                // Hero wavelength: the path is traced once; red and blue split where they leave.
                let dr = refract(r, -nk, ior - spread);
                let db = refract(r, -nk, ior + spread);
                seen.r = select(jewel_seen(pos, dr, v, lod, pr).r, seen.r, all(dr == vec3f(0.0)));
                seen.b = select(jewel_seen(pos, db, v, lod, pr).b, 0.0, all(db == vec3f(0.0)));
            }
            out += w * (1.0 - f) * seen;
        }
        w *= f;
        r = reflect(r, -nk);
    }
    // What is still inside leaves along its last direction.
    return out + w * jewel_seen(pos, r, v, lod, pr);
}

// ---- What a stone can show at its size (LOD by projected radius), and its optical lies ----
/// Projected radius (px) above which a stone is a hero (the full path), and below which it is a
/// grain (a faked interior), a few pixels (a glint only).
const JEWEL_HERO_PX: f32 = 90.0;
const JEWEL_GRAIN_PX: f32 = 28.0;
const JEWEL_SPECK_PX: f32 = 5.0;
/// Above this a hero's four-wavelength fire costs more than it shows; it splits at the exits instead.
const JEWEL_XL_PX: f32 = 180.0;
/// Internal interfaces a stone of this size gets (set by shade_surface).
var<private> jewel_bounces: i32 = JEWEL_BOUNCES;

/// The solid's radius on screen, px (0 when the host gave no frame).
fn jewel_pixels() -> f32 {
    let radius = surface_object[0].w;
    if radius <= 0.0 { return 0.0; }
    let distance = max(length(surface_object[0].xyz - frame.camera_position), 1e-3);
    return radius * frame.focal_length_in_pixels.y / distance;
}

fn u79_hash(p: vec3f) -> f32 {
    let q = fract(p * vec3f(0.1031, 0.1030, 0.0973));
    let r = q + dot(q, q.yxz + 33.33);
    return fract((r.x + r.y) * r.z);
}
/// Which facet of the stone this is (its flat normal in the stone's own frame), and the copy's seed.
fn jewel_facet_hash(n: vec3f) -> f32 {
    let local = quat_unrotate(surface_object[1], n);
    return u79_hash(round(local * 7.0) + vec3f(surface_seed * 1.618, surface_seed * 0.577, 3.1));
}
/// Sparkle: a few facets of each copy (seeded) flash when they turn a key light of the tent toward the
/// eye — HDR, so the Look's star and bloom pick them up. What a stone too small to show its interior
/// still shows. The Look sets how strong (`look_sparkle`).
fn jewel_sparkle(v: vec3f, n: vec3f) -> vec3f {
    let h = jewel_facet_hash(n);
    let key = view_to_world_dir(normalize(vec3f(0.25 * sin(h * 40.0), 0.45, 1.0)));
    let aligned = pow(max(dot(reflect(-v, n), key), 1e-6), 24.0);
    return vec3f(8.0) * step(0.82, fract(h * 13.7)) * aligned * look_sparkle();
}
/// A stone a few pixels across: the tent seen off its facet, and its sparkle.
fn jewel_speck(v: vec3f, n: vec3f, tint: vec3f, pr: f32) -> vec3f {
    return jewel_tent(reflect(-v, n), pr * pr * 1.6 + 0.05) * mix(vec3f(1.0), tint, 0.7);
}
fn view_to_world_dir(d: vec3f) -> vec3f {
    // view_from_world is a rotation (+ translation): its transpose takes view directions back.
    let m = mat3x3f(frame.view_from_world[0], frame.view_from_world[1], frame.view_from_world[2]);
    return normalize(transpose(m) * d);
}

/// A transparent solid's silhouette parts its colours where it reflects (the rim is where a real edge
/// disperses most): the reflection read at three angles, one per channel. The Look sets how far
/// (`look_rim_fringe`).

fn fil_interleaved_gradient_noise(p: vec2f) -> f32 {
    return fract(52.982919 * fract(dot(p, vec2f(0.06711056, 0.00583715))));
}

/// Motolii's surface entry, Filament's evaluateLights() for an IBL-only frame.
/// `surface` = (roughness, metallic, transmission, ior); `dispersion` = 20 / Abbe (Filament's too).
fn shade_surface(albedo: vec3f, normal: vec3f, view_dir: vec3f, world_position: vec3f, thickness: f32, surface: vec4f, dispersion: f32) -> vec3f {
    jewel_faceted = select(0.0, 1.0, length(fwidth(normal)) < 1e-3);
    let metallic = clamp(surface.y, 0.0, 1.0);
    let transmission = clamp(surface.z, 0.0, 1.0);
    let ior = max(surface.w, 1.0);
    if sun_color().w > 0.0 {
        return albedo * transmission * (1.0 - metallic);
    }
    let shade = sun_shade(world_position, normal, 0.0);

    // getPixelParams()
    let n = normal;
    let v = view_dir;
    let n_dot_v = max(dot(n, v), 1e-4);                                   // clampNoV
    let diffuse_color = (1.0 - metallic) * albedo;                        // computeDiffuseColor
    let reflectance = fil_sq((ior - 1.0) / (ior + 1.0));                  // iorToF0(ior, 1)
    let f0 = albedo * metallic + reflectance * (1.0 - metallic);          // computeF0
    let perceptual_roughness = clamp(surface.x, FIL_MIN_PERCEPTUAL_ROUGHNESS, 1.0);
    let roughness = perceptual_roughness * perceptual_roughness;
    let dfg = fil_prefiltered_dfg(perceptual_roughness, n_dot_v);
    let energy_compensation = 1.0 + f0 * (1.0 / dfg.y - 1.0);

    // evaluateIBL()
    let e = mix(dfg.xxx, dfg.yyy, f0);                                    // specularDFG
    let r = fil_getSpecularDominantDirection(n, reflect(-v, n), roughness);
    var fr = e * fil_prefiltered_radiance(r, perceptual_roughness) * energy_compensation;
    var fd = diffuse_color * fil_diffuse_irradiance(n) * (1.0 - e);

    // evaluateClearCoatIBL(): a 4% coat, as much as the sheet's Clearcoat asks (any body, glass included).
    let clear_coat = surface_clear_coat;
    if clear_coat > 0.0 {
        let fc = fil_F_Schlick_1(0.04, 1.0, n_dot_v) * clear_coat;
        fd *= 1.0 - fc;
        fr *= 1.0 - fc;
        fr += fil_prefiltered_radiance(reflect(-v, n), FIL_CLEAR_COAT_ROUGHNESS) * fc;
    }

    var ft = vec3f(0.0);
    if transmission > 0.0 {
        // evaluateRefraction(): Filament remaps roughness so ior 1 has no microfacet refraction.
        let eta_ir = 1.0 / ior;
        let refraction_roughness = mix(surface.x, 0.0, clamp(eta_ir * 3.0 - 2.0, 0.0, 1.0));
        let levels = f32(textureNumLevels(backdrop_texture));
        let lod = pow(refraction_roughness, 0.8) * max(levels - 1.0, 0.0) * 0.55;  // Motolii's backdrop mapping
        let r_in = -v;
        let no_r_in = dot(n, r_in);
        let sin2 = 1.0 - no_r_in * no_r_in;
        // Faceted stones take the four wavelengths through the whole path (their fire); smooth glass and
        // slabs split colours only where they leave (hero), unless JEWEL_HERO forces one way.
        let slab = surface_solid_slab > 0.5 && surface_solid_slab < 1.5;
        // What the stone can show at its size decides how much of the path is traced.
        let px = jewel_pixels();
        let sized = px > 0.0 && JEWEL_BOUNCES > 0;
        let speck = sized && !slab && px < JEWEL_SPECK_PX;
        if sized && px < JEWEL_HERO_PX { jewel_bounces = min(JEWEL_BOUNCES, 2); }
        if sized && px < JEWEL_GRAIN_PX { jewel_bounces = 1; }
        // Four wavelengths through the whole path only for a hero that is not so large its fire would
        // cost the frame; everything else splits the colours where the light leaves.
        let full_fire = !sized || (px >= JEWEL_HERO_PX && px < JEWEL_XL_PX);
        let hero = JEWEL_HERO == 1 || (JEWEL_HERO == 2 && (jewel_faceted < 0.5 || slab || !full_fire));
        if speck {
            ft = jewel_speck(v, n, albedo, refraction_roughness);
        } else if dispersion > 0.0 && JEWEL_BOUNCES > 0 && hero {
            // Filament's spread (P(n-1) between its 486 and 656 nm samples), split at the exits.
            let spread = dispersion / 20.0 * (ior - 1.0) * 0.5;
            ft = jewel_path(world_position, v, n, ior, spread, thickness, albedo, lod, refraction_roughness);
        } else if dispersion > 0.0 {
            // calculateDispersion(): four wavelengths, jittered, integrated by Filament's K matrices.
            let disp = dispersion / 20.0 * (ior - 1.0);
            let jitter = (fil_interleaved_gradient_noise(projected_surface_uv(world_position) * frame.framebuffer_resolution) - 0.5) * 0.35 * disp;
            let offsets = vec4f(0.70795215, 0.24790980, 0.0, -0.29204785);
            var s: array<vec3f, 4>;
            for (var i = 0; i < 4; i += 1) {
                let nd = ior + disp * offsets[i] + jitter;
                if JEWEL_BOUNCES > 0 {
                    s[i] = jewel_path(world_position, v, n, nd, 0.0, thickness, albedo, lod, refraction_roughness);
                } else {
                    var ray: fil_Refraction;
                    fil_refractedRaySolidSphere(world_position, r_in, no_r_in, sin2, 1.0 / nd, nd, thickness, n, &ray);
                    s[i] = fil_refracted_sample(ray.position, ray.direction, v, lod, refraction_roughness);
                }
            }
            ft = fil_dispersionIntegrate(s[0], s[1], s[2], s[3]);
        } else {
            if JEWEL_BOUNCES > 0 {
                ft = jewel_path(world_position, v, n, ior, 0.0, thickness, albedo, lod, refraction_roughness);
            } else {
                var ray: fil_Refraction;
                fil_refractedRaySolidSphere(world_position, r_in, no_r_in, sin2, eta_ir, ior, thickness, n, &ray);
                ft = fil_refracted_sample(ray.position, ray.direction, v, lod, refraction_roughness);
            }
        }
        if sized && jewel_faceted > 0.5 && !slab { ft += jewel_sparkle(v, n); }
        ft *= (1.0 - e);          // fresnel from the first interface
        let rim_fringe = look_rim_fringe();
        if rim_fringe > 0.0 {
            // A transparent silhouette disperses what it reflects: red and blue leave the rim a little
            // apart (grazing light parts most), so a bright band on the rim gets a red and a blue edge.
            let tangent = normalize(r - n * dot(r, n) + vec3f(1e-5));
            let part = pow(max(1.0 - n_dot_v, 0.0), 2.0) * 0.12 * rim_fringe * max(dispersion, 0.4);
            // Only the rim: where the split would be under a hundredth of a radian nothing is seen.
            if part > 0.01 {
            let e_r = fil_prefiltered_radiance(normalize(r + tangent * part), perceptual_roughness);
            let e_b = fil_prefiltered_radiance(normalize(r - tangent * part), perceptual_roughness);
            let e_g = fil_prefiltered_radiance(r, perceptual_roughness);
            fr = e * vec3f(e_r.r, e_g.g, e_b.b) * energy_compensation;
            }
        }
        ft *= diffuse_color;      // base color changes the amount of light passing through
        ft *= transmission;
        fd *= (1.0 - transmission);
    }

    // The surface's own light gets Filament's PBR Neutral shoulder; what is seen through it is the
    // composite's (already display-referred) and passes untouched.
    // The shoulder keeps the surface's look; what it folded away above 1 stays in the HDR view
    // as excess, so real highlights (a softbox in chrome, the tent in a stone) reach the Look's glare.
    let lit = (fr + fd) * shade;
    return hdr_safe(fil_PBRNeutralToneMapper(lit) + max(lit - vec3f(HDR_KNEE), vec3f(0.0)) + ft + albedo * surface_emission);
}

/// Where a lit surface's own light starts to count as HDR excess (above the shoulder's white).
const HDR_KNEE: f32 = 1.0;

/// The view is scene-linear half float. A radiance no pixel can hold (NaN, negative from the
/// wavelength integration, beyond half's range) must not reach the resolve, where one sample would
/// outweigh the other three; the 8-bit view used to clamp it at the write.
fn hdr_safe(c: vec3f) -> vec3f {
    return hdr_finite(c);
}
