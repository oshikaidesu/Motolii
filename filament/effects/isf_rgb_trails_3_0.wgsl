// RGB Trails 3.0: ISF-Files' RGB Trails 3.0.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by zoidberg), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Time
// --spill (on a thing: what it makes where the thing is not is light on what is under)
// --track@rWeight:rWeight,0,1,0,0.001
// --track@gWeight:gWeight,0,1,0,0.001
// --track@bWeight:bWeight,0,1,0,0.001
// --track@aWeight:aWeight,0,1,0.0,0.001
// --pass@accum:1,1,s1,s1
// --pass@:0,0,s1,s1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    rWeight: f32,
    gWeight: f32,
    bWeight: f32,
    aWeight: f32,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

@group(0) @binding(0) 
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
@group(0) @binding(2) 
var isf_tex_accum: texture_2d<f32>;
@group(0) @binding(3) 
var isf_smp_accum: sampler;
@group(0) @binding(4) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;

fn main_1() {
    var freshPixel: vec4<f32>;
    var stalePixel: vec4<f32>;

    let _e22 = global.passindex;
    if (i32(_e22) == 0i) {
        {
            let _e26 = isf_FragNormCoord_1;
            let _e28 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e26, 0f);
            freshPixel = _e28;
            let _e30 = isf_FragNormCoord_1;
            let _e32 = textureSampleLevel(isf_tex_accum, isf_smp_accum, _e30, 0f);
            stalePixel = _e32;
            let _e34 = freshPixel;
            let _e36 = stalePixel;
            let _e38 = global.rWeight;
            let _e40 = freshPixel;
            let _e42 = stalePixel;
            let _e44 = global.gWeight;
            let _e46 = freshPixel;
            let _e48 = stalePixel;
            let _e50 = global.bWeight;
            let _e52 = freshPixel;
            let _e54 = stalePixel;
            let _e56 = global.aWeight;
            isf_FragColor = vec4<f32>(mix(_e34.x, _e36.x, _e38), mix(_e40.y, _e42.y, _e44), mix(_e46.z, _e48.z, _e50), mix(_e52.w, _e54.w, _e56));
            return;
        }
    } else {
        let _e59 = isf_FragNormCoord_1;
        let _e61 = textureSampleLevel(isf_tex_accum, isf_smp_accum, _e59, 0f);
        isf_FragColor = _e61;
        return;
    }
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e31 = isf_FragColor;
    return FragmentOutput(_e31);
}
