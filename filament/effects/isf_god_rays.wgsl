// God Rays: ISF-Files' God Rays.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@lightDirDOTviewDir:lightDirDOTviewDir,0,1,0.5,0.001
// --track@lightPositionOnScreen_x:lightPositionOnScreen x,0,1,0.5,0.001
// --track@lightPositionOnScreen_y:lightPositionOnScreen y,0,1,0.5,0.001
// --track@quality:quality,0,1,10,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    lightDirDOTviewDir: f32,
    lightPositionOnScreen: vec2<f32>,
    quality: i32,
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

    let _e16 = isf_FragNormCoord_1;
    let _e18 = textureSampleLevel(isf_picture, isf_sampler, _e16, 0f);
    origColor = _e18;
    let _e20 = isf_FragNormCoord_1;
    let _e22 = textureSampleLevel(isf_picture, isf_sampler, _e20, 0f);
    raysColor = _e22;
    let _e24 = global.quality;
    NUM_SAMPLES = _e24;
    let _e26 = global.lightDirDOTviewDir;
    if (_e26 > 0f) {
        {
            let _e30 = NUM_SAMPLES;
            exposure = (0.1f / f32(_e30));
            let _e46 = isf_FragNormCoord_1.x;
            normSrcCoord.x = _e46;
            let _e50 = isf_FragNormCoord_1.y;
            normSrcCoord.y = _e50;
            let _e51 = normSrcCoord;
            let _e53 = global.lightPositionOnScreen;
            deltaTextCoord = vec2<f32>((_e51.xy - _e53));
            let _e57 = normSrcCoord;
            textCoo = _e57;
            let _e59 = deltaTextCoord;
            let _e61 = NUM_SAMPLES;
            let _e64 = density;
            deltaTextCoord = (_e59 * ((1f / f32(_e61)) * _e64));
            loop {
                let _e69 = i;
                let _e70 = NUM_SAMPLES;
                if !((_e69 < _e70)) {
                    break;
                }
                {
                    let _e76 = textCoo;
                    let _e77 = deltaTextCoord;
                    textCoo = (_e76 - _e77);
                    let _e79 = textCoo;
                    let _e81 = textureSampleLevel(isf_picture, isf_sampler, _e79, 0f);
                    tsample = _e81;
                    let _e83 = tsample;
                    let _e84 = illuminationDecay;
                    let _e85 = weight;
                    tsample = (_e83 * (_e84 * _e85));
                    let _e88 = raysColor;
                    let _e89 = tsample;
                    raysColor = (_e88 + _e89);
                    let _e91 = illuminationDecay;
                    let _e92 = decay;
                    illuminationDecay = (_e91 * _e92);
                }
                continuing {
                    let _e73 = i;
                    i = (_e73 + 1i);
                }
            }
            let _e94 = raysColor;
            let _e95 = exposure;
            let _e96 = global.lightDirDOTviewDir;
            raysColor = (_e94 * (_e95 * _e96));
            let _e100 = raysColor;
            let _e104 = raysColor;
            let _e109 = raysColor;
            p = (((0.3f * _e100.y) + (0.59f * _e104.x)) + (0.11f * _e109.z));
            let _e114 = origColor;
            let _e115 = p;
            isf_FragColor = (_e114 + vec4(_e115));
            return;
        }
    } else {
        {
            let _e118 = origColor;
            isf_FragColor = _e118;
            return;
        }
    }
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e23 = isf_FragColor;
    return FragmentOutput(_e23);
}
