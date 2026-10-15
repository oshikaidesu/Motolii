// Dither-Bayer: ISF-Files' Dither-Bayer.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; Hugh Kennedy, adapted by David Lublin), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@colorize:colorize,0,1,0,0.001
// --track@matrixMode:matrixMode,0,1,2,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    colorize: f32,
    matrixMode: i32,
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
var<private> gl_FragCoord_1: vec4<f32>;

fn luma(color: vec3<f32>) -> f32 {
    var color_1: vec3<f32>;

    color_1 = color;
    let _e16 = color_1;
    let _e18 = color_1;
    let _e21 = color_1;
    return (((_e16.x + _e18.y) + _e21.z) / 3f);
}

fn luma_1(color_2: vec4<f32>) -> f32 {
    var color_3: vec4<f32>;

    color_3 = color_2;
    let _e16 = color_3;
    let _e18 = color_3;
    let _e20 = color_3;
    let _e23 = color_3;
    return ((_e16.w * ((_e18.x + _e20.y) + _e23.z)) / 3f);
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
    let _e18 = position_1;
    x = i32((_e18.x - (floor((_e18.x / 8f)) * 8f)));
    let _e27 = position_1;
    y = i32((_e27.y - (floor((_e27.y / 8f)) * 8f)));
    let _e36 = x;
    let _e37 = y;
    index = (_e36 + (_e37 * 8i));
    let _e44 = x;
    if (_e44 < 8i) {
        {
            let _e47 = index;
            if (_e47 == 0i) {
                limit = 0.015625f;
            }
            let _e51 = index;
            if (_e51 == 1i) {
                limit = 0.515625f;
            }
            let _e55 = index;
            if (_e55 == 2i) {
                limit = 0.140625f;
            }
            let _e59 = index;
            if (_e59 == 3i) {
                limit = 0.640625f;
            }
            let _e63 = index;
            if (_e63 == 4i) {
                limit = 0.046875f;
            }
            let _e67 = index;
            if (_e67 == 5i) {
                limit = 0.546875f;
            }
            let _e71 = index;
            if (_e71 == 6i) {
                limit = 0.171875f;
            }
            let _e75 = index;
            if (_e75 == 7i) {
                limit = 0.671875f;
            }
            let _e79 = index;
            if (_e79 == 8i) {
                limit = 0.765625f;
            }
            let _e83 = index;
            if (_e83 == 9i) {
                limit = 0.265625f;
            }
            let _e87 = index;
            if (_e87 == 10i) {
                limit = 0.890625f;
            }
            let _e91 = index;
            if (_e91 == 11i) {
                limit = 0.390625f;
            }
            let _e95 = index;
            if (_e95 == 12i) {
                limit = 0.796875f;
            }
            let _e99 = index;
            if (_e99 == 13i) {
                limit = 0.296875f;
            }
            let _e103 = index;
            if (_e103 == 14i) {
                limit = 0.921875f;
            }
            let _e107 = index;
            if (_e107 == 15i) {
                limit = 0.421875f;
            }
            let _e111 = index;
            if (_e111 == 16i) {
                limit = 0.203125f;
            }
            let _e115 = index;
            if (_e115 == 17i) {
                limit = 0.703125f;
            }
            let _e119 = index;
            if (_e119 == 18i) {
                limit = 0.078125f;
            }
            let _e123 = index;
            if (_e123 == 19i) {
                limit = 0.578125f;
            }
            let _e127 = index;
            if (_e127 == 20i) {
                limit = 0.234375f;
            }
            let _e131 = index;
            if (_e131 == 21i) {
                limit = 0.734375f;
            }
            let _e135 = index;
            if (_e135 == 22i) {
                limit = 0.109375f;
            }
            let _e139 = index;
            if (_e139 == 23i) {
                limit = 0.609375f;
            }
            let _e143 = index;
            if (_e143 == 24i) {
                limit = 0.953125f;
            }
            let _e147 = index;
            if (_e147 == 25i) {
                limit = 0.453125f;
            }
            let _e151 = index;
            if (_e151 == 26i) {
                limit = 0.828125f;
            }
            let _e155 = index;
            if (_e155 == 27i) {
                limit = 0.328125f;
            }
            let _e159 = index;
            if (_e159 == 28i) {
                limit = 0.984375f;
            }
            let _e163 = index;
            if (_e163 == 29i) {
                limit = 0.484375f;
            }
            let _e167 = index;
            if (_e167 == 30i) {
                limit = 0.859375f;
            }
            let _e171 = index;
            if (_e171 == 31i) {
                limit = 0.359375f;
            }
            let _e175 = index;
            if (_e175 == 32i) {
                limit = 0.0625f;
            }
            let _e179 = index;
            if (_e179 == 33i) {
                limit = 0.5625f;
            }
            let _e183 = index;
            if (_e183 == 34i) {
                limit = 0.1875f;
            }
            let _e187 = index;
            if (_e187 == 35i) {
                limit = 0.6875f;
            }
            let _e191 = index;
            if (_e191 == 36i) {
                limit = 0.03125f;
            }
            let _e195 = index;
            if (_e195 == 37i) {
                limit = 0.53125f;
            }
            let _e199 = index;
            if (_e199 == 38i) {
                limit = 0.15625f;
            }
            let _e203 = index;
            if (_e203 == 39i) {
                limit = 0.65625f;
            }
            let _e207 = index;
            if (_e207 == 40i) {
                limit = 0.8125f;
            }
            let _e211 = index;
            if (_e211 == 41i) {
                limit = 0.3125f;
            }
            let _e215 = index;
            if (_e215 == 42i) {
                limit = 0.9375f;
            }
            let _e219 = index;
            if (_e219 == 43i) {
                limit = 0.4375f;
            }
            let _e223 = index;
            if (_e223 == 44i) {
                limit = 0.78125f;
            }
            let _e227 = index;
            if (_e227 == 45i) {
                limit = 0.28125f;
            }
            let _e231 = index;
            if (_e231 == 46i) {
                limit = 0.90625f;
            }
            let _e235 = index;
            if (_e235 == 47i) {
                limit = 0.40625f;
            }
            let _e239 = index;
            if (_e239 == 48i) {
                limit = 0.25f;
            }
            let _e243 = index;
            if (_e243 == 49i) {
                limit = 0.75f;
            }
            let _e247 = index;
            if (_e247 == 50i) {
                limit = 0.125f;
            }
            let _e251 = index;
            if (_e251 == 51i) {
                limit = 0.625f;
            }
            let _e255 = index;
            if (_e255 == 52i) {
                limit = 0.21875f;
            }
            let _e259 = index;
            if (_e259 == 53i) {
                limit = 0.71875f;
            }
            let _e263 = index;
            if (_e263 == 54i) {
                limit = 0.09375f;
            }
            let _e267 = index;
            if (_e267 == 55i) {
                limit = 0.59375f;
            }
            let _e271 = index;
            if (_e271 == 56i) {
                limit = 1f;
            }
            let _e275 = index;
            if (_e275 == 57i) {
                limit = 0.5f;
            }
            let _e279 = index;
            if (_e279 == 58i) {
                limit = 0.875f;
            }
            let _e283 = index;
            if (_e283 == 59i) {
                limit = 0.375f;
            }
            let _e287 = index;
            if (_e287 == 60i) {
                limit = 0.96875f;
            }
            let _e291 = index;
            if (_e291 == 61i) {
                limit = 0.46875f;
            }
            let _e295 = index;
            if (_e295 == 62i) {
                limit = 0.84375f;
            }
            let _e299 = index;
            if (_e299 == 63i) {
                limit = 0.34375f;
            }
        }
    }
    let _e303 = brightness_1;
    let _e304 = limit;
    if (_e303 < _e304) {
        local = 0f;
    } else {
        local = 1f;
    }
    let _e309 = local;
    return _e309;
}

fn dither8x8_1(position_2: vec2<f32>, color_4: vec3<f32>) -> vec3<f32> {
    var position_3: vec2<f32>;
    var color_5: vec3<f32>;

    position_3 = position_2;
    color_5 = color_4;
    let _e18 = color_5;
    let _e19 = position_3;
    let _e20 = color_5;
    let _e21 = luma(_e20);
    let _e22 = dither8x8_(_e19, _e21);
    return (_e18 * _e22);
}

fn dither8x8_2(position_4: vec2<f32>, color_6: vec4<f32>) -> vec4<f32> {
    var position_5: vec2<f32>;
    var color_7: vec4<f32>;

    position_5 = position_4;
    color_7 = color_6;
    let _e18 = color_7;
    let _e20 = position_5;
    let _e21 = color_7;
    let _e22 = luma_1(_e21);
    let _e23 = dither8x8_(_e20, _e22);
    let _e24 = (_e18.xyz * _e23);
    return vec4<f32>(_e24.x, _e24.y, _e24.z, 1f);
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
    let _e18 = position_7;
    x_1 = i32((_e18.x - (floor((_e18.x / 4f)) * 4f)));
    let _e27 = position_7;
    y_1 = i32((_e27.y - (floor((_e27.y / 4f)) * 4f)));
    let _e36 = x_1;
    let _e37 = y_1;
    index_1 = (_e36 + (_e37 * 4i));
    let _e44 = x_1;
    if (_e44 < 8i) {
        {
            let _e47 = index_1;
            if (_e47 == 0i) {
                limit_1 = 0.0625f;
            }
            let _e51 = index_1;
            if (_e51 == 1i) {
                limit_1 = 0.5625f;
            }
            let _e55 = index_1;
            if (_e55 == 2i) {
                limit_1 = 0.1875f;
            }
            let _e59 = index_1;
            if (_e59 == 3i) {
                limit_1 = 0.6875f;
            }
            let _e63 = index_1;
            if (_e63 == 4i) {
                limit_1 = 0.8125f;
            }
            let _e67 = index_1;
            if (_e67 == 5i) {
                limit_1 = 0.3125f;
            }
            let _e71 = index_1;
            if (_e71 == 6i) {
                limit_1 = 0.9375f;
            }
            let _e75 = index_1;
            if (_e75 == 7i) {
                limit_1 = 0.4375f;
            }
            let _e79 = index_1;
            if (_e79 == 8i) {
                limit_1 = 0.25f;
            }
            let _e83 = index_1;
            if (_e83 == 9i) {
                limit_1 = 0.75f;
            }
            let _e87 = index_1;
            if (_e87 == 10i) {
                limit_1 = 0.125f;
            }
            let _e91 = index_1;
            if (_e91 == 11i) {
                limit_1 = 0.625f;
            }
            let _e95 = index_1;
            if (_e95 == 12i) {
                limit_1 = 1f;
            }
            let _e99 = index_1;
            if (_e99 == 13i) {
                limit_1 = 0.5f;
            }
            let _e103 = index_1;
            if (_e103 == 14i) {
                limit_1 = 0.875f;
            }
            let _e107 = index_1;
            if (_e107 == 15i) {
                limit_1 = 0.375f;
            }
        }
    }
    let _e111 = brightness_3;
    let _e112 = limit_1;
    if (_e111 < _e112) {
        local_1 = 0f;
    } else {
        local_1 = 1f;
    }
    let _e117 = local_1;
    return _e117;
}

fn dither4x4_1(position_8: vec2<f32>, color_8: vec3<f32>) -> vec3<f32> {
    var position_9: vec2<f32>;
    var color_9: vec3<f32>;

    position_9 = position_8;
    color_9 = color_8;
    let _e18 = color_9;
    let _e19 = position_9;
    let _e20 = color_9;
    let _e21 = luma(_e20);
    let _e22 = dither4x4_(_e19, _e21);
    return (_e18 * _e22);
}

fn dither4x4_2(position_10: vec2<f32>, color_10: vec4<f32>) -> vec4<f32> {
    var position_11: vec2<f32>;
    var color_11: vec4<f32>;

    position_11 = position_10;
    color_11 = color_10;
    let _e18 = color_11;
    let _e20 = position_11;
    let _e21 = color_11;
    let _e22 = luma_1(_e21);
    let _e23 = dither4x4_(_e20, _e22);
    let _e24 = (_e18.xyz * _e23);
    return vec4<f32>(_e24.x, _e24.y, _e24.z, 1f);
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
    let _e18 = position_13;
    x_2 = i32((_e18.x - (floor((_e18.x / 2f)) * 2f)));
    let _e27 = position_13;
    y_2 = i32((_e27.y - (floor((_e27.y / 2f)) * 2f)));
    let _e36 = x_2;
    let _e37 = y_2;
    index_2 = (_e36 + (_e37 * 2i));
    let _e44 = x_2;
    if (_e44 < 8i) {
        {
            let _e47 = index_2;
            if (_e47 == 0i) {
                limit_2 = 0.25f;
            }
            let _e51 = index_2;
            if (_e51 == 1i) {
                limit_2 = 0.75f;
            }
            let _e55 = index_2;
            if (_e55 == 2i) {
                limit_2 = 1f;
            }
            let _e59 = index_2;
            if (_e59 == 3i) {
                limit_2 = 0.5f;
            }
        }
    }
    let _e63 = brightness_5;
    let _e64 = limit_2;
    if (_e63 < _e64) {
        local_2 = 0f;
    } else {
        local_2 = 1f;
    }
    let _e69 = local_2;
    return _e69;
}

fn dither2x2_1(position_14: vec2<f32>, color_12: vec3<f32>) -> vec3<f32> {
    var position_15: vec2<f32>;
    var color_13: vec3<f32>;

    position_15 = position_14;
    color_13 = color_12;
    let _e18 = color_13;
    let _e19 = position_15;
    let _e20 = color_13;
    let _e21 = luma(_e20);
    let _e22 = dither2x2_(_e19, _e21);
    return (_e18 * _e22);
}

fn dither2x2_2(position_16: vec2<f32>, color_14: vec4<f32>) -> vec4<f32> {
    var position_17: vec2<f32>;
    var color_15: vec4<f32>;

    position_17 = position_16;
    color_15 = color_14;
    let _e18 = color_15;
    let _e20 = position_17;
    let _e21 = color_15;
    let _e22 = luma_1(_e21);
    let _e23 = dither2x2_(_e20, _e22);
    let _e24 = (_e18.xyz * _e23);
    return vec4<f32>(_e24.x, _e24.y, _e24.z, 1f);
}

fn main_1() {
    var inputPixelColor: vec4<f32> = vec4(0f);
    var loc: vec2<f32>;
    var val: f32 = 1f;

    let _e18 = gl_FragCoord_1;
    loc = _e18.xy;
    let _e21 = isf_FragNormCoord_1;
    let _e23 = textureSampleLevel(isf_picture, isf_sampler, _e21, 0f);
    inputPixelColor = _e23;
    let _e26 = global.matrixMode;
    if (_e26 == 0i) {
        {
            let _e29 = loc;
            let _e30 = inputPixelColor;
            let _e31 = luma_1(_e30);
            let _e32 = dither2x2_(_e29, _e31);
            val = _e32;
        }
    } else {
        let _e33 = global.matrixMode;
        if (_e33 == 1i) {
            {
                let _e36 = loc;
                let _e37 = inputPixelColor;
                let _e38 = luma_1(_e37);
                let _e39 = dither4x4_(_e36, _e38);
                val = _e39;
            }
        } else {
            let _e40 = global.matrixMode;
            if (_e40 == 2i) {
                {
                    let _e43 = loc;
                    let _e44 = inputPixelColor;
                    let _e45 = luma_1(_e44);
                    let _e46 = dither8x8_(_e43, _e45);
                    val = _e46;
                }
            }
        }
    }
    let _e47 = inputPixelColor;
    let _e48 = val;
    let _e49 = val;
    let _e50 = val;
    inputPixelColor = (_e47 * vec4<f32>(_e48, _e49, _e50, 1f));
    let _e54 = val;
    let _e55 = val;
    let _e56 = val;
    let _e57 = inputPixelColor;
    let _e60 = inputPixelColor;
    let _e61 = global.colorize;
    isf_FragColor = mix(vec4<f32>(_e54, _e55, _e56, _e57.w), _e60, vec4(_e61));
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e23 = isf_FragColor;
    return FragmentOutput(_e23);
}
