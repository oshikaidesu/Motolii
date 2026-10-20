// Sphere Map: ISF-Files' Sphere Map.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Distort
// --track@imageScale:Image Scale,0.125,1,0.5,0.001
// --track@radiusScale:Radius Scale,0,1.999,1,0.001
// --track@pointInput_x:Rotate x,0,1,0,0.001
// --track@pointInput_y:Rotate y,0,1,0,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    imageScale: f32,
    radiusScale: f32,
    pointInput: vec2<f32>,
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
    var inputPixelColor: vec4<f32> = vec4(0f);
    var rotate: vec2<f32>;
    var p: vec2<f32>;
    var aspect: f32;
    var r: f32;
    var uv: vec2<f32>;
    var f: f32;

    let _e22 = global.pointInput;
    rotate = _e22;
    let _e25 = isf_FragNormCoord_1;
    p = ((2f * _e25.xy) - vec2(1f));
    let _e32 = global.resolution;
    let _e34 = global.resolution;
    aspect = (_e32.x / _e34.y);
    let _e39 = p;
    let _e41 = aspect;
    p.x = (_e39.x * _e41);
    let _e43 = p;
    let _e44 = p;
    let _e48 = global.radiusScale;
    r = (sqrt(dot(_e43, _e44)) * (2f - _e48));
    let _e52 = r;
    if (_e52 < 1f) {
        {
            let _e56 = global.imageScale;
            let _e59 = r;
            let _e64 = r;
            f = ((_e56 * (1f - sqrt((1f - _e59)))) / _e64);
            let _e68 = p;
            let _e70 = f;
            let _e72 = rotate;
            let _e74 = ((_e68.x * _e70) + _e72.x);
            uv.x = (_e74 - (floor((_e74 / 1f)) * 1f));
            let _e81 = p;
            let _e83 = f;
            let _e85 = rotate;
            let _e87 = ((_e81.y * _e83) + _e85.y);
            uv.y = (_e87 - (floor((_e87 / 1f)) * 1f));
            let _e93 = uv;
            let _e95 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e93, 0f);
            inputPixelColor = _e95;
        }
    }
    let _e96 = inputPixelColor;
    isf_FragColor = _e96;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e27 = isf_FragColor;
    return FragmentOutput(_e27);
}
