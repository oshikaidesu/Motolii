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
var isf_picture: texture_2d<f32>;
@group(0) @binding(1) 
var isf_sampler: sampler;
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

    let _e20 = isf_FragNormCoord_1;
    loc = _e20.xy;
    let _e23 = global.centerX;
    if (_e23 == 0f) {
        {
        }
    } else {
        let _e26 = global.centerX;
        if (_e26 == 1f) {
            {
            }
        } else {
            let _e29 = loc;
            let _e31 = global.centerX;
            if (_e29.x < _e31) {
                {
                    let _e34 = loc;
                    let _e36 = global.centerX;
                    loc.x = (_e34.x / _e36);
                    let _e40 = loc;
                    let _e44 = global.midHeight;
                    let _e47 = loc;
                    let _e50 = loc;
                    loc.y = mix((0.5f + ((_e40.y - 0.5f) / _e44)), _e47.y, (1f - _e50.x));
                    let _e55 = loc;
                    let _e57 = global.scrollAmount;
                    loc.x = (_e55.x + _e57);
                    let _e59 = global.seamless_on;
                    if (_e59 != 0i) {
                        {
                            let _e63 = loc;
                            let _e67 = loc;
                            if ((_e63.x <= 1f) || (_e67.x >= 2f)) {
                                let _e72 = loc;
                                local = _e72.x;
                            } else {
                                let _e75 = loc;
                                local = (3f - _e75.x);
                            }
                            let _e79 = local;
                            loc.x = _e79;
                        }
                    }
                    let _e81 = loc;
                    loc.x = (_e81.x - (floor((_e81.x / 1f)) * 1f));
                }
            } else {
                {
                    let _e88 = global.rightScrollOffset;
                    let _e89 = global.scrollAmount;
                    let _e90 = (_e88 + _e89);
                    rightScroll = (_e90 - (floor((_e90 / 2f)) * 2f));
                    let _e99 = loc;
                    let _e103 = global.centerX;
                    loc.x = ((1f - _e99.x) / (1f - _e103));
                    let _e108 = loc;
                    let _e112 = global.midHeight;
                    let _e115 = loc;
                    let _e118 = loc;
                    loc.y = mix((0.5f + ((_e108.y - 0.5f) / _e112)), _e115.y, (1f - _e118.x));
                    let _e123 = loc;
                    let _e125 = rightScroll;
                    loc.x = (_e123.x + _e125);
                    let _e127 = global.seamless_on;
                    if (_e127 != 0i) {
                        {
                            let _e131 = loc;
                            let _e135 = loc;
                            if ((_e131.x <= 1f) || (_e135.x >= 2f)) {
                                let _e140 = loc;
                                local_1 = _e140.x;
                            } else {
                                let _e143 = loc;
                                local_1 = (3f - _e143.x);
                            }
                            let _e147 = local_1;
                            loc.x = _e147;
                        }
                    }
                    let _e149 = loc;
                    loc.x = (_e149.x - (floor((_e149.x / 1f)) * 1f));
                }
            }
        }
    }
    let _e159 = loc;
    let _e163 = loc;
    if ((_e159.y >= 0f) && (_e163.y <= 1f)) {
        let _e168 = loc;
        let _e170 = textureSampleLevel(isf_picture, isf_sampler, _e168, 0f);
        inputPixelColor = _e170;
    }
    let _e171 = inputPixelColor;
    isf_FragColor = _e171;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e27 = isf_FragColor;
    return FragmentOutput(_e27);
}
