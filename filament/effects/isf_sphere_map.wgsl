// Sphere Map: ISF-Files' Sphere Map.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@imageScale:Image Scale,0.125,1,0.5,0.001
// --track@radiusScale:Radius Scale,0,1.999,1,0.001
// --track@pointInput_x:Rotate x,0,1,0,0.001
// --track@pointInput_y:Rotate y,0,1,0,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    imageScale: f32,
    radiusScale: f32,
    pointInput: vec2<f32>,
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
    var inputPixelColor: vec4<f32> = vec4(0f);
    var rotate: vec2<f32>;
    var p: vec2<f32>;
    var aspect: f32;
    var r: f32;
    var uv: vec2<f32>;
    var f: f32;

    let _e20 = global.pointInput;
    rotate = _e20;
    let _e23 = isf_FragNormCoord_1;
    p = ((2f * _e23.xy) - vec2(1f));
    let _e30 = global.resolution;
    let _e32 = global.resolution;
    aspect = (_e30.x / _e32.y);
    let _e37 = p;
    let _e39 = aspect;
    p.x = (_e37.x * _e39);
    let _e41 = p;
    let _e42 = p;
    let _e46 = global.radiusScale;
    r = (sqrt(dot(_e41, _e42)) * (2f - _e46));
    let _e50 = r;
    if (_e50 < 1f) {
        {
            let _e54 = global.imageScale;
            let _e57 = r;
            let _e62 = r;
            f = ((_e54 * (1f - sqrt((1f - _e57)))) / _e62);
            let _e66 = p;
            let _e68 = f;
            let _e70 = rotate;
            let _e72 = ((_e66.x * _e68) + _e70.x);
            uv.x = (_e72 - (floor((_e72 / 1f)) * 1f));
            let _e79 = p;
            let _e81 = f;
            let _e83 = rotate;
            let _e85 = ((_e79.y * _e81) + _e83.y);
            uv.y = (_e85 - (floor((_e85 / 1f)) * 1f));
            let _e91 = uv;
            let _e93 = textureSampleLevel(isf_picture, isf_sampler, _e91, 0f);
            inputPixelColor = _e93;
        }
    }
    let _e94 = inputPixelColor;
    isf_FragColor = _e94;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e25 = isf_FragColor;
    return FragmentOutput(_e25);
}
