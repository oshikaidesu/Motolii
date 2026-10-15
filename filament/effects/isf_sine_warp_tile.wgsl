// Sine Warp Tile: ISF-Files' Sine Warp Tile.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@size:size,0,0.5,0.5,0.001
// --track@rotation:rotation,0,1,0,0.001
// --track@angle:angle,0,1,0,0.001
// --track@shift_x:shift x,0,1,0.5,0.001
// --track@shift_y:shift y,0,1,0.5,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    size: f32,
    rotation: f32,
    angle: f32,
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

    let _e19 = global.rotation;
    s = sin(((tau * _e19) * 0.5f));
    let _e25 = global.rotation;
    c = cos(((tau * _e25) * 0.5f));
    let _e31 = isf_FragNormCoord_1;
    tex = _e31;
    let _e34 = global.size;
    scale = (1f / max(_e34, 0.001f));
    let _e39 = c;
    let _e40 = tex;
    let _e43 = s;
    let _e44 = tex;
    let _e48 = s;
    let _e49 = tex;
    let _e52 = c;
    let _e53 = tex;
    let _e58 = scale;
    point = (vec2<f32>(((_e39 * _e40.x) - (_e43 * _e44.y)), ((_e48 * _e49.x) + (_e52 * _e53.y))) * _e58);
    let _e61 = point;
    let _e62 = scale;
    let _e63 = global.shift;
    point = (_e61 - (_e62 * _e63));
    let _e68 = scale;
    let _e69 = point;
    let _e73 = scale;
    let _e74 = point;
    point = (vec2(0.5f) + (0.5f * vec2<f32>(sin((_e68 * _e69.x)), sin((_e73 * _e74.y)))));
    let _e86 = center;
    let _e87 = point;
    r = distance(_e86, _e87);
    let _e90 = point;
    let _e92 = center;
    let _e95 = point;
    let _e97 = center;
    a = atan2((_e90.y - _e92.y), (_e95.x - _e97.x));
    let _e102 = a;
    let _e103 = global.angle;
    s = sin((_e102 + (tau * _e103)));
    let _e107 = a;
    let _e108 = global.angle;
    c = cos((_e107 + (tau * _e108)));
    let _e112 = s;
    let _e114 = c;
    let _e117 = global.resolution;
    let _e120 = global.resolution;
    zoom = ((max(abs(_e112), abs(_e114)) * _e117.x) / _e120.y);
    let _e125 = r;
    let _e126 = c;
    let _e128 = zoom;
    point.x = (((_e125 * _e126) / _e128) + 0.5f);
    let _e133 = r;
    let _e134 = s;
    let _e136 = zoom;
    point.y = (((_e133 * _e134) / _e136) + 0.5f);
    let _e140 = point;
    return _e140;
}

fn main_1() {
    var pat: vec2<f32>;

    let _e19 = pattern();
    pat = _e19;
    let _e21 = pat;
    let _e23 = textureSampleLevel(isf_picture, isf_sampler, _e21, 0f);
    isf_FragColor = _e23;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e27 = isf_FragColor;
    return FragmentOutput(_e27);
}
