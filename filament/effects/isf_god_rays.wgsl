// God Rays: ISF-Files' God Rays.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --spill (on a thing: what it makes where the thing is not is light on what is under)
// --track@lightDirDOTviewDir:lightDirDOTviewDir,0,1,0.5,0.001
// --track@lightPositionOnScreen_x:lightPositionOnScreen x,0,1,0.5,0.001
// --track@lightPositionOnScreen_y:lightPositionOnScreen y,0,1,0.5,0.001
// --track@quality:quality,0,1,10,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    lightDirDOTviewDir: f32,
    lightPositionOnScreen: vec2<f32>,
    quality: i32,
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
    var origColor: vec4<f32>;
    var raysColor: vec4<f32>;
    var NUM_SAMPLES: i32;
    var exposure: f32;
    var decay: f32 = 1f;
    var density: f32 = 0.5f;
    var weight: f32 = 6f;
    var illuminationDecay: f32 = 1f;
    var normSrcCoord: vec2<f32>;
    var deltaTextCoord: vec2<f32>;
    var textCoo: vec2<f32>;
    var i: i32 = 0i;
    var tsample: vec4<f32>;
    var p: f32;

    let _e18 = isf_FragNormCoord_1;
    let _e20 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e18, 0f);
    origColor = _e20;
    let _e22 = isf_FragNormCoord_1;
    let _e24 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e22, 0f);
    raysColor = _e24;
    let _e26 = global.quality;
    NUM_SAMPLES = _e26;
    let _e28 = global.lightDirDOTviewDir;
    if (_e28 > 0f) {
        {
            let _e32 = NUM_SAMPLES;
            exposure = (0.1f / f32(_e32));
            let _e48 = isf_FragNormCoord_1.x;
            normSrcCoord.x = _e48;
            let _e52 = isf_FragNormCoord_1.y;
            normSrcCoord.y = _e52;
            let _e53 = normSrcCoord;
            let _e55 = global.lightPositionOnScreen;
            deltaTextCoord = vec2<f32>((_e53.xy - _e55));
            let _e59 = normSrcCoord;
            textCoo = _e59;
            let _e61 = deltaTextCoord;
            let _e63 = NUM_SAMPLES;
            let _e66 = density;
            deltaTextCoord = (_e61 * ((1f / f32(_e63)) * _e66));
            loop {
                let _e71 = i;
                let _e72 = NUM_SAMPLES;
                if !((_e71 < _e72)) {
                    break;
                }
                {
                    let _e78 = textCoo;
                    let _e79 = deltaTextCoord;
                    textCoo = (_e78 - _e79);
                    let _e81 = textCoo;
                    let _e83 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e81, 0f);
                    tsample = _e83;
                    let _e85 = tsample;
                    let _e86 = illuminationDecay;
                    let _e87 = weight;
                    tsample = (_e85 * (_e86 * _e87));
                    let _e90 = raysColor;
                    let _e91 = tsample;
                    raysColor = (_e90 + _e91);
                    let _e93 = illuminationDecay;
                    let _e94 = decay;
                    illuminationDecay = (_e93 * _e94);
                }
                continuing {
                    let _e75 = i;
                    i = (_e75 + 1i);
                }
            }
            let _e96 = raysColor;
            let _e97 = exposure;
            let _e98 = global.lightDirDOTviewDir;
            raysColor = (_e96 * (_e97 * _e98));
            let _e102 = raysColor;
            let _e106 = raysColor;
            let _e111 = raysColor;
            p = (((0.3f * _e102.y) + (0.59f * _e106.x)) + (0.11f * _e111.z));
            let _e116 = origColor;
            let _e117 = p;
            isf_FragColor = (_e116 + vec4(_e117));
            return;
        }
    } else {
        {
            let _e120 = origColor;
            isf_FragColor = _e120;
            return;
        }
    }
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e25 = isf_FragColor;
    return FragmentOutput(_e25);
}
