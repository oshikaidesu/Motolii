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
    passindex: f32,
    brightColor: vec4<f32>,
    darkColor: vec4<f32>,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

const lumcoeff: vec4<f32> = vec4<f32>(0.2126f, 0.7152f, 0.0722f, 0f);

@group(0) @binding(0) 
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;

fn main_1() {
    var srcPixel: vec4<f32>;
    var luminance: f32;

    let _e17 = isf_FragNormCoord_1;
    let _e19 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e17, 0f);
    srcPixel = _e19;
    let _e21 = srcPixel;
    luminance = dot(_e21, lumcoeff);
    let _e24 = global.darkColor;
    let _e25 = _e24.xyz;
    let _e26 = srcPixel;
    let _e32 = global.brightColor;
    let _e33 = _e32.xyz;
    let _e34 = srcPixel;
    let _e40 = luminance;
    isf_FragColor = mix(vec4<f32>(_e25.x, _e25.y, _e25.z, _e26.w), vec4<f32>(_e33.x, _e33.y, _e33.z, _e34.w), vec4(_e40));
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e25 = isf_FragColor;
    return FragmentOutput(_e25);
}
