// rayq shapes: simple glass solids, every pixel's line of sight asked of the GPU's own ray-against-mesh query.
// Three models of what glass does to a line, switched by params.model, same shapes, same studio:
//   0  "before" (the rule of model D): one index of refraction; inside, the line leaves at the first surface that
//      lets it out (mirrors and goes on only past the critical angle); no Fresnel weights.
//   1  every surface splits the line by Fresnel's law into a mirrored and a bent part, and BOTH are followed
//      (outside and inside, into other glass too), each with its share, until its share is small. One wavelength.
//   2  the same, done separately for each of params.steps wavelengths, each with its own index of refraction, so
//      each has its own way through the glass, its own mirrorings and its own direction of leaving.
// What a line that ends in the air sees: the studio (an HDR cube). Nothing is shifted afterwards; no noise;
// nothing kept between frames.
enable wgpu_ray_query;

struct Params {
    eye: vec4<f32>,
    fwd: vec4<f32>,
    right: vec4<f32>,   // times tan(half the angle of view) and the picture's width over height
    up: vec4<f32>,      // times tan(half the angle of view)
    size: vec2<u32>,
    steps: u32,         // wavelengths (1 for models 0 and 1)
    model: u32,
    look: vec4<f32>,    // the studio's brightness; the studio turned about the up axis (radians); 1: the studio is white everywhere (test); lines a pixel (1 or 4)
    opts: vec4<f32>,    // 1: no film curve (test); -; a share below this is not followed further; most surfaces met by one wavelength of one line
    wave: array<vec4<f32>, 32>,   // per wavelength: its share of red, green, blue (they add up to 1 each); its index of refraction
}

@group(0) @binding(0) var acc: acceleration_structure;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(3) var<storage, read> idx: array<u32>;
@group(0) @binding(4) var<storage, read> nrm: array<f32>;
@group(0) @binding(5) var studio: texture_cube<f32>;
@group(0) @binding(6) var studio_how: sampler;
@group(0) @binding(7) var painted: texture_storage_2d<rgba8unorm, write>;

fn smooth_at(i: u32) -> vec3<f32> { return vec3<f32>(nrm[3u * i], nrm[3u * i + 1u], nrm[3u * i + 2u]); }

fn meet(start: vec3<f32>, along: vec3<f32>, least: f32) -> RayIntersection {
    var rq: ray_query;
    rayQueryInitialize(&rq, acc, RayDesc(RAY_FLAG_FORCE_OPAQUE, 0xFFu, least, 1.0e30, start, along));
    // Not `while (rayQueryProceed(&rq)) {}`: on Metal, wgpu 29 never takes back the "more to do" it answers (naga,
    // back/msl/writer.rs, RAY_QUERY_MODERN_SUPPORT = false), so that loop has no end there; it only seemed to work
    // while Metal's compiler threw the empty loop away. All triangles here are solid, so one step is the whole search.
    for (var step = 0; step < 2; step++) { if (!rayQueryProceed(&rq)) { break; } }
    return rayQueryGetCommittedIntersection(&rq);
}
// the surface's smooth normal where the line met it, turned to face the line
fn facing(h: RayIntersection, along: vec3<f32>) -> vec3<f32> {
    let j = 3u * h.primitive_index;
    let c = h.barycentrics;
    let n = normalize(smooth_at(idx[j]) * (1.0 - c.x - c.y) + smooth_at(idx[j + 1u]) * c.x + smooth_at(idx[j + 2u]) * c.y);
    return select(n, -n, dot(n, along) > 0.0);
}
fn env(d: vec3<f32>) -> vec3<f32> {
    if (params.look.z > 0.5) { return vec3<f32>(1.0); }
    let c = cos(params.look.y);
    let s = sin(params.look.y);
    return textureSampleLevel(studio, studio_how, vec3<f32>(d.x * c + d.z * s, d.y, -d.x * s + d.z * c), 0.0).rgb * params.look.x;
}
// the share of unpolarised light a surface mirrors (Fresnel), going from index n1 into n2 at cosine ci
fn mirrored(ci: f32, n1: f32, n2: f32) -> f32 {
    let s2 = (n1 / n2) * (n1 / n2) * (1.0 - ci * ci);
    if (s2 >= 1.0) { return 1.0; }
    let ct = sqrt(1.0 - s2);
    let rs = (n1 * ci - n2 * ct) / (n1 * ci + n2 * ct);
    let rp = (n1 * ct - n2 * ci) / (n1 * ct + n2 * ci);
    return 0.5 * (rs * rs + rp * rp);
}

// model 0
fn before(eye: vec3<f32>, d: vec3<f32>, n: f32) -> vec3<f32> {
    let h1 = meet(eye, d, 1.0e-4);
    if (h1.kind == RAY_QUERY_INTERSECTION_NONE) { return env(d); }
    var dir = refract(d, facing(h1, d), 1.0 / n);
    var start = eye + d * h1.t + dir * 0.001;
    for (var k = 0; k <= 8; k++) {
        let h = meet(start, dir, 0.0);
        if (h.kind == RAY_QUERY_INTERSECTION_NONE) { return vec3<f32>(4.0, 0.0, 0.0); }   // no surface ahead
        let m = facing(h, dir);
        let leaving = refract(dir, m, n);
        if (dot(leaving, leaving) > 0.0) { return env(leaving); }
        let at = start + dir * h.t;
        dir = reflect(dir, m);
        start = at + dir * 0.001;
    }
    return vec3<f32>(4.0, 0.0, 4.0);   // still inside
}

// models 1 and 2: one wavelength of one line of sight, every part of it followed
fn every_part(eye: vec3<f32>, d0: vec3<f32>, n: f32) -> vec3<f32> {
    var from_: array<vec3<f32>, 8>;
    var along: array<vec3<f32>, 8>;
    var share: array<f32, 8>;
    var within: array<u32, 8>;   // 1: this part is inside glass
    var held = 1;
    from_[0] = eye;
    along[0] = d0;
    share[0] = 1.0;
    within[0] = 0u;
    var light = vec3<f32>(0.0);
    var met = 0u;
    let small = params.opts.z;
    let most = u32(params.opts.w);
    for (var guard = 0; guard < 96; guard++) {
        if (held == 0) { break; }
        held--;
        let o = from_[held];
        let d = along[held];
        let w = share[held];
        let inside = within[held];
        if (met >= most) { light += w * env(d); continue; }   // (out of surfaces: what is left is taken from the studio the way it points)
        let h = meet(o, d, 0.0);
        if (h.kind == RAY_QUERY_INTERSECTION_NONE) { light += w * env(d); continue; }
        met++;
        let p = o + d * h.t;
        let m = facing(h, d);
        let ci = min(1.0, -dot(m, d));
        let n1 = select(1.0, n, inside == 1u);
        let n2 = select(n, 1.0, inside == 1u);
        let f = mirrored(ci, n1, n2);
        let back = reflect(d, m);
        let through = refract(d, m, n1 / n2);
        // the two parts, the weaker first (so the stronger is followed first). A part too weak to follow, or one
        // there is no room to remember, is taken from the studio the way it points: its light is not thrown away.
        for (var part = 0; part < 2; part++) {
            let is_back = (part == 0) == (f < 0.5);
            let ww = w * select(1.0 - f, f, is_back);
            if (ww <= 0.0) { continue; }
            let dd = select(through, back, is_back);
            if (ww < small || held >= 8) { light += ww * env(dd); continue; }
            from_[held] = p + dd * 0.001;
            along[held] = dd;
            share[held] = ww;
            within[held] = select(1u - inside, inside, is_back);
            held++;
        }
    }
    for (var k = 0; k < held; k++) { light += share[k] * env(along[k]); }
    return light;
}

fn seen(at_px: vec2<f32>) -> vec4<f32> {
    let ndc = at_px / vec2<f32>(params.size) * 2.0 - 1.0;
    let d = normalize(params.fwd.xyz + params.right.xyz * ndc.x + params.up.xyz * ndc.y);
    let eye = params.eye.xyz;
    if (params.model == 6u) { return vec4<f32>(d * 0.5 + 0.5, 0.0); }                            // (tests: the direction itself)
    if (params.model == 7u) { return vec4<f32>(env(d), 0.0); }                                  // (tests: no query at all)
    if (meet(eye, d, 1.0e-4).kind == RAY_QUERY_INTERSECTION_NONE) { return vec4<f32>(env(d), 0.0); }
    if (params.model == 8u) { return vec4<f32>(1.0, 0.5, 0.0, 1.0); }                           // (tests: the way in only)
    if (params.model == 0u) { return vec4<f32>(before(eye, d, params.wave[0].w), 1.0); }
    var c = vec3<f32>(0.0);
    for (var i = 0u; i < params.steps; i++) { c += params.wave[i].rgb * every_part(eye, d, params.wave[i].w); }
    return vec4<f32>(max(c, vec3<f32>(0.0)), 1.0);
}

@compute @workgroup_size(8, 8)
fn paint(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.size.x || id.y >= params.size.y) { return; }
    let p = vec2<f32>(id.xy);
    var c = vec4<f32>(0.0);
    if (params.look.w > 1.5) {
        c = (seen(p + vec2<f32>(0.25, 0.25)) + seen(p + vec2<f32>(0.75, 0.25)) + seen(p + vec2<f32>(0.25, 0.75)) + seen(p + vec2<f32>(0.75, 0.75))) * 0.25;
    } else {
        c = seen(p + 0.5);
    }
    var rgb = c.rgb;
    if (params.opts.x < 0.5) {
        rgb = (rgb * (2.51 * rgb + 0.03)) / (rgb * (2.43 * rgb + 0.59) + 0.14);   // (Narkowicz's fit of the ACES film curve; not Filament's tone mapping)
        rgb = pow(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(1.0 / 2.2));
    }
    textureStore(painted, vec2<i32>(i32(id.x), i32(params.size.y - 1u - id.y)), vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), c.a));
}
