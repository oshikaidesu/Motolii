// Circle Wrap Distortion: ISF-Files' Circle Wrap Distortion.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Distort
// --track@inputAngle:inputAngle,0,1,0.5,0.001
// --track@inputRadius:inputRadius,0,2,1,0.001
// --track@inputCenter_x:inputCenter x,0,1,0.5,0.001
// --track@inputCenter_y:inputCenter y,0,1,0.5,0.001
// --check@mirror_on:mirror,0
// --check@correctAspect_on:correctAspect,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    inputAngle: f32,
    inputRadius: f32,
    inputCenter: vec2<f32>,
    mirror_on: i32,
    correctAspect_on: i32,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

const tau: f32 = 6.2831855f;
const pi: f32 = 3.1415927f;
const halfpi: f32 = 1.5707964f;

@group(0) @binding(0) 
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;

fn main_1() {
    var inputPixelColor: vec4<f32>;
    var loc: vec2<f32>;
    var center: vec2<f32>;
    var r: f32;
    var theta: f32;
    var local: f32;

    let _e26 = global.inputRadius;
    if (_e26 > 0f) {
        {
            let _e29 = isf_FragNormCoord_1;
            loc = _e29.xy;
            let _e32 = global.inputCenter;
            center = _e32;
            let _e34 = global.correctAspect_on;
            if (_e34 != 0i) {
                {
                    let _e38 = loc;
                    let _e40 = global.resolution;
                    let _e43 = global.resolution;
                    loc.x = ((_e38.x * _e40.x) / _e43.y);
                    let _e47 = center;
                    let _e49 = global.resolution;
                    let _e52 = global.resolution;
                    center.x = ((_e47.x * _e49.x) / _e52.y);
                }
            }
            let _e55 = loc;
            let _e56 = center;
            r = distance(_e55, _e56);
            let _e59 = global.inputAngle;
            let _e62 = loc;
            let _e64 = center;
            let _e67 = loc;
            let _e69 = center;
            theta = ((((_e59 * tau) + pi) + atan2((_e62.x - _e64.x), (_e67.y - _e69.y))) / tau);
            let _e76 = theta;
            theta = (_e76 - (floor((_e76 / 1f)) * 1f));
            let _e82 = theta;
            let _e83 = r;
            let _e86 = global.inputRadius;
            loc = vec2<f32>(_e82, ((_e83 * 2f) / _e86));
            let _e89 = loc;
            let _e93 = loc;
            let _e98 = loc;
            let _e103 = loc;
            if ((((_e89.x < 0f) || (_e93.x > 1f)) || (_e98.y < 0f)) || (_e103.y > 1f)) {
                inputPixelColor = vec4(0f);
            } else {
                {
                    let _e112 = loc;
                    loc.y = (1f - _e112.y);
                    let _e115 = global.mirror_on;
                    if (_e115 != 0i) {
                        {
                            let _e119 = loc;
                            if (_e119.x < 0.5f) {
                                let _e123 = loc;
                                local = (_e123.x * 2f);
                            } else {
                                let _e128 = loc;
                                local = (2f - (_e128.x * 2f));
                            }
                            let _e134 = local;
                            loc.x = _e134;
                        }
                    }
                    let _e135 = loc;
                    let _e137 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e135, 0f);
                    inputPixelColor = _e137;
                }
            }
        }
    }
    let _e138 = inputPixelColor;
    isf_FragColor = _e138;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e35 = isf_FragColor;
    return FragmentOutput(_e35);
}
