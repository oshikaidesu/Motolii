// DotScreen: three.js's DotScreenShader.js (MIT, Copyright (c) 2010-2026 three.js authors, github.com/mrdoob/three.js), its GLSL made WGSL by naga (filament/effects/from_three.py)
// --track@center_x:center_x,0.0,2.0,0.5,0.001
// --track@center_y:center_y,0.0,2.0,0.5,0.001
// --track@angle:angle,0.0,6.28,1.57,0.001
// --track@scale:scale,0.0,4.0,1.0,0.001
// --track@tSize_x:tSize_x,0.0,1024.0,256.0,0.001
// --track@tSize_y:tSize_y,0.0,1024.0,256.0,0.001

struct Knobs {
    center: vec2<f32>,
    angle: f32,
    scale: f32,
    tSize: vec2<f32>,
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

fn pattern() -> f32 {
    var s: f32;
    var c: f32;
    var tex: vec2<f32>;
    var point: vec2<f32>;

    let _e12 = global.angle;
    s = sin(_e12);
    let _e15 = global.angle;
    c = cos(_e15);
    let _e18 = vUv_1;
    let _e19 = global.tSize;
    let _e21 = global.center;
    tex = ((_e18 * _e19) - _e21);
    let _e24 = c;
    let _e25 = tex;
    let _e28 = s;
    let _e29 = tex;
    let _e33 = s;
    let _e34 = tex;
    let _e37 = c;
    let _e38 = tex;
    let _e43 = global.scale;
    point = (vec2<f32>(((_e24 * _e25.x) - (_e28 * _e29.y)), ((_e33 * _e34.x) + (_e37 * _e38.y))) * _e43);
    let _e46 = point;
    let _e49 = point;
    return ((sin(_e46.x) * sin(_e49.y)) * 4f);
}

fn main_1() {
    var color: vec4<f32>;
    var average: f32;

    let _e12 = vUv_1;
    let _e13 = textureSample(tDiffuse_picture, tDiffuse_sampler, _e12);
    color = _e13;
    let _e15 = color;
    let _e17 = color;
    let _e20 = color;
    average = (((_e15.x + _e17.y) + _e20.z) / 3f);
    let _e26 = average;
    let _e31 = pattern();
    let _e33 = vec3((((_e26 * 10f) - 5f) + _e31));
    let _e34 = color;
    three_FragColor = vec4<f32>(_e33.x, _e33.y, _e33.z, _e34.w);
    return;
}

@fragment 
fn main(@location(0) vUv: vec2<f32>) -> FragmentOutput {
    vUv_1 = vUv;
    main_1();
    let _e19 = three_FragColor;
    return FragmentOutput(_e19);
}
