// Cubic Warp: ISF-Files' Cubic Warp.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by carter rosenberg), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@level:level,0,100,1,0.001
// --track@center_x:center x,0,1,0.5,0.001
// --track@center_y:center y,0,1,0.5,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    level: f32,
    center: vec2<f32>,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

const pi: f32 = 3.1415927f;

@group(0) @binding(0) 
var isf_picture: texture_2d<f32>;
@group(0) @binding(1) 
var isf_sampler: sampler;
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
    let _e19 = center_1;
    let _e21 = pt_1;
    let _e26 = center_1;
    let _e28 = pt_1;
    tmp = (pow((_e19.x - _e21.x), 2f) + pow((_e26.y - _e28.y), 2f));
    let _e35 = tmp;
    return pow(_e35, 0.5f);
}

fn main_1() {
    var loc: vec2<f32>;
    var modifiedCenter: vec2<f32>;
    var k: f32 = -0.15f;
    var r2_: f32;
    var f: f32 = 0f;
    var zoom: f32;

    let _e17 = isf_FragNormCoord_1;
    loc = _e17;
    let _e18 = global.center;
    modifiedCenter = _e18;
    let _e22 = loc;
    let _e24 = modifiedCenter;
    let _e27 = loc;
    let _e29 = modifiedCenter;
    let _e33 = loc;
    let _e35 = modifiedCenter;
    let _e38 = loc;
    let _e40 = modifiedCenter;
    r2_ = (((_e22.x - _e24.x) * (_e27.x - _e29.x)) + ((_e33.y - _e35.y) * (_e38.y - _e40.y)));
    let _e48 = global.level;
    if (_e48 == 0f) {
        {
            let _e52 = r2_;
            let _e53 = k;
            f = (1f + (_e52 * _e53));
        }
    } else {
        {
            let _e57 = r2_;
            let _e58 = k;
            let _e59 = global.level;
            let _e60 = r2_;
            f = (1f + (_e57 * (_e58 + (_e59 * sqrt(_e60)))));
        }
    }
    let _e66 = global.level;
    zoom = max(sqrt(_e66), 1f);
    let _e72 = f;
    let _e73 = loc;
    let _e75 = modifiedCenter;
    let _e79 = zoom;
    let _e81 = modifiedCenter;
    loc.x = (((_e72 * (_e73.x - _e75.x)) / _e79) + _e81.x);
    let _e85 = f;
    let _e86 = loc;
    let _e88 = modifiedCenter;
    let _e92 = zoom;
    let _e94 = modifiedCenter;
    loc.y = (((_e85 * (_e86.y - _e88.y)) / _e92) + _e94.y);
    let _e97 = loc;
    let _e101 = loc;
    let _e106 = loc;
    let _e111 = loc;
    if ((((_e97.x < 0f) || (_e101.y < 0f)) || (_e106.x > 1f)) || (_e111.y > 1f)) {
        {
            isf_FragColor = vec4(0f);
            return;
        }
    } else {
        {
            let _e118 = loc;
            let _e120 = textureSampleLevel(isf_picture, isf_sampler, _e118, 0f);
            isf_FragColor = _e120;
            return;
        }
    }
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e23 = isf_FragColor;
    return FragmentOutput(_e23);
}
