// Slit Scan Mask: ISF-Files' Slit Scan Mask.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@spacing:spacing,0,1.5,1,0.001
// --track@line_width:line_width,0,1,0.33,0.001
// --track@angle:angle,-1,1,0.25,0.001
// --track@shift:shift,0,1,0.5,0.001
// --track@edgeSharpness:edgeSharpness,0,1,0,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    spacing: f32,
    line_width: f32,
    angle: f32,
    shift: f32,
    edgeSharpness: f32,
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
var<private> gl_FragCoord_1: vec4<f32>;

fn pattern() -> f32 {
    var s: f32;
    var c: f32;
    var tex: vec2<f32>;
    var spaced: f32;
    var point: vec2<f32>;
    var d: f32;
    var w: f32;

    let _e21 = global.angle;
    s = sin((_e21 * pi));
    let _e25 = global.angle;
    c = cos((_e25 * pi));
    let _e29 = isf_FragNormCoord_1;
    let _e30 = global.resolution;
    tex = (_e29 * _e30);
    let _e33 = global.resolution;
    let _e35 = global.spacing;
    spaced = (length(_e33) * _e35);
    let _e38 = c;
    let _e39 = tex;
    let _e42 = s;
    let _e43 = tex;
    let _e47 = s;
    let _e48 = tex;
    let _e51 = c;
    let _e52 = tex;
    let _e58 = spaced;
    point = (vec2<f32>(((_e38 * _e39.x) - (_e42 * _e43.y)), ((_e47 * _e48.x) + (_e51 * _e52.y))) * max((1f / _e58), 0.001f));
    let _e64 = point;
    d = _e64.y;
    let _e67 = global.line_width;
    w = _e67;
    let _e69 = w;
    let _e70 = global.spacing;
    if (_e69 > _e70) {
        {
            let _e73 = global.spacing;
            w = (0.99f * _e73);
        }
    }
    let _e75 = d;
    let _e76 = global.shift;
    let _e77 = global.spacing;
    let _e80 = w;
    let _e83 = ((_e75 + (_e76 * _e77)) + (_e80 * 0.5f));
    let _e84 = global.spacing;
    return (_e83 - (floor((_e83 / _e84)) * _e84));
}

fn main_1() {
    var freshPixel: vec4<f32>;
    var result: vec4<f32> = vec4(0f);
    var pat: f32;
    var w_1: f32;
    var percent: f32;

    let _e22 = gl_FragCoord_1;
    let _e24 = global.resolution;
    let _e27 = textureSampleLevel(isf_picture, isf_sampler, (_e22.xy / _e24), 0f);
    freshPixel = _e27;
    let _e32 = pattern();
    pat = _e32;
    let _e34 = global.line_width;
    w_1 = _e34;
    let _e36 = w_1;
    let _e37 = global.spacing;
    if (_e36 > _e37) {
        {
            let _e40 = global.spacing;
            w_1 = (0.99f * _e40);
        }
    }
    let _e42 = pat;
    let _e45 = pat;
    let _e46 = w_1;
    if ((_e42 > 0f) && (_e45 <= _e46)) {
        {
            let _e50 = w_1;
            let _e52 = pat;
            let _e56 = w_1;
            percent = (1f - (abs((_e50 - (2f * _e52))) / _e56));
            let _e60 = percent;
            percent = clamp(_e60, 0f, 1f);
            let _e64 = percent;
            let _e65 = percent;
            let _e70 = global.edgeSharpness;
            percent = mix(_e64, pow(_e65, 0.2f), _e70);
            let _e72 = result;
            let _e73 = freshPixel;
            let _e74 = percent;
            result = mix(_e72, _e73, vec4(_e74));
        }
    }
    let _e77 = result;
    isf_FragColor = _e77;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e31 = isf_FragColor;
    return FragmentOutput(_e31);
}
