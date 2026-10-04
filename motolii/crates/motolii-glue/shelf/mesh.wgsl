// Solids in the one camera. Vertex: position, color. The mesh carries no normals; the face
// normal comes from the screen derivatives of the world position, so the light is flat per face
// and agrees with the glass: the same key light up-left, a dim fill from the right, a sky lift.
struct Uniforms { vp: mat4x4<f32>, model: mat4x4<f32> };
@group(0) @binding(0) var<uniform> u: Uniforms;
struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world: vec3<f32>,
};
@vertex
fn vs(@location(0) pos: vec3<f32>, @location(1) color: vec4<f32>) -> VsOut {
    var out: VsOut;
    let world = u.model * vec4<f32>(pos, 1.0);
    out.clip = u.vp * world;
    out.color = color;
    out.world = world.xyz;
    return out;
}

const KEY: vec3<f32> = vec3<f32>(-0.55, 0.75, 0.6);
const FILL: vec3<f32> = vec3<f32>(0.8, 0.1, 0.4);

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    // dpdx × dpdy points away from the eye (framebuffer y runs down); the outward face normal is the opposite.
    let n = normalize(cross(dpdy(in.world), dpdx(in.world)));
    let key = max(dot(n, normalize(KEY)), 0.0);
    let fill = max(dot(n, normalize(FILL)), 0.0);
    let sky = 0.5 + 0.5 * n.y;
    let light = vec3<f32>(0.16, 0.17, 0.2) * sky + vec3<f32>(1.0, 0.96, 0.9) * key * 0.95 + vec3<f32>(0.55, 0.6, 0.7) * fill * 0.22;
    let albedo = in.color.rgb;
    let half = normalize(normalize(KEY) + vec3<f32>(0.0, 0.0, 1.0));
    let spec = pow(max(dot(n, half), 0.0), 36.0) * 0.12;
    return vec4<f32>(albedo * light + vec3<f32>(spec), in.color.a);
}
