// Kaleido: three.js's KaleidoShader.js (MIT, Copyright (c) 2010-2026 three.js authors, github.com/mrdoob/three.js), its GLSL made WGSL by naga (filament/effects/from_three.py)
// --spill (on a thing: what it makes where the thing is not is light on what is under)
// --track@sides:sides,0.0,24.0,6.0,0.001
// --track@angle:angle,0.0,1.0,0.0,0.001

struct Knobs {
    sides: f32,
    angle: f32,
}

struct FragmentOutput {
    @location(0) three_FragColor: vec4<f32>,
}

@group(0) @binding(0) 
var tDiffuse_picture: texture_2d<f32>;
@group(0) @binding(1) 
var tDiffuse_sampler: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> vUv_1: vec2<f32>;
var<private> three_FragColor: vec4<f32>;

fn main_1() {
    var p: vec2<f32>;
    var r: f32;
    var a: f32;
    var tau: f32 = 6.2832f;
    var color: vec4<f32>;

    let _e8 = vUv_1;
    p = (_e8 - vec2(0.5f));
    let _e13 = p;
    r = length(_e13);
    let _e16 = p;
    let _e18 = p;
    let _e21 = global.angle;
    a = (atan2(_e16.y, _e18.x) + _e21);
    let _e28 = a;
    let _e29 = tau;
    let _e30 = global.sides;
    let _e31 = (_e29 / _e30);
    a = (_e28 - (floor((_e28 / _e31)) * _e31));
    let _e36 = a;
    let _e37 = tau;
    let _e38 = global.sides;
    a = abs((_e36 - ((_e37 / _e38) / 2f)));
    let _e44 = r;
    let _e45 = a;
    let _e47 = a;
    p = (_e44 * vec2<f32>(cos(_e45), sin(_e47)));
    let _e51 = p;
    let _e55 = textureSample(tDiffuse_picture, tDiffuse_sampler, (_e51 + vec2(0.5f)));
    color = _e55;
    let _e57 = color;
    three_FragColor = _e57;
    return;
}

@fragment 
fn main(@location(0) vUv: vec2<f32>) -> FragmentOutput {
    vUv_1 = vUv;
    main_1();
    let _e15 = three_FragColor;
    return FragmentOutput(_e15);
}
