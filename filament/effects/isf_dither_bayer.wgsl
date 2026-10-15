// Dither-Bayer: ISF-Files' Dither-Bayer.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; Hugh Kennedy, adapted by David Lublin), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@colorize:colorize,0,1,0,0.001
// --track@matrixMode:matrixMode,0,1,2,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    colorize: f32,
    matrixMode: i32,
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
var<private> gl_FragCoord_1: vec4<f32>;

fn luma(color: vec3<f32>) -> f32 {
    var color_1: vec3<f32>;

    color_1 = color;
    let _e18 = color_1;
    let _e20 = color_1;
    let _e23 = color_1;
    return (((_e18.x + _e20.y) + _e23.z) / 3f);
}

fn luma_1(color_2: vec4<f32>) -> f32 {
    var color_3: vec4<f32>;

    color_3 = color_2;
    let _e18 = color_3;
    let _e20 = color_3;
    let _e22 = color_3;
    let _e25 = color_3;
    return ((_e18.w * ((_e20.x + _e22.y) + _e25.z)) / 3f);
}

fn dither8x8_(position: vec2<f32>, brightness: f32) -> f32 {
    var position_1: vec2<f32>;
    var brightness_1: f32;
    var x: i32;
    var y: i32;
    var index: i32;
    var limit: f32 = 0f;
    var local: f32;

    position_1 = position;
    brightness_1 = brightness;
    let _e20 = position_1;
    x = i32((_e20.x - (floor((_e20.x / 8f)) * 8f)));
    let _e29 = position_1;
    y = i32((_e29.y - (floor((_e29.y / 8f)) * 8f)));
    let _e38 = x;
    let _e39 = y;
    index = (_e38 + (_e39 * 8i));
    let _e46 = x;
    if (_e46 < 8i) {
        {
            let _e49 = index;
            if (_e49 == 0i) {
                limit = 0.015625f;
            }
            let _e53 = index;
            if (_e53 == 1i) {
                limit = 0.515625f;
            }
            let _e57 = index;
            if (_e57 == 2i) {
                limit = 0.140625f;
            }
            let _e61 = index;
            if (_e61 == 3i) {
                limit = 0.640625f;
            }
            let _e65 = index;
            if (_e65 == 4i) {
                limit = 0.046875f;
            }
            let _e69 = index;
            if (_e69 == 5i) {
                limit = 0.546875f;
            }
            let _e73 = index;
            if (_e73 == 6i) {
                limit = 0.171875f;
            }
            let _e77 = index;
            if (_e77 == 7i) {
                limit = 0.671875f;
            }
            let _e81 = index;
            if (_e81 == 8i) {
                limit = 0.765625f;
            }
            let _e85 = index;
            if (_e85 == 9i) {
                limit = 0.265625f;
            }
            let _e89 = index;
            if (_e89 == 10i) {
                limit = 0.890625f;
            }
            let _e93 = index;
            if (_e93 == 11i) {
                limit = 0.390625f;
            }
            let _e97 = index;
            if (_e97 == 12i) {
                limit = 0.796875f;
            }
            let _e101 = index;
            if (_e101 == 13i) {
                limit = 0.296875f;
            }
            let _e105 = index;
            if (_e105 == 14i) {
                limit = 0.921875f;
            }
            let _e109 = index;
            if (_e109 == 15i) {
                limit = 0.421875f;
            }
            let _e113 = index;
            if (_e113 == 16i) {
                limit = 0.203125f;
            }
            let _e117 = index;
            if (_e117 == 17i) {
                limit = 0.703125f;
            }
            let _e121 = index;
            if (_e121 == 18i) {
                limit = 0.078125f;
            }
            let _e125 = index;
            if (_e125 == 19i) {
                limit = 0.578125f;
            }
            let _e129 = index;
            if (_e129 == 20i) {
                limit = 0.234375f;
            }
            let _e133 = index;
            if (_e133 == 21i) {
                limit = 0.734375f;
            }
            let _e137 = index;
            if (_e137 == 22i) {
                limit = 0.109375f;
            }
            let _e141 = index;
            if (_e141 == 23i) {
                limit = 0.609375f;
            }
            let _e145 = index;
            if (_e145 == 24i) {
                limit = 0.953125f;
            }
            let _e149 = index;
            if (_e149 == 25i) {
                limit = 0.453125f;
            }
            let _e153 = index;
            if (_e153 == 26i) {
                limit = 0.828125f;
            }
            let _e157 = index;
            if (_e157 == 27i) {
                limit = 0.328125f;
            }
            let _e161 = index;
            if (_e161 == 28i) {
                limit = 0.984375f;
            }
            let _e165 = index;
            if (_e165 == 29i) {
                limit = 0.484375f;
            }
            let _e169 = index;
            if (_e169 == 30i) {
                limit = 0.859375f;
            }
            let _e173 = index;
            if (_e173 == 31i) {
                limit = 0.359375f;
            }
            let _e177 = index;
            if (_e177 == 32i) {
                limit = 0.0625f;
            }
            let _e181 = index;
            if (_e181 == 33i) {
                limit = 0.5625f;
            }
            let _e185 = index;
            if (_e185 == 34i) {
                limit = 0.1875f;
            }
            let _e189 = index;
            if (_e189 == 35i) {
                limit = 0.6875f;
            }
            let _e193 = index;
            if (_e193 == 36i) {
                limit = 0.03125f;
            }
            let _e197 = index;
            if (_e197 == 37i) {
                limit = 0.53125f;
            }
            let _e201 = index;
            if (_e201 == 38i) {
                limit = 0.15625f;
            }
            let _e205 = index;
            if (_e205 == 39i) {
                limit = 0.65625f;
            }
            let _e209 = index;
            if (_e209 == 40i) {
                limit = 0.8125f;
            }
            let _e213 = index;
            if (_e213 == 41i) {
                limit = 0.3125f;
            }
            let _e217 = index;
            if (_e217 == 42i) {
                limit = 0.9375f;
            }
            let _e221 = index;
            if (_e221 == 43i) {
                limit = 0.4375f;
            }
            let _e225 = index;
            if (_e225 == 44i) {
                limit = 0.78125f;
            }
            let _e229 = index;
            if (_e229 == 45i) {
                limit = 0.28125f;
            }
            let _e233 = index;
            if (_e233 == 46i) {
                limit = 0.90625f;
            }
            let _e237 = index;
            if (_e237 == 47i) {
                limit = 0.40625f;
            }
            let _e241 = index;
            if (_e241 == 48i) {
                limit = 0.25f;
            }
            let _e245 = index;
            if (_e245 == 49i) {
                limit = 0.75f;
            }
            let _e249 = index;
            if (_e249 == 50i) {
                limit = 0.125f;
            }
            let _e253 = index;
            if (_e253 == 51i) {
                limit = 0.625f;
            }
            let _e257 = index;
            if (_e257 == 52i) {
                limit = 0.21875f;
            }
            let _e261 = index;
            if (_e261 == 53i) {
                limit = 0.71875f;
            }
            let _e265 = index;
            if (_e265 == 54i) {
                limit = 0.09375f;
            }
            let _e269 = index;
            if (_e269 == 55i) {
                limit = 0.59375f;
            }
            let _e273 = index;
            if (_e273 == 56i) {
                limit = 1f;
            }
            let _e277 = index;
            if (_e277 == 57i) {
                limit = 0.5f;
            }
            let _e281 = index;
            if (_e281 == 58i) {
                limit = 0.875f;
            }
            let _e285 = index;
            if (_e285 == 59i) {
                limit = 0.375f;
            }
            let _e289 = index;
            if (_e289 == 60i) {
                limit = 0.96875f;
            }
            let _e293 = index;
            if (_e293 == 61i) {
                limit = 0.46875f;
            }
            let _e297 = index;
            if (_e297 == 62i) {
                limit = 0.84375f;
            }
            let _e301 = index;
            if (_e301 == 63i) {
                limit = 0.34375f;
            }
        }
    }
    let _e305 = brightness_1;
    let _e306 = limit;
    if (_e305 < _e306) {
        local = 0f;
    } else {
        local = 1f;
    }
    let _e311 = local;
    return _e311;
}

fn dither8x8_1(position_2: vec2<f32>, color_4: vec3<f32>) -> vec3<f32> {
    var position_3: vec2<f32>;
    var color_5: vec3<f32>;

    position_3 = position_2;
    color_5 = color_4;
    let _e20 = color_5;
    let _e21 = position_3;
    let _e22 = color_5;
    let _e23 = luma(_e22);
    let _e24 = dither8x8_(_e21, _e23);
    return (_e20 * _e24);
}

fn dither8x8_2(position_4: vec2<f32>, color_6: vec4<f32>) -> vec4<f32> {
    var position_5: vec2<f32>;
    var color_7: vec4<f32>;

    position_5 = position_4;
    color_7 = color_6;
    let _e20 = color_7;
    let _e22 = position_5;
    let _e23 = color_7;
    let _e24 = luma_1(_e23);
    let _e25 = dither8x8_(_e22, _e24);
    let _e26 = (_e20.xyz * _e25);
    return vec4<f32>(_e26.x, _e26.y, _e26.z, 1f);
}

fn dither4x4_(position_6: vec2<f32>, brightness_2: f32) -> f32 {
    var position_7: vec2<f32>;
    var brightness_3: f32;
    var x_1: i32;
    var y_1: i32;
    var index_1: i32;
    var limit_1: f32 = 0f;
    var local_1: f32;

    position_7 = position_6;
    brightness_3 = brightness_2;
    let _e20 = position_7;
    x_1 = i32((_e20.x - (floor((_e20.x / 4f)) * 4f)));
    let _e29 = position_7;
    y_1 = i32((_e29.y - (floor((_e29.y / 4f)) * 4f)));
    let _e38 = x_1;
    let _e39 = y_1;
    index_1 = (_e38 + (_e39 * 4i));
    let _e46 = x_1;
    if (_e46 < 8i) {
        {
            let _e49 = index_1;
            if (_e49 == 0i) {
                limit_1 = 0.0625f;
            }
            let _e53 = index_1;
            if (_e53 == 1i) {
                limit_1 = 0.5625f;
            }
            let _e57 = index_1;
            if (_e57 == 2i) {
                limit_1 = 0.1875f;
            }
            let _e61 = index_1;
            if (_e61 == 3i) {
                limit_1 = 0.6875f;
            }
            let _e65 = index_1;
            if (_e65 == 4i) {
                limit_1 = 0.8125f;
            }
            let _e69 = index_1;
            if (_e69 == 5i) {
                limit_1 = 0.3125f;
            }
            let _e73 = index_1;
            if (_e73 == 6i) {
                limit_1 = 0.9375f;
            }
            let _e77 = index_1;
            if (_e77 == 7i) {
                limit_1 = 0.4375f;
            }
            let _e81 = index_1;
            if (_e81 == 8i) {
                limit_1 = 0.25f;
            }
            let _e85 = index_1;
            if (_e85 == 9i) {
                limit_1 = 0.75f;
            }
            let _e89 = index_1;
            if (_e89 == 10i) {
                limit_1 = 0.125f;
            }
            let _e93 = index_1;
            if (_e93 == 11i) {
                limit_1 = 0.625f;
            }
            let _e97 = index_1;
            if (_e97 == 12i) {
                limit_1 = 1f;
            }
            let _e101 = index_1;
            if (_e101 == 13i) {
                limit_1 = 0.5f;
            }
            let _e105 = index_1;
            if (_e105 == 14i) {
                limit_1 = 0.875f;
            }
            let _e109 = index_1;
            if (_e109 == 15i) {
                limit_1 = 0.375f;
            }
        }
    }
    let _e113 = brightness_3;
    let _e114 = limit_1;
    if (_e113 < _e114) {
        local_1 = 0f;
    } else {
        local_1 = 1f;
    }
    let _e119 = local_1;
    return _e119;
}

fn dither4x4_1(position_8: vec2<f32>, color_8: vec3<f32>) -> vec3<f32> {
    var position_9: vec2<f32>;
    var color_9: vec3<f32>;

    position_9 = position_8;
    color_9 = color_8;
    let _e20 = color_9;
    let _e21 = position_9;
    let _e22 = color_9;
    let _e23 = luma(_e22);
    let _e24 = dither4x4_(_e21, _e23);
    return (_e20 * _e24);
}

fn dither4x4_2(position_10: vec2<f32>, color_10: vec4<f32>) -> vec4<f32> {
    var position_11: vec2<f32>;
    var color_11: vec4<f32>;

    position_11 = position_10;
    color_11 = color_10;
    let _e20 = color_11;
    let _e22 = position_11;
    let _e23 = color_11;
    let _e24 = luma_1(_e23);
    let _e25 = dither4x4_(_e22, _e24);
    let _e26 = (_e20.xyz * _e25);
    return vec4<f32>(_e26.x, _e26.y, _e26.z, 1f);
}

fn dither2x2_(position_12: vec2<f32>, brightness_4: f32) -> f32 {
    var position_13: vec2<f32>;
    var brightness_5: f32;
    var x_2: i32;
    var y_2: i32;
    var index_2: i32;
    var limit_2: f32 = 0f;
    var local_2: f32;

    position_13 = position_12;
    brightness_5 = brightness_4;
    let _e20 = position_13;
    x_2 = i32((_e20.x - (floor((_e20.x / 2f)) * 2f)));
    let _e29 = position_13;
    y_2 = i32((_e29.y - (floor((_e29.y / 2f)) * 2f)));
    let _e38 = x_2;
    let _e39 = y_2;
    index_2 = (_e38 + (_e39 * 2i));
    let _e46 = x_2;
    if (_e46 < 8i) {
        {
            let _e49 = index_2;
            if (_e49 == 0i) {
                limit_2 = 0.25f;
            }
            let _e53 = index_2;
            if (_e53 == 1i) {
                limit_2 = 0.75f;
            }
            let _e57 = index_2;
            if (_e57 == 2i) {
                limit_2 = 1f;
            }
            let _e61 = index_2;
            if (_e61 == 3i) {
                limit_2 = 0.5f;
            }
        }
    }
    let _e65 = brightness_5;
    let _e66 = limit_2;
    if (_e65 < _e66) {
        local_2 = 0f;
    } else {
        local_2 = 1f;
    }
    let _e71 = local_2;
    return _e71;
}

fn dither2x2_1(position_14: vec2<f32>, color_12: vec3<f32>) -> vec3<f32> {
    var position_15: vec2<f32>;
    var color_13: vec3<f32>;

    position_15 = position_14;
    color_13 = color_12;
    let _e20 = color_13;
    let _e21 = position_15;
    let _e22 = color_13;
    let _e23 = luma(_e22);
    let _e24 = dither2x2_(_e21, _e23);
    return (_e20 * _e24);
}

fn dither2x2_2(position_16: vec2<f32>, color_14: vec4<f32>) -> vec4<f32> {
    var position_17: vec2<f32>;
    var color_15: vec4<f32>;

    position_17 = position_16;
    color_15 = color_14;
    let _e20 = color_15;
    let _e22 = position_17;
    let _e23 = color_15;
    let _e24 = luma_1(_e23);
    let _e25 = dither2x2_(_e22, _e24);
    let _e26 = (_e20.xyz * _e25);
    return vec4<f32>(_e26.x, _e26.y, _e26.z, 1f);
}

fn main_1() {
    var inputPixelColor: vec4<f32> = vec4(0f);
    var loc: vec2<f32>;
    var val: f32 = 1f;

    let _e20 = gl_FragCoord_1;
    loc = _e20.xy;
    let _e23 = isf_FragNormCoord_1;
    let _e25 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e23, 0f);
    inputPixelColor = _e25;
    let _e28 = global.matrixMode;
    if (_e28 == 0i) {
        {
            let _e31 = loc;
            let _e32 = inputPixelColor;
            let _e33 = luma_1(_e32);
            let _e34 = dither2x2_(_e31, _e33);
            val = _e34;
        }
    } else {
        let _e35 = global.matrixMode;
        if (_e35 == 1i) {
            {
                let _e38 = loc;
                let _e39 = inputPixelColor;
                let _e40 = luma_1(_e39);
                let _e41 = dither4x4_(_e38, _e40);
                val = _e41;
            }
        } else {
            let _e42 = global.matrixMode;
            if (_e42 == 2i) {
                {
                    let _e45 = loc;
                    let _e46 = inputPixelColor;
                    let _e47 = luma_1(_e46);
                    let _e48 = dither8x8_(_e45, _e47);
                    val = _e48;
                }
            }
        }
    }
    let _e49 = inputPixelColor;
    let _e50 = val;
    let _e51 = val;
    let _e52 = val;
    inputPixelColor = (_e49 * vec4<f32>(_e50, _e51, _e52, 1f));
    let _e56 = val;
    let _e57 = val;
    let _e58 = val;
    let _e59 = inputPixelColor;
    let _e62 = inputPixelColor;
    let _e63 = global.colorize;
    isf_FragColor = mix(vec4<f32>(_e56, _e57, _e58, _e59.w), _e62, vec4(_e63));
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e25 = isf_FragColor;
    return FragmentOutput(_e25);
}
