// Slit Scan: ISF-Files' Slit Scan.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@spacing:spacing,0,1.5,1,0.001
// --track@line_width:line_width,0,1,0.33,0.001
// --track@angle:angle,-1,1,0.25,0.001
// --track@shift:shift,0,1,0.5,0.001
// --pass@bufferVariableNameA:1,0,s1,s1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    spacing: f32,
    line_width: f32,
    angle: f32,
    shift: f32,
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
var isf_tex_bufferVariableNameA: texture_2d<f32>;
@group(0) @binding(3) 
var isf_smp_bufferVariableNameA: sampler;
@group(0) @binding(4) 
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

    let _e23 = global.angle;
    s = sin((_e23 * pi));
    let _e27 = global.angle;
    c = cos((_e27 * pi));
    let _e31 = isf_FragNormCoord_1;
    let _e32 = global.resolution;
    tex = (_e31 * _e32);
    let _e35 = global.resolution;
    let _e37 = global.spacing;
    spaced = (length(_e35) * _e37);
    let _e40 = c;
    let _e41 = tex;
    let _e44 = s;
    let _e45 = tex;
    let _e49 = s;
    let _e50 = tex;
    let _e53 = c;
    let _e54 = tex;
    let _e60 = spaced;
    point = (vec2<f32>(((_e40 * _e41.x) - (_e44 * _e45.y)), ((_e49 * _e50.x) + (_e53 * _e54.y))) * max((1f / _e60), 0.001f));
    let _e66 = point;
    d = _e66.y;
    let _e69 = global.line_width;
    w = _e69;
    let _e71 = w;
    let _e72 = global.spacing;
    if (_e71 > _e72) {
        {
            let _e75 = global.spacing;
            w = (0.99f * _e75);
        }
    }
    let _e77 = d;
    let _e78 = global.shift;
    let _e79 = global.spacing;
    let _e82 = w;
    let _e85 = ((_e77 + (_e78 * _e79)) + (_e82 * 0.5f));
    let _e86 = global.spacing;
    return (_e85 - (floor((_e85 / _e86)) * _e86));
}

fn main_1() {
    var freshPixel: vec4<f32>;
    var result: vec4<f32>;
    var pat: f32;
    var w_1: f32;
    var percent: f32;

    let _e24 = gl_FragCoord_1;
    let _e26 = global.resolution;
    let _e29 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, (_e24.xy / _e26), 0f);
    freshPixel = _e29;
    let _e31 = gl_FragCoord_1;
    let _e33 = global.resolution;
    let _e36 = textureSampleLevel(isf_tex_bufferVariableNameA, isf_smp_bufferVariableNameA, (_e31.xy / _e33), 0f);
    result = _e36;
    let _e38 = pattern();
    pat = _e38;
    let _e40 = global.line_width;
    w_1 = _e40;
    let _e42 = w_1;
    let _e43 = global.spacing;
    if (_e42 > _e43) {
        {
            let _e46 = global.spacing;
            w_1 = (0.99f * _e46);
        }
    }
    let _e48 = pat;
    let _e51 = pat;
    let _e52 = w_1;
    if ((_e48 > 0f) && (_e51 <= _e52)) {
        {
            let _e56 = w_1;
            let _e58 = pat;
            let _e62 = w_1;
            percent = (1f - (abs((_e56 - (2f * _e58))) / _e62));
            let _e66 = percent;
            percent = clamp(_e66, 0f, 1f);
            let _e70 = result;
            let _e71 = freshPixel;
            let _e72 = percent;
            result = mix(_e70, _e71, vec4(_e72));
        }
    }
    let _e75 = result;
    isf_FragColor = _e75;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e35 = isf_FragColor;
    return FragmentOutput(_e35);
}
