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
    darkColor: vec4<f32>,
    midColor: vec4<f32>,
    brightColor: vec4<f32>,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

@group(0) @binding(0) 
var isf_picture: texture_2d<f32>;
@group(0) @binding(1) 
var isf_sampler: sampler;
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

    let _e16 = isf_FragNormCoord_1;
    let _e18 = textureSampleLevel(isf_picture, isf_sampler, _e16, 0f);
    pixcol = _e18;
    let _e26 = global.darkColor;
    color2_ = _e26;
    let _e36 = pixcol;
    let _e37 = lumacoeff;
    lum = dot(_e36, _e37);
    let _e40 = lum;
    if (_e40 > 0.66f) {
        {
            let _e43 = global.midColor;
            color1_ = _e43;
            let _e44 = global.brightColor;
            color2_ = _e44;
            ix = 2i;
        }
    } else {
        let _e46 = lum;
        if (_e46 > 0.33f) {
            {
                let _e49 = global.darkColor;
                color1_ = _e49;
                let _e50 = global.midColor;
                color2_ = _e50;
                ix = 1i;
            }
        }
    }
    let _e52 = color1_;
    let _e53 = color2_;
    let _e54 = lum;
    let _e55 = ix;
    thermal = mix(_e52, _e53, vec4(((_e54 - (f32(_e55) * 0.33f)) / 0.33f)));
    let _e65 = thermal;
    let _e66 = _e65.xyz;
    let _e67 = pixcol;
    isf_FragColor = vec4<f32>(_e66.x, _e66.y, _e66.z, _e67.w);
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e23 = isf_FragColor;
    return FragmentOutput(_e23);
}
