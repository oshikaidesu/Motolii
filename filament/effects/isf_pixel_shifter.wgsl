// Pixel Shifter: ISF-Files' Pixel Shifter.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@hPhase:Horizontal Phase,0,1,0,0.001
// --track@hFrequency:Horizontal Frequency,-16,16,1,0.001
// --track@hRandom:Horizontal Random,0,1,0,0.001
// --track@vPhase:Vertical Phase,0,1,0,0.001
// --track@vFrequency:Vertical Frequency,-16,16,0,0.001
// --track@vRandom:Vertical Random,0,1,0,0.001
// --check@doSin_on:Sinusoidal,1
// --check@mirror_on:Mirror,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    hPhase: f32,
    hFrequency: f32,
    hRandom: f32,
    vPhase: f32,
    vFrequency: f32,
    vRandom: f32,
    doSin_on: i32,
    mirror_on: i32,
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
var<private> PI_CONST: f32 = 3.1415927f;

fn rand(co: vec2<f32>) -> f32 {
    var co_1: vec2<f32>;

    co_1 = co;
    let _e31 = co_1;
    return fract((sin(dot(_e31.xy, vec2<f32>(12.9898f, 78.233f))) * 43758.547f));
}

fn main_1() {
    var loc: vec2<f32>;
    var modVal: f32 = 1f;

    let _e29 = isf_FragNormCoord_1;
    loc = _e29;
    let _e33 = global.mirror_on;
    if (_e33 != 0i) {
        modVal = 2f;
    }
    let _e37 = global.doSin_on;
    if (_e37 != 0i) {
        let _e41 = global.hRandom;
        let _e42 = global.time;
        let _e45 = loc;
        let _e48 = rand(vec2<f32>((_e42 * 0.127f), _e45.x));
        let _e50 = loc;
        let _e53 = global.hFrequency;
        let _e59 = PI_CONST;
        let _e61 = global.hPhase;
        let _e62 = global.hFrequency;
        let _e63 = loc;
        let _e71 = (((_e41 * _e48) + _e50.x) + ((sign(_e53) * 0.5f) * (1f + cos(((2f * _e59) * (_e61 + (_e62 * _e63.y)))))));
        let _e72 = modVal;
        loc.x = (_e71 - (floor((_e71 / _e72)) * _e72));
    } else {
        let _e78 = global.hRandom;
        let _e79 = global.time;
        let _e82 = loc;
        let _e85 = rand(vec2<f32>((_e79 * 0.129f), _e82.x));
        let _e87 = loc;
        let _e90 = global.hFrequency;
        let _e91 = loc;
        let _e95 = global.hPhase;
        let _e96 = ((((_e78 * _e85) + _e87.x) + (_e90 * _e91.y)) + _e95);
        let _e97 = modVal;
        loc.x = (_e96 - (floor((_e96 / _e97)) * _e97));
    }
    let _e102 = global.doSin_on;
    if (_e102 != 0i) {
        let _e106 = global.vRandom;
        let _e107 = global.time;
        let _e110 = loc;
        let _e113 = rand(vec2<f32>((_e107 * 0.273f), _e110.y));
        let _e115 = loc;
        let _e118 = global.vFrequency;
        let _e124 = PI_CONST;
        let _e126 = global.vPhase;
        let _e127 = global.vFrequency;
        let _e128 = loc;
        let _e136 = (((_e106 * _e113) + _e115.y) + ((sign(_e118) * 0.5f) * (1f + cos(((2f * _e124) * (_e126 + (_e127 * _e128.x)))))));
        let _e137 = modVal;
        loc.y = (_e136 - (floor((_e136 / _e137)) * _e137));
    } else {
        let _e143 = global.vRandom;
        let _e144 = global.time;
        let _e147 = loc;
        let _e150 = rand(vec2<f32>((_e144 * 0.341f), _e147.y));
        let _e152 = loc;
        let _e155 = global.vFrequency;
        let _e156 = loc;
        let _e160 = global.vPhase;
        let _e161 = ((((_e143 * _e150) + _e152.y) + (_e155 * _e156.x)) + _e160);
        let _e162 = modVal;
        loc.y = (_e161 - (floor((_e161 / _e162)) * _e162));
    }
    let _e167 = loc;
    if (_e167.x > 1f) {
        let _e173 = loc;
        loc.x = (2f - _e173.x);
    }
    let _e176 = loc;
    if (_e176.y > 1f) {
        let _e182 = loc;
        loc.y = (2f - _e182.y);
    }
    let _e185 = loc;
    let _e187 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e185, 0f);
    isf_FragColor = _e187;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e38 = isf_FragColor;
    return FragmentOutput(_e38);
}
