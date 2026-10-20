// Sine Warp Tile: ISF-Files' Sine Warp Tile.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Distort
// --track@size:size,0,0.5,0.5,0.001
// --track@rotation:rotation,0,1,0,0.001
// --track@angle:angle,0,1,0,0.001
// --track@shift_x:shift x,0,1,0.5,0.001
// --track@shift_y:shift y,0,1,0.5,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
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

    let _e21 = global.rotation;
    s = sin(((tau * _e21) * 0.5f));
    let _e27 = global.rotation;
    c = cos(((tau * _e27) * 0.5f));
    let _e33 = isf_FragNormCoord_1;
    tex = _e33;
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
    let _e64 = scale;
    let _e65 = global.shift;
    point = (_e63 - (_e64 * _e65));
    let _e70 = scale;
    let _e71 = point;
    let _e75 = scale;
    let _e76 = point;
    point = (vec2(0.5f) + (0.5f * vec2<f32>(sin((_e70 * _e71.x)), sin((_e75 * _e76.y)))));
    let _e88 = center;
    let _e89 = point;
    r = distance(_e88, _e89);
    let _e92 = point;
    let _e94 = center;
    let _e97 = point;
    let _e99 = center;
    a = atan2((_e92.y - _e94.y), (_e97.x - _e99.x));
    let _e104 = a;
    let _e105 = global.angle;
    s = sin((_e104 + (tau * _e105)));
    let _e109 = a;
    let _e110 = global.angle;
    c = cos((_e109 + (tau * _e110)));
    let _e114 = s;
    let _e116 = c;
    let _e119 = global.resolution;
    let _e122 = global.resolution;
    zoom = ((max(abs(_e114), abs(_e116)) * _e119.x) / _e122.y);
    let _e127 = r;
    let _e128 = c;
    let _e130 = zoom;
    point.x = (((_e127 * _e128) / _e130) + 0.5f);
    let _e135 = r;
    let _e136 = s;
    let _e138 = zoom;
    point.y = (((_e135 * _e136) / _e138) + 0.5f);
    let _e142 = point;
    return _e142;
}

fn main_1() {
    var pat: vec2<f32>;

    let _e21 = pattern();
    pat = _e21;
    let _e23 = pat;
    let _e25 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e23, 0f);
    isf_FragColor = _e25;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e29 = isf_FragColor;
    return FragmentOutput(_e29);
}
