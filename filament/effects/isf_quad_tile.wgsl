// Quad Tile: ISF-Files' Quad Tile.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Distort
// --track@size:size,0,2,0.5,0.001
// --track@rotation:rotation,0,1,0,0.001
// --track@angle:angle,0,1,0,0.001
// --track@slide1_x:slide1 x,0,1,0,0.001
// --track@slide1_y:slide1 y,0,1,0,0.001
// --track@slide2_x:slide2 x,0,1,0,0.001
// --track@slide2_y:slide2 y,0,1,0,0.001
// --track@shift_x:shift x,0,1,0,0.001
// --track@shift_y:shift y,0,1,0,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    size: f32,
    rotation: f32,
    angle: f32,
    slide1_: vec2<f32>,
    slide2_: vec2<f32>,
    shift: vec2<f32>,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

const tau: f32 = 6.2831855f;

@group(0) @binding(0) 
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;

fn pattern() -> vec2<f32> {
    var s: f32;
    var c: f32;
    var tex: vec2<f32>;
    var scale: f32;
    var point: vec2<f32>;
    var center: vec2<f32> = vec2<f32>(0.5f, 0.5f);
    var r: f32;
    var a: f32;
    var zoom: f32;

    let _e25 = global.rotation;
    s = sin((tau * _e25));
    let _e29 = global.rotation;
    c = cos((tau * _e29));
    let _e33 = isf_FragNormCoord_1;
    let _e34 = global.resolution;
    tex = (_e33 * _e34);
    let _e38 = global.size;
    scale = (1f / max(_e38, 0.001f));
    let _e43 = c;
    let _e44 = tex;
    let _e47 = s;
    let _e48 = tex;
    let _e52 = s;
    let _e53 = tex;
    let _e56 = c;
    let _e57 = tex;
    let _e62 = scale;
    point = (vec2<f32>(((_e43 * _e44.x) - (_e47 * _e48.y)), ((_e52 * _e53.x) + (_e56 * _e57.y))) * _e62);
    let _e65 = point;
    let _e66 = global.shift;
    let _e67 = global.resolution;
    let _e70 = global.resolution;
    point = ((_e65 - (_e66 * _e67)) / _e70);
    let _e72 = point;
    let _e74 = vec2(1f);
    point = (_e72 - (floor((_e72 / _e74)) * _e74));
    let _e79 = point;
    if (_e79.x < 0.5f) {
        {
            let _e84 = point;
            let _e86 = global.slide1_;
            let _e88 = (_e84.y + _e86.y);
            point.y = (_e88 - (floor((_e88 / 1f)) * 1f));
        }
    } else {
        {
            let _e95 = point;
            let _e97 = global.slide2_;
            let _e99 = (_e95.y + _e97.y);
            point.y = (_e99 - (floor((_e99 / 1f)) * 1f));
        }
    }
    let _e105 = point;
    if (_e105.y < 0.5f) {
        {
            let _e110 = point;
            let _e112 = global.slide1_;
            let _e114 = (_e110.x + _e112.x);
            point.x = (_e114 - (floor((_e114 / 1f)) * 1f));
        }
    } else {
        {
            let _e121 = point;
            let _e123 = global.slide2_;
            let _e125 = (_e121.x + _e123.x);
            point.x = (_e125 - (floor((_e125 / 1f)) * 1f));
        }
    }
    let _e134 = point;
    point = (vec2(1f) - abs((vec2(1f) - (2f * _e134))));
    let _e145 = center;
    let _e146 = point;
    r = distance(_e145, _e146);
    let _e149 = point;
    let _e151 = center;
    let _e154 = point;
    let _e156 = center;
    a = atan2((_e149.y - _e151.y), (_e154.x - _e156.x));
    let _e161 = a;
    let _e162 = global.angle;
    s = sin((_e161 + (tau * _e162)));
    let _e166 = a;
    let _e167 = global.angle;
    c = cos((_e166 + (tau * _e167)));
    let _e171 = global.resolution;
    let _e173 = global.resolution;
    zoom = (_e171.x / _e173.y);
    let _e178 = r;
    let _e179 = c;
    let _e181 = zoom;
    point.x = (((_e178 * _e179) / _e181) + 0.5f);
    let _e186 = r;
    let _e187 = s;
    let _e189 = zoom;
    point.y = (((_e186 * _e187) / _e189) + 0.5f);
    let _e193 = point;
    return _e193;
}

fn main_1() {
    var pat: vec2<f32>;

    let _e25 = pattern();
    pat = _e25;
    let _e27 = pat;
    let _e29 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e27, 0f);
    isf_FragColor = _e29;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e33 = isf_FragColor;
    return FragmentOutput(_e33);
}
