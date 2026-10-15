// Hyperspace: ISF-Files' Hyperspace.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@centerX:centerX,0,1,0.5,0.001
// --track@scrollAmount:scrollAmount,0,2,0,0.001
// --track@rightScrollOffset:rightScrollOffset,0,1,0,0.001
// --track@midHeight:midHeight,0,1,0.5,0.001
// --check@seamless_on:seamless,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    centerX: f32,
    scrollAmount: f32,
    rightScrollOffset: f32,
    midHeight: f32,
    seamless_on: i32,
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
    var loc: vec2<f32>;
    var local: f32;
    var rightScroll: f32;
    var local_1: f32;
    var inputPixelColor: vec4<f32> = vec4(0f);

    let _e22 = isf_FragNormCoord_1;
    loc = _e22.xy;
    let _e25 = global.centerX;
    if (_e25 == 0f) {
        {
        }
    } else {
        let _e28 = global.centerX;
        if (_e28 == 1f) {
            {
            }
        } else {
            let _e31 = loc;
            let _e33 = global.centerX;
            if (_e31.x < _e33) {
                {
                    let _e36 = loc;
                    let _e38 = global.centerX;
                    loc.x = (_e36.x / _e38);
                    let _e42 = loc;
                    let _e46 = global.midHeight;
                    let _e49 = loc;
                    let _e52 = loc;
                    loc.y = mix((0.5f + ((_e42.y - 0.5f) / _e46)), _e49.y, (1f - _e52.x));
                    let _e57 = loc;
                    let _e59 = global.scrollAmount;
                    loc.x = (_e57.x + _e59);
                    let _e61 = global.seamless_on;
                    if (_e61 != 0i) {
                        {
                            let _e65 = loc;
                            let _e69 = loc;
                            if ((_e65.x <= 1f) || (_e69.x >= 2f)) {
                                let _e74 = loc;
                                local = _e74.x;
                            } else {
                                let _e77 = loc;
                                local = (3f - _e77.x);
                            }
                            let _e81 = local;
                            loc.x = _e81;
                        }
                    }
                    let _e83 = loc;
                    loc.x = (_e83.x - (floor((_e83.x / 1f)) * 1f));
                }
            } else {
                {
                    let _e90 = global.rightScrollOffset;
                    let _e91 = global.scrollAmount;
                    let _e92 = (_e90 + _e91);
                    rightScroll = (_e92 - (floor((_e92 / 2f)) * 2f));
                    let _e101 = loc;
                    let _e105 = global.centerX;
                    loc.x = ((1f - _e101.x) / (1f - _e105));
                    let _e110 = loc;
                    let _e114 = global.midHeight;
                    let _e117 = loc;
                    let _e120 = loc;
                    loc.y = mix((0.5f + ((_e110.y - 0.5f) / _e114)), _e117.y, (1f - _e120.x));
                    let _e125 = loc;
                    let _e127 = rightScroll;
                    loc.x = (_e125.x + _e127);
                    let _e129 = global.seamless_on;
                    if (_e129 != 0i) {
                        {
                            let _e133 = loc;
                            let _e137 = loc;
                            if ((_e133.x <= 1f) || (_e137.x >= 2f)) {
                                let _e142 = loc;
                                local_1 = _e142.x;
                            } else {
                                let _e145 = loc;
                                local_1 = (3f - _e145.x);
                            }
                            let _e149 = local_1;
                            loc.x = _e149;
                        }
                    }
                    let _e151 = loc;
                    loc.x = (_e151.x - (floor((_e151.x / 1f)) * 1f));
                }
            }
        }
    }
    let _e161 = loc;
    let _e165 = loc;
    if ((_e161.y >= 0f) && (_e165.y <= 1f)) {
        let _e170 = loc;
        let _e172 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e170, 0f);
        inputPixelColor = _e172;
    }
    let _e173 = inputPixelColor;
    isf_FragColor = _e173;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e29 = isf_FragColor;
    return FragmentOutput(_e29);
}
