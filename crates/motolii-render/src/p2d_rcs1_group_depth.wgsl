struct P2dRcs1Uniform {
    rect: vec4<f32>,
    depth: vec4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> u: P2dRcs1Uniform;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
    var position: vec2<f32>;
    switch (vid) {
        case 0u: {
            position = vec2<f32>(u.rect.x, u.rect.y);
        }
        case 1u: {
            position = vec2<f32>(u.rect.z, u.rect.y);
        }
        case 2u: {
            position = vec2<f32>(u.rect.z, u.rect.w);
        }
        case 3u: {
            position = vec2<f32>(u.rect.x, u.rect.y);
        }
        case 4u: {
            position = vec2<f32>(u.rect.z, u.rect.w);
        }
        case 5u: {
            position = vec2<f32>(u.rect.x, u.rect.w);
        }
        default: {
            position = vec2<f32>(u.rect.x, u.rect.y);
        }
    }
    return VsOut(vec4<f32>(position, u.depth.x, 1.0), u.color);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return in.color;
}
