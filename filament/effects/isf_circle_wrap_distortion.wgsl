// Circle Wrap Distortion: ISF-Files' Circle Wrap Distortion.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
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
var isf_picture: texture_2d<f32>;
@group(0) @binding(1) 
var isf_sampler: sampler;
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

    let _e24 = global.inputRadius;
    if (_e24 > 0f) {
        {
            let _e27 = isf_FragNormCoord_1;
            loc = _e27.xy;
            let _e30 = global.inputCenter;
            center = _e30;
            let _e32 = global.correctAspect_on;
            if (_e32 != 0i) {
                {
                    let _e36 = loc;
                    let _e38 = global.resolution;
                    let _e41 = global.resolution;
                    loc.x = ((_e36.x * _e38.x) / _e41.y);
                    let _e45 = center;
                    let _e47 = global.resolution;
                    let _e50 = global.resolution;
                    center.x = ((_e45.x * _e47.x) / _e50.y);
                }
            }
            let _e53 = loc;
            let _e54 = center;
            r = distance(_e53, _e54);
            let _e57 = global.inputAngle;
            let _e60 = loc;
            let _e62 = center;
            let _e65 = loc;
            let _e67 = center;
            theta = ((((_e57 * tau) + pi) + atan2((_e60.x - _e62.x), (_e65.y - _e67.y))) / tau);
            let _e74 = theta;
            theta = (_e74 - (floor((_e74 / 1f)) * 1f));
            let _e80 = theta;
            let _e81 = r;
            let _e84 = global.inputRadius;
            loc = vec2<f32>(_e80, ((_e81 * 2f) / _e84));
            let _e87 = loc;
            let _e91 = loc;
            let _e96 = loc;
            let _e101 = loc;
            if ((((_e87.x < 0f) || (_e91.x > 1f)) || (_e96.y < 0f)) || (_e101.y > 1f)) {
                inputPixelColor = vec4(0f);
            } else {
                {
                    let _e110 = loc;
                    loc.y = (1f - _e110.y);
                    let _e113 = global.mirror_on;
                    if (_e113 != 0i) {
                        {
                            let _e117 = loc;
                            if (_e117.x < 0.5f) {
                                let _e121 = loc;
                                local = (_e121.x * 2f);
                            } else {
                                let _e126 = loc;
                                local = (2f - (_e126.x * 2f));
                            }
                            let _e132 = local;
                            loc.x = _e132;
                        }
                    }
                    let _e133 = loc;
                    let _e135 = textureSampleLevel(isf_picture, isf_sampler, _e133, 0f);
                    inputPixelColor = _e135;
                }
            }
        }
    }
    let _e136 = inputPixelColor;
    isf_FragColor = _e136;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e33 = isf_FragColor;
    return FragmentOutput(_e33);
}
