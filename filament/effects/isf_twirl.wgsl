// Twirl: ISF-Files' Twirl.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --spill (on a thing: what it makes where the thing is not is light on what is under)
// --track@radius:radius,0,1,5,0.001
// --track@amount:amount,-10,10,0,0.001
// --track@center_x:center x,0,1,0.5,0.001
// --track@center_y:center y,0,1,0.5,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    radius: f32,
    amount: f32,
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

fn main_1() {
    var uv: vec2<f32>;
    var texSize: vec2<f32>;
    var tc: vec2<f32>;
    var radius_sized: f32;
    var dist: f32;
    var percent: f32;
    var theta: f32;
    var s: f32;
    var c: f32;
    var loc: vec2<f32>;
    var color: vec4<f32>;

    let _e21 = isf_FragNormCoord_1.x;
    let _e24 = isf_FragNormCoord_1.y;
    uv = vec2<f32>(_e21, _e24);
    let _e27 = global.resolution;
    texSize = _e27;
    let _e29 = uv;
    let _e30 = texSize;
    tc = (_e29 * _e30);
    let _e33 = global.radius;
    let _e34 = global.resolution;
    let _e36 = global.resolution;
    radius_sized = (_e33 * max(_e34.x, _e36.y));
    let _e41 = tc;
    let _e42 = global.center;
    let _e43 = global.resolution;
    tc = (_e41 - (_e42 * _e43));
    let _e46 = tc;
    dist = length(_e46);
    let _e49 = dist;
    let _e50 = radius_sized;
    if (_e49 < _e50) {
        {
            let _e52 = radius_sized;
            let _e53 = dist;
            let _e55 = radius_sized;
            percent = ((_e52 - _e53) / _e55);
            let _e58 = percent;
            let _e59 = percent;
            let _e61 = global.amount;
            theta = ((((_e58 * _e59) * _e61) * 2f) * pi);
            let _e67 = theta;
            s = sin(_e67);
            let _e70 = theta;
            c = cos(_e70);
            let _e73 = tc;
            let _e74 = c;
            let _e75 = s;
            let _e79 = tc;
            let _e80 = s;
            let _e81 = c;
            tc = vec2<f32>(dot(_e73, vec2<f32>(_e74, -(_e75))), dot(_e79, vec2<f32>(_e80, _e81)));
        }
    }
    let _e85 = tc;
    let _e86 = global.center;
    let _e87 = global.resolution;
    tc = (_e85 + (_e86 * _e87));
    let _e90 = tc;
    let _e91 = texSize;
    loc = (_e90 / _e91);
    let _e94 = loc;
    let _e96 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e94, 0f);
    color = _e96;
    let _e98 = loc;
    let _e102 = loc;
    let _e107 = loc;
    let _e112 = loc;
    if ((((_e98.x < 0f) || (_e102.y < 0f)) || (_e107.x > 1f)) || (_e112.y > 1f)) {
        {
            isf_FragColor = vec4(0f);
            return;
        }
    } else {
        {
            let _e119 = color;
            isf_FragColor = _e119;
            return;
        }
    }
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e27 = isf_FragColor;
    return FragmentOutput(_e27);
}
