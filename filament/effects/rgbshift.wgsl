// RGBShift: three.js's RGBShiftShader.js (MIT, Copyright (c) 2010-2026 three.js authors, github.com/mrdoob/three.js), its GLSL made WGSL by naga (filament/effects/from_three.py)
// --spill (on a thing: what it makes where the thing is not is light on what is under)
// --track@amount:amount,0.0,1.0,0.005,0.001
// --track@angle:angle,0.0,1.0,0.0,0.001

struct Knobs {
    amount: f32,
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
    var offset: vec2<f32>;
    var cr: vec4<f32>;
    var cga: vec4<f32>;
    var cb: vec4<f32>;

    let _e8 = global.amount;
    let _e9 = global.angle;
    let _e11 = global.angle;
    offset = (_e8 * vec2<f32>(cos(_e9), sin(_e11)));
    let _e16 = vUv_1;
    let _e17 = offset;
    let _e19 = textureSample(tDiffuse_picture, tDiffuse_sampler, (_e16 + _e17));
    cr = _e19;
    let _e21 = vUv_1;
    let _e22 = textureSample(tDiffuse_picture, tDiffuse_sampler, _e21);
    cga = _e22;
    let _e24 = vUv_1;
    let _e25 = offset;
    let _e27 = textureSample(tDiffuse_picture, tDiffuse_sampler, (_e24 - _e25));
    cb = _e27;
    let _e29 = cr;
    let _e31 = cga;
    let _e33 = cb;
    let _e35 = cga;
    three_FragColor = vec4<f32>(_e29.x, _e31.y, _e33.z, _e35.w);
    return;
}

@fragment 
fn main(@location(0) vUv: vec2<f32>) -> FragmentOutput {
    vUv_1 = vUv;
    main_1();
    let _e15 = three_FragColor;
    return FragmentOutput(_e15);
}
