// Circle Splash Distortion: ISF-Files' Circle Splash Distortion.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Distort
// --track@radius:radius,0,0.75,0.125,0.001
// --track@streaks:streaks,0,1,0,0.001
// --track@center_x:center x,0,1,0.5,0.001
// --track@center_y:center y,0,1,0.5,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    radius: f32,
    streaks: f32,
    center: vec2<f32>,
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

fn distance_(inCenter: vec2<f32>, pt: vec2<f32>) -> f32 {
    var inCenter_1: vec2<f32>;
    var pt_1: vec2<f32>;
    var tmp: f32;

    inCenter_1 = inCenter;
    pt_1 = pt;
    let _e22 = inCenter_1;
    let _e24 = pt_1;
    let _e29 = inCenter_1;
    let _e31 = pt_1;
    tmp = (pow((_e22.x - _e24.x), 2f) + pow((_e29.y - _e31.y), 2f));
    let _e38 = tmp;
    return pow(_e38, 0.5f);
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

    let _e20 = isf_FragNormCoord_1.x;
    let _e23 = isf_FragNormCoord_1.y;
    uv = vec2<f32>(_e20, _e23);
    let _e26 = global.resolution;
    texSize = _e26;
    let _e28 = uv;
    let _e29 = texSize;
    tc = (_e28 * _e29);
    let _e32 = uv;
    let _e33 = texSize;
    tc2_ = (_e32 * _e33);
    let _e36 = global.center;
    let _e37 = texSize;
    modifiedCenter = (_e36 * _e37);
    let _e40 = modifiedCenter;
    let _e41 = tc;
    let _e42 = distance_(_e40, _e41);
    r = _e42;
    let _e44 = global.resolution;
    render_length = length(_e44);
    let _e47 = tc;
    let _e49 = modifiedCenter;
    let _e52 = tc;
    let _e54 = modifiedCenter;
    a = atan2((_e47.y - _e49.y), (_e52.x - _e54.x));
    let _e59 = global.radius;
    let _e60 = render_length;
    let _e63 = render_length;
    radius_sized = clamp((_e59 * _e60), 1f, _e63);
    let _e66 = tc;
    let _e67 = modifiedCenter;
    tc = (_e66 - _e67);
    let _e69 = tc2_;
    let _e70 = modifiedCenter;
    tc2_ = (_e69 - _e70);
    let _e72 = r;
    let _e73 = radius_sized;
    if (_e72 < _e73) {
        {
            let _e76 = r;
            let _e77 = a;
            tc.x = (_e76 * cos(_e77));
            let _e81 = r;
            let _e82 = a;
            tc.y = (_e81 * sin(_e82));
            let _e85 = tc;
            tc2_ = _e85;
        }
    } else {
        {
            let _e87 = radius_sized;
            let _e88 = a;
            tc.x = (_e87 * cos(_e88));
            let _e92 = radius_sized;
            let _e93 = a;
            tc.y = (_e92 * sin(_e93));
            let _e97 = radius_sized;
            let _e98 = global.streaks;
            let _e99 = render_length;
            let _e102 = a;
            tc2_.x = ((_e97 + (_e98 * _e99)) * cos(_e102));
            let _e106 = radius_sized;
            let _e107 = global.streaks;
            let _e108 = render_length;
            let _e111 = a;
            tc2_.y = ((_e106 + (_e107 * _e108)) * sin(_e111));
        }
    }
    let _e114 = tc;
    let _e115 = modifiedCenter;
    tc = (_e114 + _e115);
    let _e117 = tc2_;
    let _e118 = modifiedCenter;
    tc2_ = (_e117 + _e118);
    let _e120 = tc;
    let _e121 = texSize;
    loc = (_e120 / _e121);
    let _e124 = loc;
    let _e128 = loc;
    let _e133 = loc;
    let _e138 = loc;
    if ((((_e124.x < 0f) || (_e128.y < 0f)) || (_e133.x > 1f)) || (_e138.y > 1f)) {
        {
            isf_FragColor = vec4(0f);
            return;
        }
    } else {
        {
            let _e145 = loc;
            let _e147 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e145, 0f);
            result = _e147;
            let _e149 = global.streaks;
            if (_e149 > 0f) {
                {
                    let _e152 = tc2_;
                    let _e153 = texSize;
                    loc2_ = (_e152 / _e153);
                    let _e156 = loc2_;
                    let _e158 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e156, 0f);
                    mixColor = _e158;
                    let _e160 = result;
                    let _e161 = mixColor;
                    let _e163 = r;
                    let _e164 = radius_sized;
                    let _e166 = render_length;
                    let _e169 = global.streaks;
                    result = mix(_e160, _e161, vec4(clamp(((2f * ((_e163 - _e164) / _e166)) * _e169), 0f, 1f)));
                }
            }
            let _e176 = result;
            isf_FragColor = _e176;
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
