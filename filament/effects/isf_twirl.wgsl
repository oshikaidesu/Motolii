// Twirl: ISF-Files' Twirl.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@radius:radius,0,1,5,0.001
// --track@amount:amount,-10,10,0,0.001
// --track@center_x:center x,0,1,0.5,0.001
// --track@center_y:center y,0,1,0.5,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    radius: f32,
    amount: f32,
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

    let _e19 = isf_FragNormCoord_1.x;
    let _e22 = isf_FragNormCoord_1.y;
    uv = vec2<f32>(_e19, _e22);
    let _e25 = global.resolution;
    texSize = _e25;
    let _e27 = uv;
    let _e28 = texSize;
    tc = (_e27 * _e28);
    let _e31 = global.radius;
    let _e32 = global.resolution;
    let _e34 = global.resolution;
    radius_sized = (_e31 * max(_e32.x, _e34.y));
    let _e39 = tc;
    let _e40 = global.center;
    let _e41 = global.resolution;
    tc = (_e39 - (_e40 * _e41));
    let _e44 = tc;
    dist = length(_e44);
    let _e47 = dist;
    let _e48 = radius_sized;
    if (_e47 < _e48) {
        {
            let _e50 = radius_sized;
            let _e51 = dist;
            let _e53 = radius_sized;
            percent = ((_e50 - _e51) / _e53);
            let _e56 = percent;
            let _e57 = percent;
            let _e59 = global.amount;
            theta = ((((_e56 * _e57) * _e59) * 2f) * pi);
            let _e65 = theta;
            s = sin(_e65);
            let _e68 = theta;
            c = cos(_e68);
            let _e71 = tc;
            let _e72 = c;
            let _e73 = s;
            let _e77 = tc;
            let _e78 = s;
            let _e79 = c;
            tc = vec2<f32>(dot(_e71, vec2<f32>(_e72, -(_e73))), dot(_e77, vec2<f32>(_e78, _e79)));
        }
    }
    let _e83 = tc;
    let _e84 = global.center;
    let _e85 = global.resolution;
    tc = (_e83 + (_e84 * _e85));
    let _e88 = tc;
    let _e89 = texSize;
    loc = (_e88 / _e89);
    let _e92 = loc;
    let _e94 = textureSampleLevel(isf_picture, isf_sampler, _e92, 0f);
    color = _e94;
    let _e96 = loc;
    let _e100 = loc;
    let _e105 = loc;
    let _e110 = loc;
    if ((((_e96.x < 0f) || (_e100.y < 0f)) || (_e105.x > 1f)) || (_e110.y > 1f)) {
        {
            isf_FragColor = vec4(0f);
            return;
        }
    } else {
        {
            let _e117 = color;
            isf_FragColor = _e117;
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
