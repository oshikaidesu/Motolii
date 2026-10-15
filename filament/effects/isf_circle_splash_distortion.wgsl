// Circle Splash Distortion: ISF-Files' Circle Splash Distortion.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@radius:radius,0,0.75,0.125,0.001
// --track@streaks:streaks,0,1,0,0.001
// --track@center_x:center x,0,1,0.5,0.001
// --track@center_y:center y,0,1,0.5,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    radius: f32,
    streaks: f32,
    center: vec2<f32>,
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

fn distance_(inCenter: vec2<f32>, pt: vec2<f32>) -> f32 {
    var inCenter_1: vec2<f32>;
    var pt_1: vec2<f32>;
    var tmp: f32;

    inCenter_1 = inCenter;
    pt_1 = pt;
    let _e20 = inCenter_1;
    let _e22 = pt_1;
    let _e27 = inCenter_1;
    let _e29 = pt_1;
    tmp = (pow((_e20.x - _e22.x), 2f) + pow((_e27.y - _e29.y), 2f));
    let _e36 = tmp;
    return pow(_e36, 0.5f);
}

fn main_1() {
    var uv: vec2<f32>;
    var texSize: vec2<f32>;
    var tc: vec2<f32>;
    var tc2_: vec2<f32>;
    var modifiedCenter: vec2<f32>;
    var r: f32;
    var render_length: f32;
    var a: f32;
    var radius_sized: f32;
    var loc: vec2<f32>;
    var result: vec4<f32>;
    var loc2_: vec2<f32>;
    var mixColor: vec4<f32>;

    let _e18 = isf_FragNormCoord_1.x;
    let _e21 = isf_FragNormCoord_1.y;
    uv = vec2<f32>(_e18, _e21);
    let _e24 = global.resolution;
    texSize = _e24;
    let _e26 = uv;
    let _e27 = texSize;
    tc = (_e26 * _e27);
    let _e30 = uv;
    let _e31 = texSize;
    tc2_ = (_e30 * _e31);
    let _e34 = global.center;
    let _e35 = texSize;
    modifiedCenter = (_e34 * _e35);
    let _e38 = modifiedCenter;
    let _e39 = tc;
    let _e40 = distance_(_e38, _e39);
    r = _e40;
    let _e42 = global.resolution;
    render_length = length(_e42);
    let _e45 = tc;
    let _e47 = modifiedCenter;
    let _e50 = tc;
    let _e52 = modifiedCenter;
    a = atan2((_e45.y - _e47.y), (_e50.x - _e52.x));
    let _e57 = global.radius;
    let _e58 = render_length;
    let _e61 = render_length;
    radius_sized = clamp((_e57 * _e58), 1f, _e61);
    let _e64 = tc;
    let _e65 = modifiedCenter;
    tc = (_e64 - _e65);
    let _e67 = tc2_;
    let _e68 = modifiedCenter;
    tc2_ = (_e67 - _e68);
    let _e70 = r;
    let _e71 = radius_sized;
    if (_e70 < _e71) {
        {
            let _e74 = r;
            let _e75 = a;
            tc.x = (_e74 * cos(_e75));
            let _e79 = r;
            let _e80 = a;
            tc.y = (_e79 * sin(_e80));
            let _e83 = tc;
            tc2_ = _e83;
        }
    } else {
        {
            let _e85 = radius_sized;
            let _e86 = a;
            tc.x = (_e85 * cos(_e86));
            let _e90 = radius_sized;
            let _e91 = a;
            tc.y = (_e90 * sin(_e91));
            let _e95 = radius_sized;
            let _e96 = global.streaks;
            let _e97 = render_length;
            let _e100 = a;
            tc2_.x = ((_e95 + (_e96 * _e97)) * cos(_e100));
            let _e104 = radius_sized;
            let _e105 = global.streaks;
            let _e106 = render_length;
            let _e109 = a;
            tc2_.y = ((_e104 + (_e105 * _e106)) * sin(_e109));
        }
    }
    let _e112 = tc;
    let _e113 = modifiedCenter;
    tc = (_e112 + _e113);
    let _e115 = tc2_;
    let _e116 = modifiedCenter;
    tc2_ = (_e115 + _e116);
    let _e118 = tc;
    let _e119 = texSize;
    loc = (_e118 / _e119);
    let _e122 = loc;
    let _e126 = loc;
    let _e131 = loc;
    let _e136 = loc;
    if ((((_e122.x < 0f) || (_e126.y < 0f)) || (_e131.x > 1f)) || (_e136.y > 1f)) {
        {
            isf_FragColor = vec4(0f);
            return;
        }
    } else {
        {
            let _e143 = loc;
            let _e145 = textureSampleLevel(isf_picture, isf_sampler, _e143, 0f);
            result = _e145;
            let _e147 = global.streaks;
            if (_e147 > 0f) {
                {
                    let _e150 = tc2_;
                    let _e151 = texSize;
                    loc2_ = (_e150 / _e151);
                    let _e154 = loc2_;
                    let _e156 = textureSampleLevel(isf_picture, isf_sampler, _e154, 0f);
                    mixColor = _e156;
                    let _e158 = result;
                    let _e159 = mixColor;
                    let _e161 = r;
                    let _e162 = radius_sized;
                    let _e164 = render_length;
                    let _e167 = global.streaks;
                    result = mix(_e158, _e159, vec4(clamp(((2f * ((_e161 - _e162) / _e164)) * _e167), 0f, 1f)));
                }
            }
            let _e174 = result;
            isf_FragColor = _e174;
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
