// rayq band: every pixel's line of sight through the glass band, by the rule of model D (filament/band_bvh.mat):
// bend in (smooth normal of the surface entered), ask the mesh for the next surface, leave if Snell's law lets the
// line out (flat normal of the triangle met), mirror and ask again if not. The asking is the GPU's own query.
// One colour, no branching, no noise, nothing kept between frames.
enable wgpu_ray_query;

struct Params {
    eye: vec4<f32>,
    fwd: vec4<f32>,
    right: vec4<f32>,   // times tan(half the angle of view) and the picture's width over height
    up: vec4<f32>,      // times tan(half the angle of view)
    size: vec2<u32>,
    ior: f32,
    most: f32,          // mirrorings inside a line may have; below 0: only the way in is asked
    look: vec4<f32>,    // paint only: the studio's brightness; the studio turned about the up axis (radians); 1: smooth normals inside too
}

@group(0) @binding(0) var acc: acceleration_structure;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read> pos: array<f32>;
@group(0) @binding(3) var<storage, read> idx: array<u32>;
@group(0) @binding(4) var<storage, read> nrm: array<f32>;
@group(0) @binding(5) var<storage, read_write> numbers: array<vec4<f32>>;          // check: two a pixel
@group(0) @binding(6) var picture: texture_storage_2d<rgba16float, write>;         // draw
@group(0) @binding(7) var studio: texture_cube<f32>;                               // paint
@group(0) @binding(8) var studio_how: sampler;
@group(0) @binding(9) var painted: texture_storage_2d<rgba8unorm, write>;

struct Line {
    count: f32,      // mirrorings before leaving; 99: still inside after `most`; -1: no surface ahead
    way: f32,        // the whole way inside
    firstWay: f32,   // to the first surface inside
    firstTri: f32,
    glass: f32,      // 1: the line of sight meets the glass
    inTri: f32,      // the triangle entered
    out: vec3<f32>,  // the direction it leaves in
}

fn corner(i: u32) -> vec3<f32> { return vec3<f32>(pos[3u * i], pos[3u * i + 1u], pos[3u * i + 2u]); }
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

fn follow(px: vec2<u32>) -> Line { return follow_at(vec2<f32>(px) + 0.5); }

fn follow_at(at_px: vec2<f32>) -> Line {
    var line = Line(99.0, 0.0, -1.0, 0.0, 0.0, 0.0, vec3<f32>(0.0));
    let ndc = at_px / vec2<f32>(params.size) * 2.0 - 1.0;
    let d = normalize(params.fwd.xyz + params.right.xyz * ndc.x + params.up.xyz * ndc.y);
    let h1 = meet(params.eye.xyz, d, 1.0e-4);
    if (h1.kind == RAY_QUERY_INTERSECTION_NONE) { return line; }
    line.glass = 1.0;
    line.inTri = f32(h1.primitive_index);
    let i = 3u * h1.primitive_index;
    let b = h1.barycentrics;
    var n = normalize(smooth_at(idx[i]) * (1.0 - b.x - b.y) + smooth_at(idx[i + 1u]) * b.x + smooth_at(idx[i + 2u]) * b.y);
    if (dot(n, d) > 0.0) { n = -n; }
    var dir = refract(d, n, 1.0 / params.ior);
    var start = params.eye.xyz + d * h1.t + dir * 0.001;
    for (var k = 0; k <= 32; k++) {
        if (f32(k) > params.most) { break; }
        let h = meet(start, dir, 0.0);
        if (h.kind == RAY_QUERY_INTERSECTION_NONE) { line.count = -1.0; break; }
        line.way += h.t + 0.001;
        if (k == 0) { line.firstWay = h.t + 0.001; line.firstTri = f32(h.primitive_index); }
        let j = 3u * h.primitive_index;
        let a = corner(idx[j]);
        var flat = normalize(cross(corner(idx[j + 1u]) - a, corner(idx[j + 2u]) - a));
        if (params.look.z > 0.5) {   // (paint, when asked: where the line meets is still the triangle's; the normal there is the smooth one)
            let c = h.barycentrics;
            flat = normalize(smooth_at(idx[j]) * (1.0 - c.x - c.y) + smooth_at(idx[j + 1u]) * c.x + smooth_at(idx[j + 2u]) * c.y);
        }
        if (dot(flat, dir) > 0.0) { flat = -flat; }   // (facing the line)
        let leaving = refract(dir, flat, params.ior);   // (zero: past the critical angle)
        if (dot(leaving, leaving) > 0.0) { dir = leaving; line.count = f32(k); break; }
        let at = start + dir * h.t;
        dir = reflect(dir, flat);
        start = at + dir * 0.001;
    }
    line.out = dir;
    return line;
}

@compute @workgroup_size(8, 8)
fn check(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.size.x || id.y >= params.size.y) { return; }
    let line = follow(id.xy);
    let at = 2u * (id.y * params.size.x + id.x);
    numbers[at] = vec4<f32>(line.count, line.way, line.firstWay, line.firstTri);
    numbers[at + 1u] = vec4<f32>(line.glass, line.inTri, 0.0, 0.0);
}

@compute @workgroup_size(8, 8)
fn draw(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.size.x || id.y >= params.size.y) { return; }
    let line = follow(id.xy);
    // (what a renderer would keep for the next step: where the line leaves to, and whether it left)
    textureStore(picture, vec2<i32>(id.xy), vec4<f32>(line.out, line.count));
}

// paint: the same lines as a picture, the way model D's picture is made: where a line leaves, the studio that way;
// magenta: still inside after `most`; red: no surface ahead; the ground is one flat grey (no scene here).
// Four lines a pixel (a still picture's edges), and a plain film curve: this is not Filament's own tone mapping.
fn seen(at_px: vec2<f32>) -> vec3<f32> {
    let line = follow_at(at_px);
    if (line.glass < 0.5) { return vec3<f32>(0.60); }
    if (line.count < -0.5) { return vec3<f32>(4.0, 0.0, 0.0); }
    if (line.count > 98.0) { return vec3<f32>(4.0, 0.0, 4.0); }
    let c = cos(params.look.y);
    let s = sin(params.look.y);
    let d = vec3<f32>(line.out.x * c + line.out.z * s, line.out.y, -line.out.x * s + line.out.z * c);
    return textureSampleLevel(studio, studio_how, d, 0.0).rgb * params.look.x;
}

@compute @workgroup_size(8, 8)
fn paint(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.size.x || id.y >= params.size.y) { return; }
    let p = vec2<f32>(id.xy);
    var c = seen(p + vec2<f32>(0.25, 0.25)) + seen(p + vec2<f32>(0.75, 0.25)) + seen(p + vec2<f32>(0.25, 0.75)) + seen(p + vec2<f32>(0.75, 0.75));
    c = c * 0.25;
    c = clamp((c * (2.51 * c + 0.03)) / (c * (2.43 * c + 0.59) + 0.14), vec3<f32>(0.0), vec3<f32>(1.0));   // (Narkowicz's fit of the ACES film curve)
    textureStore(painted, vec2<i32>(i32(id.x), i32(params.size.y - 1u - id.y)), vec4<f32>(pow(c, vec3<f32>(1.0 / 2.2)), 1.0));
}
