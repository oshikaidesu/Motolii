// Cubic Warp: ISF-Files' Cubic Warp.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by carter rosenberg), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Distort
// --track@level:level,0,100,1,0.001
// --track@center_x:center x,0,1,0.5,0.001
// --track@center_y:center y,0,1,0.5,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    level: f32,
    center: vec2<f32>,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

const pi: f32 = 3.1415927f;

@group(0) @binding(0) 
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;

fn distance_(center: vec2<f32>, pt: vec2<f32>) -> f32 {
    var center_1: vec2<f32>;
    var pt_1: vec2<f32>;
    var tmp: f32;

    center_1 = center;
    pt_1 = pt;
    let _e21 = center_1;
    let _e23 = pt_1;
    let _e28 = center_1;
    let _e30 = pt_1;
    tmp = (pow((_e21.x - _e23.x), 2f) + pow((_e28.y - _e30.y), 2f));
    let _e37 = tmp;
    return pow(_e37, 0.5f);
}

fn main_1() {
    var loc: vec2<f32>;
    var modifiedCenter: vec2<f32>;
    var k: f32 = -0.15f;
    var r2_: f32;
    var f: f32 = 0f;
    var zoom: f32;

    let _e19 = isf_FragNormCoord_1;
    loc = _e19;
    let _e20 = global.center;
    modifiedCenter = _e20;
    let _e24 = loc;
    let _e26 = modifiedCenter;
    let _e29 = loc;
    let _e31 = modifiedCenter;
    let _e35 = loc;
    let _e37 = modifiedCenter;
    let _e40 = loc;
    let _e42 = modifiedCenter;
    r2_ = (((_e24.x - _e26.x) * (_e29.x - _e31.x)) + ((_e35.y - _e37.y) * (_e40.y - _e42.y)));
    let _e50 = global.level;
    if (_e50 == 0f) {
        {
            let _e54 = r2_;
            let _e55 = k;
            f = (1f + (_e54 * _e55));
        }
    } else {
        {
            let _e59 = r2_;
            let _e60 = k;
            let _e61 = global.level;
            let _e62 = r2_;
            f = (1f + (_e59 * (_e60 + (_e61 * sqrt(_e62)))));
        }
    }
    let _e68 = global.level;
    zoom = max(sqrt(_e68), 1f);
    let _e74 = f;
    let _e75 = loc;
    let _e77 = modifiedCenter;
    let _e81 = zoom;
    let _e83 = modifiedCenter;
    loc.x = (((_e74 * (_e75.x - _e77.x)) / _e81) + _e83.x);
    let _e87 = f;
    let _e88 = loc;
    let _e90 = modifiedCenter;
    let _e94 = zoom;
    let _e96 = modifiedCenter;
    loc.y = (((_e87 * (_e88.y - _e90.y)) / _e94) + _e96.y);
    let _e99 = loc;
    let _e103 = loc;
    let _e108 = loc;
    let _e113 = loc;
    if ((((_e99.x < 0f) || (_e103.y < 0f)) || (_e108.x > 1f)) || (_e113.y > 1f)) {
        {
            isf_FragColor = vec4(0f);
            return;
        }
    } else {
        {
            let _e120 = loc;
            let _e122 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e120, 0f);
            isf_FragColor = _e122;
            return;
        }
    }
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e25 = isf_FragColor;
    return FragmentOutput(_e25);
}
