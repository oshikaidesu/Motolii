// Posterize: ISF-Files' Posterize.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Color
// --track@gamma:Gamma,0.5,2,1.25,0.001
// --track@numColors:Quality,3,32,6,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    gamma: f32,
    numColors: f32,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

@group(0) @binding(0) 
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;

fn main_1() {
    var c: vec4<f32>;

    let _e16 = isf_FragNormCoord_1;
    let _e18 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e16, 0f);
    c = _e18;
    let _e20 = c;
    let _e22 = c;
    let _e24 = global.gamma;
    let _e25 = global.gamma;
    let _e26 = global.gamma;
    let _e28 = pow(_e22.xyz, vec3<f32>(_e24, _e25, _e26));
    c.x = _e28.x;
    c.y = _e28.y;
    c.z = _e28.z;
    let _e35 = c;
    let _e37 = c;
    let _e39 = global.numColors;
    let _e40 = (_e37.xyz * _e39);
    c.x = _e40.x;
    c.y = _e40.y;
    c.z = _e40.z;
    let _e47 = c;
    let _e49 = c;
    let _e51 = floor(_e49.xyz);
    c.x = _e51.x;
    c.y = _e51.y;
    c.z = _e51.z;
    let _e58 = c;
    let _e60 = c;
    let _e62 = global.numColors;
    let _e64 = (_e60.xyz / vec3(_e62));
    c.x = _e64.x;
    c.y = _e64.y;
    c.z = _e64.z;
    let _e71 = c;
    let _e73 = c;
    let _e76 = global.gamma;
    let _e79 = pow(_e73.xyz, vec3((1f / _e76)));
    c.x = _e79.x;
    c.y = _e79.y;
    c.z = _e79.z;
    let _e86 = c;
    isf_FragColor = _e86;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e23 = isf_FragColor;
    return FragmentOutput(_e23);
}
