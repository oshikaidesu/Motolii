// Quad Tile: ISF-Files' Quad Tile.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
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
var isf_picture: texture_2d<f32>;
@group(0) @binding(1) 
var isf_sampler: sampler;
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

    let _e23 = global.rotation;
    s = sin((tau * _e23));
    let _e27 = global.rotation;
    c = cos((tau * _e27));
    let _e31 = isf_FragNormCoord_1;
    let _e32 = global.resolution;
    tex = (_e31 * _e32);
    let _e36 = global.size;
    scale = (1f / max(_e36, 0.001f));
    let _e41 = c;
    let _e42 = tex;
    let _e45 = s;
    let _e46 = tex;
    let _e50 = s;
    let _e51 = tex;
    let _e54 = c;
    let _e55 = tex;
    let _e60 = scale;
    point = (vec2<f32>(((_e41 * _e42.x) - (_e45 * _e46.y)), ((_e50 * _e51.x) + (_e54 * _e55.y))) * _e60);
    let _e63 = point;
    let _e64 = global.shift;
    let _e65 = global.resolution;
    let _e68 = global.resolution;
    point = ((_e63 - (_e64 * _e65)) / _e68);
    let _e70 = point;
    let _e72 = vec2(1f);
    point = (_e70 - (floor((_e70 / _e72)) * _e72));
    let _e77 = point;
    if (_e77.x < 0.5f) {
        {
            let _e82 = point;
            let _e84 = global.slide1_;
            let _e86 = (_e82.y + _e84.y);
            point.y = (_e86 - (floor((_e86 / 1f)) * 1f));
        }
    } else {
        {
            let _e93 = point;
            let _e95 = global.slide2_;
            let _e97 = (_e93.y + _e95.y);
            point.y = (_e97 - (floor((_e97 / 1f)) * 1f));
        }
    }
    let _e103 = point;
    if (_e103.y < 0.5f) {
        {
            let _e108 = point;
            let _e110 = global.slide1_;
            let _e112 = (_e108.x + _e110.x);
            point.x = (_e112 - (floor((_e112 / 1f)) * 1f));
        }
    } else {
        {
            let _e119 = point;
            let _e121 = global.slide2_;
            let _e123 = (_e119.x + _e121.x);
            point.x = (_e123 - (floor((_e123 / 1f)) * 1f));
        }
    }
    let _e132 = point;
    point = (vec2(1f) - abs((vec2(1f) - (2f * _e132))));
    let _e143 = center;
    let _e144 = point;
    r = distance(_e143, _e144);
    let _e147 = point;
    let _e149 = center;
    let _e152 = point;
    let _e154 = center;
    a = atan2((_e147.y - _e149.y), (_e152.x - _e154.x));
    let _e159 = a;
    let _e160 = global.angle;
    s = sin((_e159 + (tau * _e160)));
    let _e164 = a;
    let _e165 = global.angle;
    c = cos((_e164 + (tau * _e165)));
    let _e169 = global.resolution;
    let _e171 = global.resolution;
    zoom = (_e169.x / _e171.y);
    let _e176 = r;
    let _e177 = c;
    let _e179 = zoom;
    point.x = (((_e176 * _e177) / _e179) + 0.5f);
    let _e184 = r;
    let _e185 = s;
    let _e187 = zoom;
    point.y = (((_e184 * _e185) / _e187) + 0.5f);
    let _e191 = point;
    return _e191;
}

fn main_1() {
    var pat: vec2<f32>;

    let _e23 = pattern();
    pat = _e23;
    let _e25 = pat;
    let _e27 = textureSampleLevel(isf_picture, isf_sampler, _e25, 0f);
    isf_FragColor = _e27;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e31 = isf_FragColor;
    return FragmentOutput(_e31);
}
