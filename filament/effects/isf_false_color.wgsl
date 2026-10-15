// False Color: ISF-Files' False Color.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by zoidberg), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@brightColor_x:brightColor r,0,1,1.0,0.001
// --track@brightColor_y:brightColor g,0,1,0.9,0.001
// --track@brightColor_z:brightColor b,0,1,0.8,0.001
// --track@brightColor_w:brightColor a,0,1,1.0,0.001
// --track@darkColor_x:darkColor r,0,1,0.3,0.001
// --track@darkColor_y:darkColor g,0,1,0.0,0.001
// --track@darkColor_z:darkColor b,0,1,0.0,0.001
// --track@darkColor_w:darkColor a,0,1,1.0,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    brightColor: vec4<f32>,
    darkColor: vec4<f32>,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

const lumcoeff: vec4<f32> = vec4<f32>(0.2126f, 0.7152f, 0.0722f, 0f);

@group(0) @binding(0) 
var isf_picture: texture_2d<f32>;
@group(0) @binding(1) 
var isf_sampler: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;

fn main_1() {
    var srcPixel: vec4<f32>;
    var luminance: f32;

    let _e15 = isf_FragNormCoord_1;
    let _e17 = textureSampleLevel(isf_picture, isf_sampler, _e15, 0f);
    srcPixel = _e17;
    let _e19 = srcPixel;
    luminance = dot(_e19, lumcoeff);
    let _e22 = global.darkColor;
    let _e23 = _e22.xyz;
    let _e24 = srcPixel;
    let _e30 = global.brightColor;
    let _e31 = _e30.xyz;
    let _e32 = srcPixel;
    let _e38 = luminance;
    isf_FragColor = mix(vec4<f32>(_e23.x, _e23.y, _e23.z, _e24.w), vec4<f32>(_e31.x, _e31.y, _e31.z, _e32.w), vec4(_e38));
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e23 = isf_FragColor;
    return FragmentOutput(_e23);
}
