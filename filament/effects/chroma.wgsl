// Chroma: a lens's colour fringe, wild: red and blue drawn apart from green, more the further from the middle (a
// thing's middle, on a thing; the picture's, on the stage), spilling past the edges as light
// --spill (on a thing: what it makes where the thing is not is light on what is under)
// --track@amount:Amount,0,160,48,0.1
// --track@falloff:Falloff,0.3,4,1.1,0.01
// --track@twist:Twist,-1,1,0,0.01
// --track@origin_x:Middle x,0,4000,0,1
// --track@origin_y:Middle y,0,4000,0,1

struct Knobs {
    amount: f32,
    falloff: f32,
    twist: f32,
    origin_x: f32,
    origin_y: f32,
    pad: f32,
    resolution: vec2f,
}

@group(0) @binding(0) var picture: texture_2d<f32>;
@group(0) @binding(1) var near: sampler;
@group(0) @binding(2) var<uniform> k: Knobs;

@fragment fn fs_main(@builtin(position) frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f {
    let middle = select(k.resolution * 0.5, vec2f(k.origin_x, k.origin_y), k.origin_x > 0.0 || k.origin_y > 0.0);
    let d = frag.xy - middle;
    let reach = length(k.resolution) * 0.5;
    let f = pow(clamp(length(d) / reach, 0.0, 1.0), k.falloff);
    let dir = normalize(d + vec2f(1e-4)) ;
    let side = vec2f(-dir.y, dir.x);
    let push = (dir + side * k.twist) * f * k.amount * k.resolution.y / 1080.0 / k.resolution;
    let r = textureSampleLevel(picture, near, uv + push, 0.0);
    let g = textureSampleLevel(picture, near, uv, 0.0);
    let b = textureSampleLevel(picture, near, uv - push, 0.0);
    return vec4f(r.r, g.g, b.b, max(r.a, max(g.a, b.a)));
}
