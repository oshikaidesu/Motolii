struct View {
    viewport: vec2f,
}

@group(0) @binding(0) var<uniform> view: View;

struct RectInstance {
    center: vec2f,
    half_size: vec2f,
    color: vec4f,
}

@group(1) @binding(0) var<storage, read> instances: array<RectInstance>;

fn px_to_ndc(p: vec2f) -> vec2f {
    let ndc = (p / view.viewport) * 2.0 - 1.0;
    return vec2f(ndc.x, -ndc.y);
}

var<private> quad: array<vec2f, 6> = array<vec2f, 6>(
    vec2f(-1.0, -1.0),
    vec2f( 1.0, -1.0),
    vec2f(-1.0,  1.0),
    vec2f( 1.0, -1.0),
    vec2f( 1.0,  1.0),
    vec2f(-1.0,  1.0),
);

struct VsOut {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VsOut {
    let inst = instances[ii];
    let corner = quad[vi] * inst.half_size;
    let px = inst.center + corner;
    var out: VsOut;
    out.position = vec4f(px_to_ndc(px), 0.0, 1.0);
    out.color = inst.color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
    return in.color;
}
