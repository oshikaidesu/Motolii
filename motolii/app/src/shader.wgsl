struct Immediates {
    angle_aspect: vec4<f32>,
};

var<immediate> pc: Immediates;

struct VOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.7),
        vec2<f32>(-0.65, -0.5),
        vec2<f32>(0.65, -0.5),
    );
    var colors = array<vec4<f32>, 3>(
        vec4<f32>(1.0, 0.25, 0.25, 1.0),
        vec4<f32>(0.25, 1.0, 0.35, 1.0),
        vec4<f32>(0.3, 0.5, 1.0, 1.0),
    );

    let a = pc.angle_aspect.x;
    let aspect = pc.angle_aspect.y;
    let c = cos(a);
    let s = sin(a);
    let p = positions[i];
    let rp = vec2<f32>(c * p.x - s * p.y, s * p.x + c * p.y);

    var out: VOut;
    out.position = vec4<f32>(rp.x / aspect, rp.y, 0.0, 1.0);
    out.color = colors[i];
    return out;
}

@fragment
fn fs_main(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}
