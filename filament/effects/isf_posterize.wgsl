// Posterize: ISF-Files' Posterize.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@gamma:Gamma,0.5,2,1.25,0.001
// --track@numColors:Quality,3,32,6,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    gamma: f32,
    numColors: f32,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

@group(0) @binding(0) 
var isf_picture: texture_2d<f32>;
@group(0) @binding(1) 
var isf_sampler: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;

fn main_1() {
    var c: vec4<f32>;

    let _e14 = isf_FragNormCoord_1;
    let _e16 = textureSampleLevel(isf_picture, isf_sampler, _e14, 0f);
    c = _e16;
    let _e18 = c;
    let _e20 = c;
    let _e22 = global.gamma;
    let _e23 = global.gamma;
    let _e24 = global.gamma;
    let _e26 = pow(_e20.xyz, vec3<f32>(_e22, _e23, _e24));
    c.x = _e26.x;
    c.y = _e26.y;
    c.z = _e26.z;
    let _e33 = c;
    let _e35 = c;
    let _e37 = global.numColors;
    let _e38 = (_e35.xyz * _e37);
    c.x = _e38.x;
    c.y = _e38.y;
    c.z = _e38.z;
    let _e45 = c;
    let _e47 = c;
    let _e49 = floor(_e47.xyz);
    c.x = _e49.x;
    c.y = _e49.y;
    c.z = _e49.z;
    let _e56 = c;
    let _e58 = c;
    let _e60 = global.numColors;
    let _e62 = (_e58.xyz / vec3(_e60));
    c.x = _e62.x;
    c.y = _e62.y;
    c.z = _e62.z;
    let _e69 = c;
    let _e71 = c;
    let _e74 = global.gamma;
    let _e77 = pow(_e71.xyz, vec3((1f / _e74)));
    c.x = _e77.x;
    c.y = _e77.y;
    c.z = _e77.z;
    let _e84 = c;
    isf_FragColor = _e84;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e21 = isf_FragColor;
    return FragmentOutput(_e21);
}
