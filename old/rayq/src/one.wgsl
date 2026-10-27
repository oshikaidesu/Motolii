// rayq one: one triangle, two lines of sight asked of the GPU's own ray-against-mesh query. Nothing else.
enable wgpu_ray_query;

@group(0) @binding(0) var acc: acceleration_structure;
@group(0) @binding(1) var<storage, read_write> answer: array<vec4<f32>>;

fn ask(start: vec3<f32>, along: vec3<f32>) -> vec4<f32> {
    var rq: ray_query;
    rayQueryInitialize(&rq, acc, RayDesc(RAY_FLAG_FORCE_OPAQUE, 0xFFu, 0.0, 1000.0, start, along));
    // Not `while (rayQueryProceed(&rq)) {}`: on Metal, wgpu 29 never takes back the "more to do" it answers (naga,
    // back/msl/writer.rs, RAY_QUERY_MODERN_SUPPORT = false), so that loop has no end there; it only seemed to work
    // while Metal's compiler threw the empty loop away. All triangles here are solid, so one step is the whole search.
    for (var step = 0; step < 2; step++) { if (!rayQueryProceed(&rq)) { break; } }
    let hit = rayQueryGetCommittedIntersection(&rq);
    return vec4<f32>(f32(hit.kind), hit.t, hit.barycentrics);
}

@compute @workgroup_size(1)
fn main() {
    answer[0] = ask(vec3<f32>(0.25, 0.25, -1.0), vec3<f32>(0.0, 0.0, 1.0));   // meets the triangle
    answer[1] = ask(vec3<f32>(2.0, 2.0, -1.0), vec3<f32>(0.0, 0.0, 1.0));     // passes it by
}
