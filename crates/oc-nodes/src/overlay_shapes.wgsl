struct OverlayUniform {
    shape_kind: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    params0: vec4<f32>,
    params1: vec4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var input_tex: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(0) @binding(2) var<uniform> overlay: OverlayUniform;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    let p = positions[vertex_index];
    var out: VsOut;
    out.pos = vec4<f32>(p, 0.0, 1.0);
    out.uv = p * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

fn dist_to_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let ab = b - a;
    let denom = dot(ab, ab);
    if denom <= 1e-8 {
        return distance(p, a);
    }
    let t = clamp(dot(p - a, ab) / denom, 0.0, 1.0);
    return distance(p, a + ab * t);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let base = textureSample(input_tex, input_sampler, in.uv);
    let px = in.pos.xy;

    if overlay.shape_kind == 0u {
        let inside =
            px.x >= overlay.params0.x &&
            px.x < overlay.params1.x &&
            px.y >= overlay.params0.y &&
            px.y < overlay.params1.y;
        if inside {
            return overlay.color;
        }
        return base;
    }

    if overlay.shape_kind == 1u {
        let center = overlay.params0.xy;
        let radius = overlay.params0.z;
        if distance(px, center) < radius {
            return overlay.color;
        }
        return base;
    }

    let start = overlay.params0.xy;
    let half_width = overlay.params0.z * 0.5;
    let end = overlay.params1.xy;
    if dist_to_segment(px, start, end) < half_width {
        return overlay.color;
    }
    return base;
}
