/*{
  "ID": "motolii.cube_mirror",
  "LABEL": "Cube Mirror",
  "STAGE": "surface",
  "DESCRIPTION": "A mirror of the world around it: six Views from its centre (a cube), over the environment's light where they saw nothing",
  "VIEWS": [
    { "FROM": "layer", "LOOK": [1, 0, 0], "UP": [0, -1, 0] },
    { "FROM": "layer", "LOOK": [-1, 0, 0], "UP": [0, -1, 0] },
    { "FROM": "layer", "LOOK": [0, 1, 0], "UP": [0, 0, 1] },
    { "FROM": "layer", "LOOK": [0, -1, 0], "UP": [0, 0, -1] },
    { "FROM": "layer", "LOOK": [0, 0, 1], "UP": [0, -1, 0] },
    { "FROM": "layer", "LOOK": [0, 0, -1], "UP": [0, -1, 0] }
  ],
  "VIEW_SIZE": 256,
  "INPUTS": [
    { "NAME": "roughness", "LABEL": "Roughness", "TYPE": "float", "DEFAULT": 0.05, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "tint", "LABEL": "Tint", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0 }
  ]
}*/

// The six Views, in the order VIEWS asks for them: a cube around the layer.
fn cube_face(r: vec3f) -> u32 {
    let a = abs(r);
    if a.x >= a.y && a.x >= a.z { return select(1u, 0u, r.x >= 0.0); }
    if a.y >= a.z { return select(3u, 2u, r.y >= 0.0); }
    return select(5u, 4u, r.z >= 0.0);
}

fn surface(in: SurfaceIn, p: SurfaceParams) -> vec3f {
    let r = reflect(-in.view_dir, in.normal);
    let around = environment_specular_along(r, p.roughness);
    var seen = vec4f(0.0);
    if view_count() == 6u {
        let looks = array<vec3f, 6>(vec3f(1, 0, 0), vec3f(-1, 0, 0), vec3f(0, 1, 0), vec3f(0, -1, 0), vec3f(0, 0, 1), vec3f(0, 0, -1));
        let ups = array<vec3f, 6>(vec3f(0, -1, 0), vec3f(0, -1, 0), vec3f(0, 0, 1), vec3f(0, 0, -1), vec3f(0, -1, 0), vec3f(0, -1, 0));
        let face = cube_face(r);
        let forward = looks[face];
        let up = ups[face];
        let uv = vec2f(dot(r, cross(forward, up)), -dot(r, up)) / max(dot(r, forward), 1e-6) * 0.5 + 0.5;
        seen = view_sample(face, uv, p.roughness * 6.0);
    }
    let reflected = seen.rgb + (1.0 - seen.a) * around;
    return reflected * mix(vec3f(1.0), in.albedo, p.tint);
}
