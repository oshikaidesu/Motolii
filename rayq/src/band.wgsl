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
}

@group(0) @binding(0) var acc: acceleration_structure;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read> pos: array<f32>;
@group(0) @binding(3) var<storage, read> idx: array<u32>;
@group(0) @binding(4) var<storage, read> nrm: array<f32>;
@group(0) @binding(5) var<storage, read_write> numbers: array<vec4<f32>>;          // check: two a pixel
@group(0) @binding(6) var picture: texture_storage_2d<rgba16float, write>;         // draw

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
    rayQueryInitialize(&rq, acc, RayDesc(0u, 0xFFu, least, 1.0e30, start, along));
    while (rayQueryProceed(&rq)) {}
    return rayQueryGetCommittedIntersection(&rq);
}

fn follow(px: vec2<u32>) -> Line {
    var line = Line(99.0, 0.0, -1.0, 0.0, 0.0, 0.0, vec3<f32>(0.0));
    let ndc = (vec2<f32>(px) + 0.5) / vec2<f32>(params.size) * 2.0 - 1.0;
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
