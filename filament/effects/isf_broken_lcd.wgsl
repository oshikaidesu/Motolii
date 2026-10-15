// Broken LCD: ISF-Files' Broken LCD.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@flickerLevel:flickerLevel,0,1,0,0.001
// --track@patternStyle:Pattern Style,0,1,0,1
// --track@glitchBrightness:Glitch Thickness,0,4,3,0.001
// --track@glitchBrightnessCurve:Glitch Shape,1,4,2,0.001
// --track@glitchRadius:Glitch Spread,0,1,0.5,0.001
// --track@glitchSmoothness:Glitch Edge Smoothness,0,1,0.5,0.001
// --track@glitchScale:Glitch Size,0,100,4.1013007164001465,0.001
// --track@noiseSeed:Glitch Seed,0,10,5.086348533630371,0.001
// --track@patternSeed:Pattern Seed,0,10,2.3372085094451904,0.001
// --track@tintBrightness:tintBrightness,0,4,4,0.001
// --track@tintRadius:tintRadius,0,1,0.22620739042758942,0.001
// --track@tintBrightnessCurve:tintBrightnessCurve,1,4,1.2606533765792847,0.001
// --track@tintSeed:tintSeed,0,20,0,0.001
// --track@tintColor1_x:tintColor1 r,0,1,0.9019607901573181,0.001
// --track@tintColor1_y:tintColor1 g,0,1,0.250980406999588,0.001
// --track@tintColor1_z:tintColor1 b,0,1,0.250980406999588,0.001
// --track@tintColor1_w:tintColor1 a,0,1,1,0.001
// --track@tintColor2_x:tintColor2 r,0,1,0.250980406999588,0.001
// --track@tintColor2_y:tintColor2 g,0,1,0.501960813999176,0.001
// --track@tintColor2_z:tintColor2 b,0,1,0.9019607901573181,0.001
// --track@tintColor2_w:tintColor2 a,0,1,1,0.001
// --track@tintScale:tintScale,0,100,4,0.001
// --track@rowGlitchLevel:rowGlitchLevel,0,1,0.009999999776482582,0.001
// --track@rowFlickerLevel:rowFlickerLevel,0,1,0,0.001
// --track@rowGlitchSeed:rowGlitchSeed,0,1,0.3409999907016754,0.001
// --check@hardEdge_on:Hard Edge,1
// --track@alphaMode:Alpha Mode,0,1,0,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    flickerLevel: f32,
    patternStyle: i32,
    glitchBrightness: f32,
    glitchBrightnessCurve: f32,
    glitchRadius: f32,
    glitchSmoothness: f32,
    glitchScale: f32,
    noiseSeed: f32,
    patternSeed: f32,
    tintBrightness: f32,
    tintRadius: f32,
    tintBrightnessCurve: f32,
    tintSeed: f32,
    tintColor1_: vec4<f32>,
    tintColor2_: vec4<f32>,
    tintScale: f32,
    rowGlitchLevel: f32,
    rowFlickerLevel: f32,
    rowGlitchSeed: f32,
    hardEdge_on: i32,
    alphaMode: i32,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

const pi: f32 = 3.1415927f;

@group(0) @binding(0) 
var isf_picture: texture_2d<f32>;
@group(0) @binding(1) 
var isf_sampler: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;

fn rotatePoint(pt: vec2<f32>, angle: f32, center: vec2<f32>) -> vec2<f32> {
    var pt_1: vec2<f32>;
    var angle_1: f32;
    var center_1: vec2<f32>;
    var returnMe: vec2<f32>;
    var s: f32;
    var c: f32;
    var xnew: f32;
    var ynew: f32;

    pt_1 = pt;
    angle_1 = angle;
    center_1 = center;
    let _e60 = angle_1;
    s = sin((_e60 * pi));
    let _e64 = angle_1;
    c = cos((_e64 * pi));
    let _e68 = pt_1;
    returnMe = _e68;
    let _e70 = returnMe;
    let _e72 = center_1;
    returnMe.x = (_e70.x - _e72.x);
    let _e76 = returnMe;
    let _e78 = center_1;
    returnMe.y = (_e76.y - _e78.y);
    let _e81 = returnMe;
    let _e83 = c;
    let _e85 = returnMe;
    let _e87 = s;
    xnew = ((_e81.x * _e83) - (_e85.y * _e87));
    let _e91 = returnMe;
    let _e93 = s;
    let _e95 = returnMe;
    let _e97 = c;
    ynew = ((_e91.x * _e93) + (_e95.y * _e97));
    let _e102 = xnew;
    let _e103 = center_1;
    returnMe.x = (_e102 + _e103.x);
    let _e107 = ynew;
    let _e108 = center_1;
    returnMe.y = (_e107 + _e108.y);
    let _e111 = returnMe;
    return _e111;
}

fn sign_(p1_: vec2<f32>, p2_: vec2<f32>, p3_: vec2<f32>) -> f32 {
    var p1_1: vec2<f32>;
    var p2_1: vec2<f32>;
    var p3_1: vec2<f32>;

    p1_1 = p1_;
    p2_1 = p2_;
    p3_1 = p3_;
    let _e59 = p1_1;
    let _e61 = p3_1;
    let _e64 = p2_1;
    let _e66 = p3_1;
    let _e70 = p2_1;
    let _e72 = p3_1;
    let _e75 = p1_1;
    let _e77 = p3_1;
    return (((_e59.x - _e61.x) * (_e64.y - _e66.y)) - ((_e70.x - _e72.x) * (_e75.y - _e77.y)));
}

fn PointInTriangle(pt_2: vec2<f32>, v1_: vec2<f32>, v2_: vec2<f32>, v3_: vec2<f32>) -> bool {
    var pt_3: vec2<f32>;
    var v1_1: vec2<f32>;
    var v2_1: vec2<f32>;
    var v3_1: vec2<f32>;
    var b1_: bool;
    var b2_: bool;
    var b3_: bool;

    pt_3 = pt_2;
    v1_1 = v1_;
    v2_1 = v2_;
    v3_1 = v3_;
    let _e64 = pt_3;
    let _e65 = v1_1;
    let _e66 = v2_1;
    let _e67 = sign_(_e64, _e65, _e66);
    b1_ = (_e67 < 0f);
    let _e70 = pt_3;
    let _e71 = v2_1;
    let _e72 = v3_1;
    let _e73 = sign_(_e70, _e71, _e72);
    b2_ = (_e73 < 0f);
    let _e76 = pt_3;
    let _e77 = v3_1;
    let _e78 = v1_1;
    let _e79 = sign_(_e76, _e77, _e78);
    b3_ = (_e79 < 0f);
    let _e82 = b1_;
    let _e83 = b2_;
    let _e85 = b2_;
    let _e86 = b3_;
    return ((_e82 == _e83) && (_e85 == _e86));
}

fn RotatedPointInTriangle(pt_4: vec2<f32>, v1_2: vec2<f32>, v2_2: vec2<f32>, v3_2: vec2<f32>, center_2: vec2<f32>) -> bool {
    var pt_5: vec2<f32>;
    var v1_3: vec2<f32>;
    var v2_3: vec2<f32>;
    var v3_3: vec2<f32>;
    var center_3: vec2<f32>;
    var b1_1: bool;
    var b2_1: bool;
    var b3_1: bool;
    var v1r: vec2<f32>;
    var v2r: vec2<f32>;
    var v3r: vec2<f32>;

    pt_5 = pt_4;
    v1_3 = v1_2;
    v2_3 = v2_2;
    v3_3 = v3_2;
    center_3 = center_2;
    let _e66 = v1_3;
    v1r = _e66;
    let _e68 = v2_3;
    v2r = _e68;
    let _e70 = v3_3;
    v3r = _e70;
    let _e72 = pt_5;
    let _e73 = v1r;
    let _e74 = v2r;
    let _e75 = sign_(_e72, _e73, _e74);
    b1_1 = (_e75 < 0f);
    let _e78 = pt_5;
    let _e79 = v2r;
    let _e80 = v3r;
    let _e81 = sign_(_e78, _e79, _e80);
    b2_1 = (_e81 < 0f);
    let _e84 = pt_5;
    let _e85 = v3r;
    let _e86 = v1r;
    let _e87 = sign_(_e84, _e85, _e86);
    b3_1 = (_e87 < 0f);
    let _e90 = b1_1;
    let _e91 = b2_1;
    let _e93 = b2_1;
    let _e94 = b3_1;
    return ((_e90 == _e91) && (_e93 == _e94));
}

fn isPointInShape(pt_6: vec2<f32>, shape: i32, shapeCoordinates: vec4<f32>) -> f32 {
    var pt_7: vec2<f32>;
    var shape_1: i32;
    var shapeCoordinates_1: vec4<f32>;
    var returnMe_1: f32 = 0f;

    pt_7 = pt_6;
    shape_1 = shape;
    shapeCoordinates_1 = shapeCoordinates;
    let _e61 = shape_1;
    if (_e61 == 0i) {
        {
            let _e64 = pt_7;
            let _e65 = shapeCoordinates_1;
            let _e67 = shapeCoordinates_1;
            let _e70 = shapeCoordinates_1;
            let _e74 = shapeCoordinates_1;
            let _e76 = shapeCoordinates_1;
            let _e81 = shapeCoordinates_1;
            let _e83 = shapeCoordinates_1;
            let _e89 = RotatedPointInTriangle(_e64, _e65.xy, (_e67.xy + vec2<f32>(0f, _e70.w)), (_e74.xy + vec2<f32>(_e76.z, 0f)), (_e81.xy + (_e83.zw / vec2(2f))));
            if _e89 {
                {
                    returnMe_1 = 1f;
                    let _e91 = pt_7;
                    let _e93 = shapeCoordinates_1;
                    let _e96 = pt_7;
                    let _e98 = shapeCoordinates_1;
                    if ((_e91.x > _e93.x) && (_e96.x < _e98.x)) {
                        {
                            let _e102 = pt_7;
                            let _e104 = shapeCoordinates_1;
                            let _e107 = global.resolution;
                            returnMe_1 = clamp(((_e102.x - _e104.x) / _e107.x), 0f, 1f);
                            let _e113 = returnMe_1;
                            returnMe_1 = pow(_e113, 0.5f);
                        }
                    } else {
                        let _e116 = pt_7;
                        let _e118 = shapeCoordinates_1;
                        let _e120 = shapeCoordinates_1;
                        let _e124 = pt_7;
                        let _e126 = shapeCoordinates_1;
                        let _e128 = shapeCoordinates_1;
                        if ((_e116.x > (_e118.x + _e120.z)) && (_e124.x < (_e126.x + _e128.z))) {
                            {
                                let _e133 = shapeCoordinates_1;
                                let _e135 = shapeCoordinates_1;
                                let _e138 = pt_7;
                                let _e141 = global.resolution;
                                returnMe_1 = clamp((((_e133.x + _e135.z) - _e138.x) / _e141.x), 0f, 1f);
                                let _e147 = returnMe_1;
                                returnMe_1 = pow(_e147, 0.5f);
                            }
                        }
                    }
                }
            } else {
                let _e150 = pt_7;
                let _e151 = shapeCoordinates_1;
                let _e153 = shapeCoordinates_1;
                let _e156 = shapeCoordinates_1;
                let _e159 = shapeCoordinates_1;
                let _e163 = shapeCoordinates_1;
                let _e165 = shapeCoordinates_1;
                let _e170 = shapeCoordinates_1;
                let _e172 = shapeCoordinates_1;
                let _e178 = RotatedPointInTriangle(_e150, (_e151.xy + _e153.zw), (_e156.xy + vec2<f32>(0f, _e159.w)), (_e163.xy + vec2<f32>(_e165.z, 0f)), (_e170.xy + (_e172.zw / vec2(2f))));
                if _e178 {
                    {
                        returnMe_1 = 1f;
                        let _e180 = pt_7;
                        let _e182 = shapeCoordinates_1;
                        let _e185 = pt_7;
                        let _e187 = shapeCoordinates_1;
                        if ((_e180.x > _e182.x) && (_e185.x < _e187.x)) {
                            {
                                let _e191 = pt_7;
                                let _e193 = shapeCoordinates_1;
                                let _e196 = global.resolution;
                                returnMe_1 = clamp(((_e191.x - _e193.x) / _e196.x), 0f, 1f);
                                let _e202 = returnMe_1;
                                returnMe_1 = pow(_e202, 0.5f);
                            }
                        } else {
                            let _e205 = pt_7;
                            let _e207 = shapeCoordinates_1;
                            let _e209 = shapeCoordinates_1;
                            let _e213 = pt_7;
                            let _e215 = shapeCoordinates_1;
                            let _e217 = shapeCoordinates_1;
                            if ((_e205.x > (_e207.x + _e209.z)) && (_e213.x < (_e215.x + _e217.z))) {
                                {
                                    let _e222 = shapeCoordinates_1;
                                    let _e224 = shapeCoordinates_1;
                                    let _e227 = pt_7;
                                    let _e230 = global.resolution;
                                    returnMe_1 = clamp((((_e222.x + _e224.z) - _e227.x) / _e230.x), 0f, 1f);
                                    let _e236 = returnMe_1;
                                    returnMe_1 = pow(_e236, 0.5f);
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        let _e239 = shape_1;
        if (_e239 == 1i) {
            {
                let _e242 = pt_7;
                let _e243 = shapeCoordinates_1;
                let _e245 = shapeCoordinates_1;
                let _e247 = shapeCoordinates_1;
                let _e251 = shapeCoordinates_1;
                let _e255 = shapeCoordinates_1;
                let _e257 = shapeCoordinates_1;
                let _e262 = shapeCoordinates_1;
                let _e264 = shapeCoordinates_1;
                let _e270 = RotatedPointInTriangle(_e242, _e243.xy, (_e245.xy + vec2<f32>((_e247.z / 2f), _e251.w)), (_e255.xy + vec2<f32>(_e257.z, 0f)), (_e262.xy + (_e264.zw / vec2(2f))));
                if _e270 {
                    {
                        returnMe_1 = 1f;
                    }
                }
            }
        } else {
            let _e272 = shape_1;
            if (_e272 == 2i) {
                {
                    let _e275 = pt_7;
                    let _e276 = shapeCoordinates_1;
                    let _e278 = shapeCoordinates_1;
                    returnMe_1 = distance(_e275, vec2<f32>((_e276.xy + (_e278.zw / vec2(2f)))));
                    let _e286 = returnMe_1;
                    let _e287 = shapeCoordinates_1;
                    let _e289 = shapeCoordinates_1;
                    if (_e286 < (min(_e287.z, _e289.w) / 2f)) {
                        {
                            returnMe_1 = 1f;
                        }
                    } else {
                        {
                            returnMe_1 = 0f;
                        }
                    }
                }
            } else {
                let _e297 = shape_1;
                if (_e297 == 3i) {
                    {
                        let _e300 = pt_7;
                        let _e301 = shapeCoordinates_1;
                        let _e304 = shapeCoordinates_1;
                        let _e310 = shapeCoordinates_1;
                        let _e312 = shapeCoordinates_1;
                        let _e316 = shapeCoordinates_1;
                        let _e320 = shapeCoordinates_1;
                        let _e322 = shapeCoordinates_1;
                        let _e324 = shapeCoordinates_1;
                        let _e330 = shapeCoordinates_1;
                        let _e332 = shapeCoordinates_1;
                        let _e338 = RotatedPointInTriangle(_e300, (_e301.xy + vec2<f32>(0f, (_e304.w / 2f))), (_e310.xy + vec2<f32>((_e312.z / 2f), _e316.w)), (_e320.xy + vec2<f32>(_e322.z, (_e324.w / 2f))), (_e330.xy + (_e332.zw / vec2(2f))));
                        if _e338 {
                            {
                                returnMe_1 = 1f;
                            }
                        } else {
                            let _e340 = pt_7;
                            let _e341 = shapeCoordinates_1;
                            let _e344 = shapeCoordinates_1;
                            let _e350 = shapeCoordinates_1;
                            let _e352 = shapeCoordinates_1;
                            let _e359 = shapeCoordinates_1;
                            let _e361 = shapeCoordinates_1;
                            let _e363 = shapeCoordinates_1;
                            let _e369 = shapeCoordinates_1;
                            let _e371 = shapeCoordinates_1;
                            let _e377 = RotatedPointInTriangle(_e340, (_e341.xy + vec2<f32>(0f, (_e344.w / 2f))), (_e350.xy + vec2<f32>((_e352.z / 2f), 0f)), (_e359.xy + vec2<f32>(_e361.z, (_e363.w / 2f))), (_e369.xy + (_e371.zw / vec2(2f))));
                            if _e377 {
                                {
                                    returnMe_1 = 1f;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let _e379 = returnMe_1;
    return _e379;
}

fn hash(p: vec2<f32>, seed: f32) -> f32 {
    var p_1: vec2<f32>;
    var seed_1: f32;

    p_1 = p;
    seed_1 = seed;
    let _e58 = seed_1;
    let _e61 = p_1;
    p_1 = (50f * fract(((vec2((_e58 * 1.17921f)) + (_e61 * 0.3183099f)) + vec2<f32>(0.71f, 0.113f))));
    let _e75 = p_1;
    let _e77 = p_1;
    let _e80 = p_1;
    let _e82 = p_1;
    return (-1f + (2f * fract(((_e75.x * _e77.y) * (_e80.x + _e82.y)))));
}

fn noise(p_2: vec2<f32>, seed_2: f32) -> f32 {
    var p_3: vec2<f32>;
    var seed_3: f32;
    var i: vec2<f32>;
    var f: vec2<f32>;
    var u: vec2<f32>;
    var v1_4: f32;
    var v2_4: f32;

    p_3 = p_2;
    seed_3 = seed_2;
    let _e57 = p_3;
    i = floor(_e57);
    let _e60 = p_3;
    f = fract(_e60);
    let _e63 = f;
    let _e64 = f;
    let _e68 = f;
    u = ((_e63 * _e64) * (vec2(3f) - (2f * _e68)));
    let _e74 = i;
    let _e79 = seed_3;
    let _e81 = hash((_e74 + vec2<f32>(0f, 0f)), floor(_e79));
    let _e82 = i;
    let _e87 = seed_3;
    let _e89 = hash((_e82 + vec2<f32>(1f, 0f)), floor(_e87));
    let _e90 = u;
    let _e93 = i;
    let _e98 = seed_3;
    let _e100 = hash((_e93 + vec2<f32>(0f, 1f)), floor(_e98));
    let _e101 = i;
    let _e106 = seed_3;
    let _e108 = hash((_e101 + vec2<f32>(1f, 1f)), floor(_e106));
    let _e109 = u;
    let _e112 = u;
    v1_4 = mix(mix(_e81, _e89, _e90.x), mix(_e100, _e108, _e109.x), _e112.y);
    let _e116 = i;
    let _e121 = seed_3;
    let _e123 = hash((_e116 + vec2<f32>(0f, 0f)), ceil(_e121));
    let _e124 = i;
    let _e129 = seed_3;
    let _e131 = hash((_e124 + vec2<f32>(1f, 0f)), ceil(_e129));
    let _e132 = u;
    let _e135 = i;
    let _e140 = seed_3;
    let _e142 = hash((_e135 + vec2<f32>(0f, 1f)), ceil(_e140));
    let _e143 = i;
    let _e148 = seed_3;
    let _e150 = hash((_e143 + vec2<f32>(1f, 1f)), ceil(_e148));
    let _e151 = u;
    let _e154 = u;
    v2_4 = mix(mix(_e123, _e131, _e132.x), mix(_e142, _e150, _e151.x), _e154.y);
    let _e158 = v1_4;
    let _e159 = v2_4;
    let _e160 = seed_3;
    return mix(_e158, _e159, fract(_e160));
}

fn map(n: f32, i1_: f32, i2_: f32, o1_: f32, o2_: f32) -> f32 {
    var n_1: f32;
    var i1_1: f32;
    var i2_1: f32;
    var o1_1: f32;
    var o2_1: f32;

    n_1 = n;
    i1_1 = i1_;
    i2_1 = i2_;
    o1_1 = o1_;
    o2_1 = o2_;
    let _e63 = o1_1;
    let _e64 = o2_1;
    let _e65 = o1_1;
    let _e67 = n_1;
    let _e68 = i1_1;
    let _e71 = i2_1;
    let _e72 = i1_1;
    return (_e63 + (((_e64 - _e65) * (_e67 - _e68)) / (_e71 - _e72)));
}

fn rand(co: vec2<f32>) -> f32 {
    var co_1: vec2<f32>;

    co_1 = co;
    let _e55 = co_1;
    return fract((sin(dot(_e55.xy, vec2<f32>(12.9898f, 78.233f))) * 43758.547f));
}

fn rand4_(co_2: vec4<f32>) -> vec4<f32> {
    var co_3: vec4<f32>;
    var returnMe_2: vec4<f32> = vec4(0f);

    co_3 = co_2;
    let _e59 = co_3;
    let _e61 = rand(_e59.xy);
    returnMe_2.x = _e61;
    let _e63 = co_3;
    let _e65 = rand(_e63.yz);
    returnMe_2.y = _e65;
    let _e67 = co_3;
    let _e69 = rand(_e67.zw);
    returnMe_2.z = _e69;
    let _e71 = co_3;
    let _e73 = rand(_e71.xz);
    returnMe_2.w = _e73;
    let _e74 = returnMe_2;
    return _e74;
}

fn patternForType(coord: vec2<f32>, pType: i32, seed_4: vec2<f32>) -> vec4<f32> {
    var coord_1: vec2<f32>;
    var pType_1: i32;
    var seed_5: vec2<f32>;
    var returnMe_3: vec4<f32> = vec4(0f);
    var pt_8: i32;
    var p_4: vec2<f32>;
    var uv: vec2<f32>;
    var f_1: f32 = 0f;
    var blockness: f32;
    var m: mat2x2<f32> = mat2x2<f32>(vec2<f32>(1.6f, 1.2f), vec2<f32>(-1.2f, 1.6f));
    var d: f32;
    var flickCoord: vec2<f32>;
    var local: f32;
    var flickRand: f32;
    var doFlick: bool = false;
    var modCoord: vec2<f32>;
    var local_1: f32;
    var local_2: f32;
    var local_3: f32;
    var local_4: vec2<f32>;
    var modCoord_1: vec2<f32>;
    var local_5: f32;
    var local_6: f32;
    var local_7: f32;
    var modCoord_2: vec2<f32>;
    var local_8: f32;
    var local_9: f32;
    var local_10: f32;
    var modCoord_3: vec2<f32>;
    var local_11: f32;
    var local_12: f32;
    var local_13: f32;
    var modCoord_4: vec2<f32>;
    var local_14: vec4<f32>;
    var local_15: vec4<f32>;
    var modCoord_5: vec2<f32>;

    coord_1 = coord;
    pType_1 = pType;
    seed_5 = seed_4;
    let _e62 = pType_1;
    pt_8 = _e62;
    let _e64 = pt_8;
    if (_e64 <= 2i) {
        {
            let _e68 = gl_FragCoord_1;
            let _e70 = global.resolution;
            p_4 = (_e68.xy / _e70);
            let _e73 = p_4;
            let _e74 = global.resolution;
            let _e76 = global.resolution;
            uv = (_e73 * vec2<f32>((_e74.x / _e76.y), 1f));
            let _e85 = global.glitchSmoothness;
            if (_e85 < 1f) {
                {
                    let _e88 = global.noiseSeed;
                    let _e89 = uv;
                    let _e91 = global.glitchScale;
                    let _e98 = global.glitchScale;
                    let _e105 = rand((_e88 * (floor((_e89 * (1f + (_e91 * 50f)))) / vec2((1f + (_e98 * 50f))))));
                    blockness = _e105;
                    let _e107 = blockness;
                    let _e108 = global.glitchSmoothness;
                    if (_e107 > _e108) {
                        let _e110 = uv;
                        let _e112 = global.glitchScale;
                        let _e119 = global.glitchScale;
                        uv = (floor((_e110 * (1f + (_e112 * 20f)))) / vec2((1f + (_e119 * 20f))));
                    }
                }
            }
            {
                let _e125 = uv;
                let _e126 = global.glitchScale;
                uv = (_e125 * _e126);
                let _e138 = uv;
                let _e139 = global.patternSeed;
                let _e140 = noise(_e138, _e139);
                f_1 = (0.5f * _e140);
                let _e142 = m;
                let _e143 = uv;
                uv = (_e142 * _e143);
                let _e145 = f_1;
                let _e147 = uv;
                let _e148 = global.patternSeed;
                let _e149 = noise(_e147, _e148);
                f_1 = (_e145 + (0.25f * _e149));
                let _e152 = m;
                let _e153 = uv;
                uv = (_e152 * _e153);
                let _e155 = f_1;
                let _e157 = uv;
                let _e158 = global.patternSeed;
                let _e159 = noise(_e157, _e158);
                f_1 = (_e155 + (0.125f * _e159));
                let _e162 = m;
                let _e163 = uv;
                uv = (_e162 * _e163);
                let _e165 = f_1;
                let _e167 = uv;
                let _e168 = global.patternSeed;
                let _e169 = noise(_e167, _e168);
                f_1 = (_e165 + (0.0625f * _e169));
                let _e172 = m;
                let _e173 = uv;
                uv = (_e172 * _e173);
            }
            let _e175 = f_1;
            f_1 = pow(_e175, 0.5f);
            let _e178 = isf_FragNormCoord_1;
            d = distance(_e178, vec2(0.5f));
            let _e183 = d;
            let _e184 = global.glitchRadius;
            if (_e183 > _e184) {
                {
                    let _e186 = f_1;
                    let _e187 = d;
                    let _e188 = global.glitchRadius;
                    f_1 = (_e186 + (_e187 - _e188));
                }
            }
            let _e191 = pt_8;
            if (_e191 == 0i) {
                {
                    let _e195 = f_1;
                    pt_8 = (5i + i32((min(_e195, 1f) * 3.99f)));
                }
            } else {
                let _e202 = pt_8;
                if (_e202 == 1i) {
                    {
                        let _e206 = f_1;
                        pt_8 = (3i + i32((min(_e206, 1f) * 3.99f)));
                    }
                } else {
                    let _e213 = pt_8;
                    if (_e213 == 2i) {
                        {
                            let _e217 = f_1;
                            pt_8 = (4i + i32((_e217 * 5.99f)));
                        }
                    }
                }
            }
        }
    }
    let _e222 = coord_1;
    flickCoord = _e222;
    let _e224 = pt_8;
    if (_e224 == 5i) {
        {
            let _e227 = coord_1;
            let _e229 = coord_1;
            let _e232 = global.patternSeed;
            let _e237 = global.patternSeed;
            flickCoord = vec2<f32>(_e227.x, (floor((_e229.y / (1f + _e232))) * (1f + _e237)));
        }
    } else {
        let _e241 = pt_8;
        if (_e241 == 6i) {
            {
                let _e244 = coord_1;
                let _e247 = global.patternSeed;
                let _e252 = global.patternSeed;
                let _e255 = coord_1;
                flickCoord = vec2<f32>((floor((_e244.x / (1f + _e247))) * (1f + _e252)), _e255.y);
            }
        } else {
            let _e258 = pt_8;
            if (_e258 > 6i) {
                {
                    let _e261 = coord_1;
                    let _e263 = global.patternSeed;
                    let _e269 = global.patternSeed;
                    flickCoord = (floor((_e261 / vec2((1f + _e263)))) * (1f + _e269));
                }
            }
        }
    }
    let _e272 = global.flickerLevel;
    if (_e272 > 0f) {
        let _e275 = global.time;
        let _e276 = flickCoord;
        let _e278 = global.patternSeed;
        let _e281 = rand(((_e275 * _e276) + vec2(_e278)));
        local = _e281;
    } else {
        local = 0f;
    }
    let _e284 = local;
    flickRand = _e284;
    let _e288 = flickRand;
    let _e289 = global.flickerLevel;
    if (_e288 < _e289) {
        {
            doFlick = true;
        }
    }
    let _e292 = pt_8;
    if (_e292 == 3i) {
        {
            returnMe_3 = vec4(0f);
        }
    } else {
        let _e297 = pt_8;
        if (_e297 == 4i) {
            {
                let _e300 = doFlick;
                if _e300 {
                    {
                        let _e301 = isf_FragNormCoord_1;
                        let _e303 = textureSampleLevel(isf_picture, isf_sampler, _e301, 0f);
                        returnMe_3 = _e303;
                    }
                } else {
                    {
                        let _e304 = global.resolution;
                        let _e305 = global.patternSeed;
                        let _e306 = coord_1;
                        let _e308 = rand((_e305 * _e306));
                        let _e309 = global.patternSeed;
                        let _e310 = coord_1;
                        let _e313 = rand((_e309 * _e310.yx));
                        modCoord = (_e304 * vec2<f32>(_e308, _e313));
                        let _e317 = modCoord;
                        let _e319 = global.patternSeed;
                        modCoord = (_e317 * max((1f + _e319), 1.00001f));
                        let _e325 = modCoord;
                        let _e327 = modCoord;
                        let _e329 = (_e325.x + _e327.y);
                        if ((_e329 - (floor((_e329 / 4f)) * 4f)) < 2f) {
                            local_1 = 1f;
                        } else {
                            local_1 = 0f;
                        }
                        let _e340 = local_1;
                        returnMe_3.x = _e340;
                        let _e342 = modCoord;
                        let _e344 = modCoord;
                        let _e346 = (_e342.x + _e344.y);
                        if ((_e346 - (floor((_e346 / 5f)) * 5f)) < 2f) {
                            local_2 = 1f;
                        } else {
                            local_2 = 0f;
                        }
                        let _e357 = local_2;
                        returnMe_3.y = _e357;
                        let _e359 = modCoord;
                        let _e361 = modCoord;
                        let _e363 = (_e359.x + _e361.y);
                        if ((_e363 - (floor((_e363 / 7f)) * 7f)) < 2f) {
                            local_3 = 1f;
                        } else {
                            local_3 = 0f;
                        }
                        let _e374 = local_3;
                        returnMe_3.z = _e374;
                        returnMe_3.w = 1f;
                    }
                }
            }
        } else {
            let _e377 = pt_8;
            if (_e377 == 5i) {
                {
                    let _e380 = global.patternSeed;
                    let _e381 = coord_1;
                    let _e384 = rand(vec2<f32>(_e380, _e381.y));
                    if (_e384 < 0.25f) {
                        let _e387 = coord_1;
                        local_4 = _e387;
                    } else {
                        let _e388 = coord_1;
                        local_4 = _e388.yx;
                    }
                    let _e391 = local_4;
                    modCoord_1 = _e391;
                    let _e393 = modCoord_1;
                    let _e395 = global.patternSeed;
                    modCoord_1 = (_e393 * (1f + _e395));
                    let _e398 = doFlick;
                    if _e398 {
                        {
                            let _e399 = modCoord_1;
                            let _e403 = global.resolution;
                            let _e406 = textureSampleLevel(isf_picture, isf_sampler, (vec2<f32>(_e399.x, 0f) / _e403), 0f);
                            let _e407 = modCoord_1;
                            let _e409 = global.resolution;
                            let _e412 = global.resolution;
                            let _e415 = textureSampleLevel(isf_picture, isf_sampler, (vec2<f32>(_e407.x, _e409.y) / _e412), 0f);
                            let _e416 = modCoord_1;
                            let _e418 = global.resolution;
                            returnMe_3 = mix(_e406, _e415, vec4((_e416.y / _e418.y)));
                        }
                    } else {
                        {
                            let _e424 = modCoord_1;
                            if ((_e424.x - (floor((_e424.x / 4f)) * 4f)) < 1f) {
                                local_5 = 1f;
                            } else {
                                local_5 = 0f;
                            }
                            let _e436 = local_5;
                            returnMe_3.x = _e436;
                            let _e438 = modCoord_1;
                            if ((_e438.x - (floor((_e438.x / 5f)) * 5f)) < 1f) {
                                local_6 = 1f;
                            } else {
                                local_6 = 0f;
                            }
                            let _e450 = local_6;
                            returnMe_3.y = _e450;
                            let _e452 = modCoord_1;
                            if ((_e452.x - (floor((_e452.x / 7f)) * 7f)) < 1f) {
                                local_7 = 1f;
                            } else {
                                local_7 = 0f;
                            }
                            let _e464 = local_7;
                            returnMe_3.z = _e464;
                            returnMe_3.w = 1f;
                        }
                    }
                }
            } else {
                let _e467 = pt_8;
                if (_e467 == 6i) {
                    {
                        let _e470 = coord_1;
                        modCoord_2 = _e470;
                        let _e473 = modCoord_2;
                        let _e476 = global.patternSeed;
                        modCoord_2.x = (_e473.x * (1f + _e476));
                        let _e479 = modCoord_2;
                        let _e481 = global.patternSeed;
                        modCoord_2 = (_e479 * (1f + _e481));
                        let _e484 = modCoord_2;
                        let _e485 = global.resolution;
                        modCoord_2 = (_e484 - (floor((_e484 / _e485)) * _e485));
                        let _e490 = doFlick;
                        if _e490 {
                            {
                                let _e491 = modCoord_2;
                                let _e495 = global.resolution;
                                let _e498 = textureSampleLevel(isf_picture, isf_sampler, (vec2<f32>(_e491.x, 0f) / _e495), 0f);
                                let _e499 = modCoord_2;
                                let _e501 = global.resolution;
                                let _e504 = global.resolution;
                                let _e507 = textureSampleLevel(isf_picture, isf_sampler, (vec2<f32>(_e499.x, _e501.y) / _e504), 0f);
                                let _e508 = modCoord_2;
                                let _e510 = global.resolution;
                                returnMe_3 = mix(_e498, _e507, vec4((_e508.y / _e510.y)));
                            }
                        } else {
                            {
                                let _e516 = modCoord_2;
                                if ((_e516.x - (floor((_e516.x / 4f)) * 4f)) < 1f) {
                                    local_8 = 1f;
                                } else {
                                    local_8 = 0f;
                                }
                                let _e528 = local_8;
                                returnMe_3.x = _e528;
                                let _e530 = modCoord_2;
                                if ((_e530.x - (floor((_e530.x / 5f)) * 5f)) < 1f) {
                                    local_9 = 1f;
                                } else {
                                    local_9 = 0f;
                                }
                                let _e542 = local_9;
                                returnMe_3.y = _e542;
                                let _e544 = modCoord_2;
                                if ((_e544.x - (floor((_e544.x / 7f)) * 7f)) < 1f) {
                                    local_10 = 1f;
                                } else {
                                    local_10 = 0f;
                                }
                                let _e556 = local_10;
                                returnMe_3.z = _e556;
                                returnMe_3.w = 1f;
                            }
                        }
                    }
                } else {
                    let _e559 = pt_8;
                    if (_e559 == 7i) {
                        {
                            let _e562 = coord_1;
                            modCoord_3 = _e562;
                            let _e565 = modCoord_3;
                            let _e568 = global.patternSeed;
                            modCoord_3.y = (_e565.y * (1f + _e568));
                            let _e571 = modCoord_3;
                            let _e572 = global.resolution;
                            modCoord_3 = (_e571 - (floor((_e571 / _e572)) * _e572));
                            let _e577 = doFlick;
                            if _e577 {
                                {
                                    let _e579 = modCoord_3;
                                    let _e582 = global.resolution;
                                    let _e585 = textureSampleLevel(isf_picture, isf_sampler, (vec2<f32>(0f, _e579.y) / _e582), 0f);
                                    let _e586 = global.resolution;
                                    let _e588 = modCoord_3;
                                    let _e591 = global.resolution;
                                    let _e594 = textureSampleLevel(isf_picture, isf_sampler, (vec2<f32>(_e586.x, _e588.y) / _e591), 0f);
                                    let _e595 = modCoord_3;
                                    let _e597 = global.resolution;
                                    returnMe_3 = mix(_e585, _e594, vec4((_e595.x / _e597.x)));
                                }
                            } else {
                                {
                                    let _e603 = modCoord_3;
                                    if ((_e603.y - (floor((_e603.y / 4f)) * 4f)) < 1f) {
                                        local_11 = 1f;
                                    } else {
                                        local_11 = 0f;
                                    }
                                    let _e615 = local_11;
                                    returnMe_3.x = _e615;
                                    let _e617 = modCoord_3;
                                    if ((_e617.y - (floor((_e617.y / 5f)) * 5f)) < 1f) {
                                        local_12 = 1f;
                                    } else {
                                        local_12 = 0f;
                                    }
                                    let _e629 = local_12;
                                    returnMe_3.y = _e629;
                                    let _e631 = modCoord_3;
                                    if ((_e631.y - (floor((_e631.y / 7f)) * 7f)) < 1f) {
                                        local_13 = 1f;
                                    } else {
                                        local_13 = 0f;
                                    }
                                    let _e643 = local_13;
                                    returnMe_3.z = _e643;
                                    returnMe_3.w = 1f;
                                }
                            }
                        }
                    } else {
                        let _e646 = pt_8;
                        if (_e646 == 8i) {
                            {
                                let _e649 = coord_1;
                                let _e651 = global.patternSeed;
                                let _e653 = (_e649 * (1f + _e651));
                                let _e655 = vec2(10f);
                                modCoord_4 = (_e653 - (floor((_e653 / _e655)) * _e655));
                                let _e661 = doFlick;
                                if _e661 {
                                    {
                                        let _e662 = modCoord_4;
                                        if (_e662.x < 5f) {
                                            let _e666 = modCoord_4;
                                            let _e670 = global.resolution;
                                            let _e673 = textureSampleLevel(isf_picture, isf_sampler, (vec2<f32>(_e666.x, 0f) / _e670), 0f);
                                            local_14 = _e673;
                                        } else {
                                            let _e674 = modCoord_4;
                                            let _e676 = global.resolution;
                                            let _e679 = global.resolution;
                                            let _e682 = textureSampleLevel(isf_picture, isf_sampler, (vec2<f32>(_e674.x, _e676.y) / _e679), 0f);
                                            local_14 = _e682;
                                        }
                                        let _e684 = local_14;
                                        returnMe_3 = _e684;
                                    }
                                } else {
                                    {
                                        let _e685 = modCoord_4;
                                        if (_e685.x < 5f) {
                                            local_15 = vec4(1f);
                                        } else {
                                            local_15 = vec4<f32>(0f, 0f, 0f, 1f);
                                        }
                                        let _e697 = local_15;
                                        returnMe_3 = _e697;
                                    }
                                }
                            }
                        } else {
                            {
                                let _e698 = coord_1;
                                let _e700 = global.patternSeed;
                                let _e702 = (_e698 * (1f + _e700));
                                let _e704 = vec2(80f);
                                modCoord_5 = (_e702 - (floor((_e702 / _e704)) * _e704));
                                returnMe_3 = vec4<f32>(0.9f, 0.9f, 0.9f, 1f);
                                let _e715 = doFlick;
                                if _e715 {
                                    let _e716 = coord_1;
                                    let _e718 = global.patternSeed;
                                    let _e724 = global.patternSeed;
                                    let _e727 = global.resolution;
                                    let _e730 = textureSampleLevel(isf_picture, isf_sampler, ((floor((_e716 / vec2((1f + _e718)))) * (1f + _e724)) / _e727), 0f);
                                    returnMe_3 = _e730;
                                } else {
                                    let _e731 = modCoord_5;
                                    let _e735 = modCoord_5;
                                    if ((_e731.x < 40f) && (_e735.y < 40f)) {
                                        returnMe_3 = vec4<f32>(0.5f, 0.5f, 0.5f, 1f);
                                    } else {
                                        let _e745 = modCoord_5;
                                        let _e749 = modCoord_5;
                                        if ((_e745.x > 40f) && (_e749.y > 40f)) {
                                            returnMe_3 = vec4<f32>(0.5f, 0.5f, 0.5f, 1f);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let _e759 = returnMe_3;
    return _e759;
}

fn main_1() {
    var p_5: vec2<f32>;
    var returnMe_4: vec4<f32>;
    var f_2: f32 = 1f;
    var didRowGlitch: bool = false;
    var rowGlitch: f32;
    var bgColor: vec4<f32>;
    var uv_1: vec2<f32>;
    var blockness_1: f32;
    var m_1: mat2x2<f32> = mat2x2<f32>(vec2<f32>(1.6f, 1.2f), vec2<f32>(-1.2f, 1.6f));
    var d_1: f32;
    var local_16: f32;
    var local_17: f32;
    var bgColor_1: vec4<f32>;
    var uv_2: vec2<f32>;
    var m_2: mat2x2<f32> = mat2x2<f32>(vec2<f32>(1.6f, 1.2f), vec2<f32>(-1.2f, 1.6f));
    var d_2: f32;
    var local_18: f32;
    var bgColor_2: vec4<f32>;
    var local_19: f32;

    let _e54 = gl_FragCoord_1;
    let _e56 = global.resolution;
    p_5 = (_e54.xy / _e56);
    let _e59 = isf_FragNormCoord_1;
    let _e61 = textureSampleLevel(isf_picture, isf_sampler, _e59, 0f);
    returnMe_4 = _e61;
    let _e67 = global.rowGlitchLevel;
    if (_e67 > 0f) {
        {
            let _e70 = p_5;
            let _e72 = global.patternSeed;
            let _e76 = global.noiseSeed;
            let _e80 = rand(vec2<f32>(((_e70.y + _e72) + 0.239f), (_e76 + 0.821f)));
            rowGlitch = _e80;
            let _e82 = rowGlitch;
            let _e83 = global.rowGlitchLevel;
            if (_e82 < _e83) {
                {
                    let _e85 = p_5;
                    let _e88 = global.patternSeed;
                    let _e94 = global.noiseSeed;
                    let _e99 = rand(vec2<f32>(((_e85.y + (1.287f * _e88)) + 2.239f), ((6.12f * _e94) + 3.477f)));
                    rowGlitch = _e99;
                    let _e100 = p_5;
                    let _e102 = rowGlitch;
                    if (_e100.x < _e102) {
                        {
                            let _e104 = rowGlitch;
                            let _e105 = p_5;
                            let _e109 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e104, _e105.y), 0f);
                            returnMe_4 = _e109;
                            let _e110 = rowGlitch;
                            let _e112 = global.rowGlitchSeed;
                            let _e114 = p_5;
                            let _e117 = global.rowGlitchSeed;
                            let _e119 = p_5;
                            let _e122 = rowGlitch;
                            let _e126 = global.rowGlitchSeed;
                            let _e128 = rand4_(vec4<f32>(_e110, ((1.28f * _e112) + _e114.y), (_e117 + floor(((10f * _e119.x) / _e122))), _e126));
                            bgColor = _e128;
                            let _e130 = returnMe_4;
                            let _e131 = bgColor;
                            let _e133 = global.rowFlickerLevel;
                            let _e134 = global.time;
                            let _e135 = p_5;
                            let _e138 = rowGlitch;
                            let _e139 = global.rowGlitchSeed;
                            let _e142 = rand(vec2<f32>((_e134 + _e135.y), (_e138 + _e139)));
                            returnMe_4 = mix(_e130, _e131, vec4((1f - (_e133 * _e142))));
                            didRowGlitch = true;
                        }
                    }
                }
            }
        }
    }
    {
        let _e148 = p_5;
        let _e149 = global.resolution;
        let _e151 = global.resolution;
        uv_1 = (_e148 * vec2<f32>((_e149.x / _e151.y), 1f));
        f_2 = 0f;
        let _e159 = global.glitchSmoothness;
        if (_e159 < 1f) {
            {
                let _e162 = global.noiseSeed;
                let _e163 = uv_1;
                let _e165 = global.glitchScale;
                let _e172 = global.glitchScale;
                let _e179 = rand((_e162 * (floor((_e163 * (1f + (_e165 * 100f)))) / vec2((1f + (_e172 * 100f))))));
                blockness_1 = _e179;
                let _e181 = blockness_1;
                let _e182 = global.glitchSmoothness;
                if (_e181 > _e182) {
                    let _e184 = uv_1;
                    let _e186 = global.glitchScale;
                    let _e193 = global.glitchScale;
                    uv_1 = (floor((_e184 * (1f + (_e186 * 25f)))) / vec2((1f + (_e193 * 25f))));
                }
            }
        }
        {
            let _e199 = uv_1;
            let _e200 = global.glitchScale;
            uv_1 = (_e199 * _e200);
            let _e212 = uv_1;
            let _e213 = global.noiseSeed;
            let _e214 = noise(_e212, _e213);
            f_2 = (0.5f * _e214);
            let _e216 = m_1;
            let _e217 = uv_1;
            uv_1 = (_e216 * _e217);
            let _e219 = f_2;
            let _e221 = uv_1;
            let _e222 = global.noiseSeed;
            let _e223 = noise(_e221, _e222);
            f_2 = (_e219 + (0.25f * _e223));
            let _e226 = m_1;
            let _e227 = uv_1;
            uv_1 = (_e226 * _e227);
            let _e229 = f_2;
            let _e231 = uv_1;
            let _e232 = global.noiseSeed;
            let _e233 = noise(_e231, _e232);
            f_2 = (_e229 + (0.125f * _e233));
            let _e236 = m_1;
            let _e237 = uv_1;
            uv_1 = (_e236 * _e237);
            let _e239 = f_2;
            let _e241 = uv_1;
            let _e242 = global.noiseSeed;
            let _e243 = noise(_e241, _e242);
            f_2 = (_e239 + (0.0625f * _e243));
            let _e246 = m_1;
            let _e247 = uv_1;
            uv_1 = (_e246 * _e247);
        }
        let _e250 = f_2;
        let _e252 = global.glitchBrightnessCurve;
        let _e255 = global.glitchBrightness;
        f_2 = (1f - (pow(_e250, (5f - _e252)) * _e255));
        let _e258 = isf_FragNormCoord_1;
        d_1 = distance(_e258, vec2(0.5f));
        let _e263 = d_1;
        let _e264 = global.glitchRadius;
        if (_e263 > _e264) {
            {
                let _e266 = f_2;
                let _e267 = d_1;
                let _e268 = global.glitchRadius;
                f_2 = (_e266 + (_e267 - _e268));
            }
        }
        let _e271 = f_2;
        if (_e271 > 1f) {
            local_16 = 1f;
        } else {
            let _e275 = f_2;
            local_16 = _e275;
        }
        let _e277 = local_16;
        f_2 = _e277;
        let _e278 = global.hardEdge_on;
        if (_e278 != 0i) {
            {
                let _e281 = f_2;
                if (_e281 > 0.9f) {
                    local_17 = 1f;
                } else {
                    local_17 = 0f;
                }
                let _e287 = local_17;
                f_2 = _e287;
            }
        }
        let _e288 = f_2;
        if (_e288 < 1f) {
            {
                let _e291 = gl_FragCoord_1;
                let _e293 = global.patternStyle;
                let _e294 = global.patternSeed;
                let _e297 = patternForType(_e291.xy, _e293, vec2<f32>(_e294, 0.2941f));
                bgColor_1 = _e297;
                let _e299 = returnMe_4;
                let _e300 = bgColor_1;
                let _e302 = f_2;
                returnMe_4 = mix(_e299, _e300, vec4((1f - _e302)));
            }
        }
    }
    {
        let _e306 = p_5;
        let _e307 = global.resolution;
        let _e309 = global.resolution;
        uv_2 = (_e306 * vec2<f32>((_e307.x / _e309.y), 1f));
        f_2 = 0f;
        {
            let _e317 = uv_2;
            let _e318 = global.tintScale;
            uv_2 = (_e317 * _e318);
            let _e330 = uv_2;
            let _e331 = global.tintSeed;
            let _e332 = noise(_e330, _e331);
            f_2 = (0.5f * _e332);
            let _e334 = m_2;
            let _e335 = uv_2;
            uv_2 = (_e334 * _e335);
            let _e337 = f_2;
            let _e339 = uv_2;
            let _e340 = global.tintSeed;
            let _e341 = noise(_e339, _e340);
            f_2 = (_e337 + (0.25f * _e341));
            let _e344 = m_2;
            let _e345 = uv_2;
            uv_2 = (_e344 * _e345);
            let _e347 = f_2;
            let _e349 = uv_2;
            let _e350 = global.tintSeed;
            let _e351 = noise(_e349, _e350);
            f_2 = (_e347 + (0.125f * _e351));
            let _e354 = m_2;
            let _e355 = uv_2;
            uv_2 = (_e354 * _e355);
            let _e357 = f_2;
            let _e359 = uv_2;
            let _e360 = global.tintSeed;
            let _e361 = noise(_e359, _e360);
            f_2 = (_e357 + (0.0625f * _e361));
            let _e364 = m_2;
            let _e365 = uv_2;
            uv_2 = (_e364 * _e365);
        }
        let _e368 = f_2;
        let _e370 = global.tintBrightnessCurve;
        let _e373 = global.tintBrightness;
        f_2 = (1f - (pow(_e368, (5f - _e370)) * _e373));
        let _e376 = isf_FragNormCoord_1;
        d_2 = distance(_e376, vec2(0.5f));
        let _e381 = d_2;
        let _e382 = global.tintRadius;
        if (_e381 > _e382) {
            {
                let _e384 = f_2;
                let _e385 = d_2;
                let _e386 = global.tintRadius;
                f_2 = (_e384 + (_e385 - _e386));
            }
        }
        let _e389 = f_2;
        if (_e389 > 1f) {
            local_18 = 1f;
        } else {
            let _e393 = f_2;
            local_18 = _e393;
        }
        let _e395 = local_18;
        f_2 = _e395;
        let _e396 = f_2;
        if (_e396 < 1f) {
            {
                let _e399 = global.tintColor1_;
                let _e400 = global.tintColor2_;
                let _e401 = f_2;
                let _e402 = global.tintSeed;
                let _e405 = rand(vec2<f32>(_e402, 0.231f));
                bgColor_2 = mix(_e399, _e400, vec4((_e401 + _e405)));
                let _e410 = global.hardEdge_on;
                if (_e410 != 0i) {
                    {
                        let _e413 = f_2;
                        if (_e413 > 0.9f) {
                            local_19 = 1f;
                        } else {
                            local_19 = 0f;
                        }
                        let _e419 = local_19;
                        f_2 = _e419;
                    }
                }
                let _e420 = returnMe_4;
                let _e421 = bgColor_2;
                let _e422 = returnMe_4;
                let _e425 = f_2;
                returnMe_4 = mix(_e420, (_e421 * _e422), vec4((1f - _e425)));
            }
        }
    }
    let _e429 = global.alphaMode;
    if (_e429 == 1i) {
        let _e433 = returnMe_4;
        let _e435 = f_2;
        returnMe_4.w = (_e433.w * _e435);
    } else {
        let _e437 = global.alphaMode;
        if (_e437 == 2i) {
            let _e441 = f_2;
            returnMe_4.w = _e441;
        }
    }
    let _e442 = returnMe_4;
    isf_FragColor = _e442;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e63 = isf_FragColor;
    return FragmentOutput(_e63);
}
