// Trio Tone: ISF-Files' Trio Tone.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@darkColor_x:darkColor r,0,1,0.0,0.001
// --track@darkColor_y:darkColor g,0,1,0.164,0.001
// --track@darkColor_z:darkColor b,0,1,1.0,0.001
// --track@darkColor_w:darkColor a,0,1,1.0,0.001
// --track@midColor_x:midColor r,0,1,0.0,0.001
// --track@midColor_y:midColor g,0,1,1.0,0.001
// --track@midColor_z:midColor b,0,1,0.0,0.001
// --track@midColor_w:midColor a,0,1,1.0,0.001
// --track@brightColor_x:brightColor r,0,1,1.0,0.001
// --track@brightColor_y:brightColor g,0,1,0.0,0.001
// --track@brightColor_z:brightColor b,0,1,0.0,0.001
// --track@brightColor_w:brightColor a,0,1,1.0,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    darkColor: vec4<f32>,
    midColor: vec4<f32>,
    brightColor: vec4<f32>,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

@group(0) @binding(0) 
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;

fn main_1() {
    var pixcol: vec4<f32>;
    var color1_: vec4<f32> = vec4<f32>(0f, 0f, 0f, 1f);
    var color2_: vec4<f32>;
    var ix: i32 = 0i;
    var lumacoeff: vec4<f32> = vec4<f32>(0.2126f, 0.7152f, 0.0722f, 0f);
    var lum: f32;
    var thermal: vec4<f32>;

    let _e18 = isf_FragNormCoord_1;
    let _e20 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e18, 0f);
    pixcol = _e20;
    let _e28 = global.darkColor;
    color2_ = _e28;
    let _e38 = pixcol;
    let _e39 = lumacoeff;
    lum = dot(_e38, _e39);
    let _e42 = lum;
    if (_e42 > 0.66f) {
        {
            let _e45 = global.midColor;
            color1_ = _e45;
            let _e46 = global.brightColor;
            color2_ = _e46;
            ix = 2i;
        }
    } else {
        let _e48 = lum;
        if (_e48 > 0.33f) {
            {
                let _e51 = global.darkColor;
                color1_ = _e51;
                let _e52 = global.midColor;
                color2_ = _e52;
                ix = 1i;
            }
        }
    }
    let _e54 = color1_;
    let _e55 = color2_;
    let _e56 = lum;
    let _e57 = ix;
    thermal = mix(_e54, _e55, vec4(((_e56 - (f32(_e57) * 0.33f)) / 0.33f)));
    let _e67 = thermal;
    let _e68 = _e67.xyz;
    let _e69 = pixcol;
    isf_FragColor = vec4<f32>(_e68.x, _e68.y, _e68.z, _e69.w);
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e25 = isf_FragColor;
    return FragmentOutput(_e25);
}
