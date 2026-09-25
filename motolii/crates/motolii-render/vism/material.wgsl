// Motolii's standard material, the frame side: the host's constants (the sun's light cookie, the
// Views a Vism asked for, the Look's surface strengths) and the solid's frame. The lit model itself
// is Filament's (material_filament.wgsl, after this module); the two are one program's material.
// Appended to a surface program's surface hook; reads the frame's resources (re_renderer's bindings).

/// What a solid tells its surface (the host's slots 12..22 when the Vism's own inputs leave them
/// free; `surface_extras`): 1 = an extruded slab, 2 = any other solid, 0 = not told.
var<private> surface_solid_slab: f32 = 0.0;
/// The solid's frame, (centre.xyz, radius) and its rotation quaternion (x, y, z, w).
var<private> surface_object: array<vec4f, 2>;
/// A sheet's clear coat (0..1) and emission (the surface's own colour × this, scene-linear).
var<private> surface_clear_coat: f32 = 0.0;
var<private> surface_emission: f32 = 0.0;
/// Half the solid's depth along its own z (a slab's thickness / 2).
var<private> surface_half_depth: f32 = 0.0;
/// A stable number per copy (its place in the frame's layer list): a Repeater's copies differ in
/// facets and sparkle without an instance of work of their own.
var<private> surface_seed: f32 = 0.0;

/// Hands the host's solid frame (`in.params[3..6]`) and a sheet's extra knobs to the material; adds 0.
fn surface_extras(frame0: vec4f, frame1: vec4f, kind: vec4f, clear_coat: f32, emission: f32) -> f32 {
    surface_object = array<vec4f, 2>(frame0, frame1);
    surface_solid_slab = kind.x;
    surface_half_depth = kind.y;
    surface_seed = kind.z;
    surface_clear_coat = clear_coat;
    surface_emission = emission;
    return 0.0;
}

/// A radiance the half-float view can hold. NaN is tested on its bits (Metal's fast math folds
/// `x != x` away); negatives and values past half's range are clamped, since in the resolve one such
/// sample would outweigh the other three.
fn hdr_finite(c: vec3f) -> vec3f {
    let bits = bitcast<vec3u>(c) & vec3u(0x7fffffffu);
    let finite = select(c, vec3f(0.0), bits >= vec3u(0x7f800000u));
    return clamp(finite, vec3f(0.0), vec3f(64.0));
}

/// The composition's Look as the surfaces see it (compositor/look.rs `LookParams::surface`):
/// x = how far a transparent rim parts its colours, y = the strength of a cut stone's sparkle.
fn look_rim_fringe() -> f32 { return frame.program_constants[8].x; }
fn look_sparkle() -> f32 { return frame.program_constants[8].y; }

/// xyz: world direction toward the sun; w: its share of the diffuse light (0 = no sun, no shadow).
fn sun_direction() -> vec4f { return frame.program_constants[0]; }
/// rgb: the sun's tint; w = 1 while the light cookie is captured (surfaces write what they let through).
fn sun_color() -> vec4f { return frame.program_constants[1]; }
/// World → light cookie uv (orthographic, looking along the sun).
fn light_uv_from_world() -> mat4x4f {
    return mat4x4f(frame.program_constants[2], frame.program_constants[3], frame.program_constants[4], frame.program_constants[5]);
}

/// The Views this surface's Vism asked for (`VIEWS`, in their order), side by side in one picture;
/// 0 when it asked for none or the host drew none.
fn view_count() -> u32 { return u32(frame.program_constants[6].x); }
/// The coarsest mip level the host generated for the Views (`VIEW_BLUR`); reads are clamped to it.
fn view_lod_limit() -> f32 { return frame.program_constants[6].y; }
/// The lod a roughness (0..1) reads the Views at: the backdrop's mapping (`transmitted_backdrop`,
/// mirrored by the host's `backdrop_levels_read`), so a Vism declaring `VIEW_BLUR` reads no
/// level the host did not generate.
fn view_lod(roughness: f32) -> f32 {
    let levels = f32(textureNumLevels(view_capture_texture));
    return pow(clamp(roughness, 0.0, 1.0), 0.8) * max(levels - 1.0, 0.0) * 0.55;
}
/// Where they were taken from: the centre of the requesting layer (world).
fn view_origin() -> vec3f { return frame.program_constants[7].xyz; }
/// View `i` at `uv` (0..1 across that View, +y down), premultiplied: alpha is what the View saw
/// there. `lod` blurs it by mip level, no coarser than the host generated.
fn view_sample(i: u32, uv: vec2f, lod_asked: f32) -> vec4f {
    let lod = min(lod_asked, view_lod_limit());
    let n = f32(max(view_count(), 1u));
    let size = vec2f(textureDimensions(view_capture_texture));
    let margin = min(0.49, exp2(lod) * n / size.x);
    let local = clamp(uv, vec2f(margin), vec2f(1.0 - margin));
    return textureSampleLevel(view_capture_texture, screen_sampler, vec2f((f32(i) + local.x) / n, local.y), lod);
}

/// Light from the sun that reaches `position`: 1 where nothing blocks it, the blocker's tint × coverage
/// where the cookie says something does. No depth: a blocker shades everything along its ray.
fn sun_light_through(position: vec3f) -> vec3f {
    if sun_direction().w <= 0.0 {
        return vec3f(1.0);
    }
    let uv = (light_uv_from_world() * vec4f(position, 1.0)).xy;
    if any(uv < vec2f(0.0)) || any(uv > vec2f(1.0)) {
        return vec3f(1.0);
    }
    let cookie = textureSampleLevel(coverage_texture, screen_sampler, uv, 0.0);
    return vec3f(1.0 - cookie.a) + cookie.rgb;
}

/// Shadow factor: only the sun's share of the light is taken away, scaled by how much the surface
/// faces the sun. `facing_floor` keeps a flat, unlit picture readable as a receiver.
fn sun_shade(position: vec3f, normal: vec3f, facing_floor: f32) -> vec3f {
    let facing = max(clamp(dot(normal, sun_direction().xyz), 0.0, 1.0), facing_floor);
    let share = sun_direction().w * facing;
    return vec3f(1.0) - share * (vec3f(1.0) - sun_light_through(position));
}
