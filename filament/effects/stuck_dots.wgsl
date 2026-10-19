// Stuck Dots: a thing's own shade as dots stuck to it, as Spider-Verse's halftone moves with its character: lit places a
// flat colour, shaded ones dots of a darker ink growing with the shade, the grid at the thing's place and turned with it
// --track@cell:Cell,2,40,9,0.1
// --track@angle:Angle,-90,90,22,0.1
// --track@ink:Ink,0,1,0.45,0.01
// --track@lift:Lift,0,1,0.35,0.01
// --track@contrast:Contrast,0.2,3,1.4,0.01

struct Knobs {
    cell: f32,
    angle: f32,
    ink: f32,
    lift: f32,
    contrast: f32,
    origin_x: f32,
    origin_y: f32,
    turn: f32,
    resolution: vec2f,
}

@group(0) @binding(0) var picture: texture_2d<f32>;
@group(0) @binding(1) var near: sampler;
@group(0) @binding(2) var<uniform> k: Knobs;

@fragment fn fs_main(@builtin(position) frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f {
    let px = max(k.cell * k.resolution.y / 1080.0, 1.0);
    let a = radians(k.angle + k.turn);
    let r = mat2x2f(cos(a), sin(a), -sin(a), cos(a));
    let p = r * (frag.xy - vec2f(k.origin_x, k.origin_y)) / px;   // (the grid at the thing, turned with it)
    let mid = floor(p) + 0.5;
    let at = (transpose(r) * mid) * px + vec2f(k.origin_x, k.origin_y);   // (the cell's middle back on the picture)
    let s = textureSampleLevel(picture, near, at / k.resolution, 0.0);
    let c = textureSampleLevel(picture, near, uv, 0.0);
    let col = c.rgb / max(c.a, 1e-4);
    let shade = clamp((1.0 - dot(s.rgb / max(s.a, 1e-4), vec3f(0.2126, 0.7152, 0.0722))) * k.contrast, 0.0, 1.0);
    let radius = sqrt(shade) * 0.72;
    let inDot = 1.0 - smoothstep(radius - 0.06, radius + 0.06, length(p - mid));
    let base = mix(col, vec3f(1.0), k.lift * (1.0 - shade));   // (lit: lifted toward paper)
    let ink = col * (1.0 - k.ink);
    return vec4f(mix(base, ink, inDot) * c.a, c.a);
}
