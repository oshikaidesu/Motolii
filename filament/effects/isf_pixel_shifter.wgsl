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
var isf_picture: texture_2d<f32>;
@group(0) @binding(1) 
var isf_sampler: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;
var<private> PI_CONST: f32 = 3.1415927f;

fn rand(co: vec2<f32>) -> f32 {
    var co_1: vec2<f32>;

    co_1 = co;
    let _e29 = co_1;
    return fract((sin(dot(_e29.xy, vec2<f32>(12.9898f, 78.233f))) * 43758.547f));
}

fn main_1() {
    var loc: vec2<f32>;
    var modVal: f32 = 1f;

    let _e27 = isf_FragNormCoord_1;
    loc = _e27;
    let _e31 = global.mirror_on;
    if (_e31 != 0i) {
        modVal = 2f;
    }
    let _e35 = global.doSin_on;
    if (_e35 != 0i) {
        let _e39 = global.hRandom;
        let _e40 = global.time;
        let _e43 = loc;
        let _e46 = rand(vec2<f32>((_e40 * 0.127f), _e43.x));
        let _e48 = loc;
        let _e51 = global.hFrequency;
        let _e57 = PI_CONST;
        let _e59 = global.hPhase;
        let _e60 = global.hFrequency;
        let _e61 = loc;
        let _e69 = (((_e39 * _e46) + _e48.x) + ((sign(_e51) * 0.5f) * (1f + cos(((2f * _e57) * (_e59 + (_e60 * _e61.y)))))));
        let _e70 = modVal;
        loc.x = (_e69 - (floor((_e69 / _e70)) * _e70));
    } else {
        let _e76 = global.hRandom;
        let _e77 = global.time;
        let _e80 = loc;
        let _e83 = rand(vec2<f32>((_e77 * 0.129f), _e80.x));
        let _e85 = loc;
        let _e88 = global.hFrequency;
        let _e89 = loc;
        let _e93 = global.hPhase;
        let _e94 = ((((_e76 * _e83) + _e85.x) + (_e88 * _e89.y)) + _e93);
        let _e95 = modVal;
        loc.x = (_e94 - (floor((_e94 / _e95)) * _e95));
    }
    let _e100 = global.doSin_on;
    if (_e100 != 0i) {
        let _e104 = global.vRandom;
        let _e105 = global.time;
        let _e108 = loc;
        let _e111 = rand(vec2<f32>((_e105 * 0.273f), _e108.y));
        let _e113 = loc;
        let _e116 = global.vFrequency;
        let _e122 = PI_CONST;
        let _e124 = global.vPhase;
        let _e125 = global.vFrequency;
        let _e126 = loc;
        let _e134 = (((_e104 * _e111) + _e113.y) + ((sign(_e116) * 0.5f) * (1f + cos(((2f * _e122) * (_e124 + (_e125 * _e126.x)))))));
        let _e135 = modVal;
        loc.y = (_e134 - (floor((_e134 / _e135)) * _e135));
    } else {
        let _e141 = global.vRandom;
        let _e142 = global.time;
        let _e145 = loc;
        let _e148 = rand(vec2<f32>((_e142 * 0.341f), _e145.y));
        let _e150 = loc;
        let _e153 = global.vFrequency;
        let _e154 = loc;
        let _e158 = global.vPhase;
        let _e159 = ((((_e141 * _e148) + _e150.y) + (_e153 * _e154.x)) + _e158);
        let _e160 = modVal;
        loc.y = (_e159 - (floor((_e159 / _e160)) * _e160));
    }
    let _e165 = loc;
    if (_e165.x > 1f) {
        let _e171 = loc;
        loc.x = (2f - _e171.x);
    }
    let _e174 = loc;
    if (_e174.y > 1f) {
        let _e180 = loc;
        loc.y = (2f - _e180.y);
    }
    let _e183 = loc;
    let _e185 = textureSampleLevel(isf_picture, isf_sampler, _e183, 0f);
    isf_FragColor = _e185;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e36 = isf_FragColor;
    return FragmentOutput(_e36);
}
