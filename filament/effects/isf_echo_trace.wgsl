// Echo Trace: ISF-Files' Echo Trace.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Time
// --spill (on a thing: what it makes where the thing is not is light on what is under)
// --track@thresh:Threshold,0,1,0,0.001
// --track@gain:Gain,0,2,1,0.001
// --check@hardCutoff_on:Hard Cutoff,1
// --check@invert_on:Invert,0
// --pass@bufferVariableNameA:1,1,s1,s1
// --pass@:0,0,s1,s1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    thresh: f32,
    gain: f32,
    hardCutoff_on: i32,
    invert_on: i32,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

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

fn main_1() {
    var freshPixel: vec4<f32>;
    var stalePixel: vec4<f32>;
    var brightLevel: f32;

    let _e23 = gl_FragCoord_1;
    let _e25 = global.resolution;
    let _e28 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, (_e23.xy / _e25), 0f);
    freshPixel = _e28;
    let _e30 = gl_FragCoord_1;
    let _e32 = global.resolution;
    let _e35 = textureSampleLevel(isf_tex_bufferVariableNameA, isf_smp_bufferVariableNameA, (_e30.xy / _e32), 0f);
    stalePixel = _e35;
    let _e37 = freshPixel;
    let _e39 = freshPixel;
    let _e42 = freshPixel;
    brightLevel = (((_e37.x + _e39.z) + _e42.y) / 3f);
    let _e48 = global.invert_on;
    if (_e48 != 0i) {
        let _e52 = brightLevel;
        brightLevel = (1f - _e52);
    }
    let _e54 = brightLevel;
    let _e55 = global.gain;
    brightLevel = (_e54 * _e55);
    let _e57 = global.hardCutoff_on;
    if (_e57 != 0i) {
        {
            let _e60 = brightLevel;
            let _e61 = global.thresh;
            if (_e60 < _e61) {
                brightLevel = 1f;
            } else {
                brightLevel = 0f;
            }
        }
    }
    let _e65 = freshPixel;
    let _e66 = stalePixel;
    let _e67 = brightLevel;
    isf_FragColor = mix(_e65, _e66, vec4(_e67));
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e33 = isf_FragColor;
    return FragmentOutput(_e33);
}
