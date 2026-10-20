// Broken LCD: ISF-Files' Broken LCD.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Glitch
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
    passindex: f32,
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
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
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
    let _e62 = angle_1;
    s = sin((_e62 * pi));
    let _e66 = angle_1;
    c = cos((_e66 * pi));
    let _e70 = pt_1;
    returnMe = _e70;
    let _e72 = returnMe;
    let _e74 = center_1;
    returnMe.x = (_e72.x - _e74.x);
    let _e78 = returnMe;
    let _e80 = center_1;
    returnMe.y = (_e78.y - _e80.y);
    let _e83 = returnMe;
    let _e85 = c;
    let _e87 = returnMe;
    let _e89 = s;
    xnew = ((_e83.x * _e85) - (_e87.y * _e89));
    let _e93 = returnMe;
    let _e95 = s;
    let _e97 = returnMe;
    let _e99 = c;
    ynew = ((_e93.x * _e95) + (_e97.y * _e99));
    let _e104 = xnew;
    let _e105 = center_1;
    returnMe.x = (_e104 + _e105.x);
    let _e109 = ynew;
    let _e110 = center_1;
    returnMe.y = (_e109 + _e110.y);
    let _e113 = returnMe;
    return _e113;
}

fn sign_(p1_: vec2<f32>, p2_: vec2<f32>, p3_: vec2<f32>) -> f32 {
    var p1_1: vec2<f32>;
    var p2_1: vec2<f32>;
    var p3_1: vec2<f32>;

    p1_1 = p1_;
    p2_1 = p2_;
    p3_1 = p3_;
    let _e61 = p1_1;
    let _e63 = p3_1;
    let _e66 = p2_1;
    let _e68 = p3_1;
    let _e72 = p2_1;
    let _e74 = p3_1;
    let _e77 = p1_1;
    let _e79 = p3_1;
    return (((_e61.x - _e63.x) * (_e66.y - _e68.y)) - ((_e72.x - _e74.x) * (_e77.y - _e79.y)));
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
    let _e66 = pt_3;
    let _e67 = v1_1;
    let _e68 = v2_1;
    let _e69 = sign_(_e66, _e67, _e68);
    b1_ = (_e69 < 0f);
    let _e72 = pt_3;
    let _e73 = v2_1;
    let _e74 = v3_1;
    let _e75 = sign_(_e72, _e73, _e74);
    b2_ = (_e75 < 0f);
    let _e78 = pt_3;
    let _e79 = v3_1;
    let _e80 = v1_1;
    let _e81 = sign_(_e78, _e79, _e80);
    b3_ = (_e81 < 0f);
    let _e84 = b1_;
    let _e85 = b2_;
    let _e87 = b2_;
    let _e88 = b3_;
    return ((_e84 == _e85) && (_e87 == _e88));
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
    let _e68 = v1_3;
    v1r = _e68;
    let _e70 = v2_3;
    v2r = _e70;
    let _e72 = v3_3;
    v3r = _e72;
    let _e74 = pt_5;
    let _e75 = v1r;
    let _e76 = v2r;
    let _e77 = sign_(_e74, _e75, _e76);
    b1_1 = (_e77 < 0f);
    let _e80 = pt_5;
    let _e81 = v2r;
    let _e82 = v3r;
    let _e83 = sign_(_e80, _e81, _e82);
    b2_1 = (_e83 < 0f);
    let _e86 = pt_5;
    let _e87 = v3r;
    let _e88 = v1r;
    let _e89 = sign_(_e86, _e87, _e88);
    b3_1 = (_e89 < 0f);
    let _e92 = b1_1;
    let _e93 = b2_1;
    let _e95 = b2_1;
    let _e96 = b3_1;
    return ((_e92 == _e93) && (_e95 == _e96));
}

fn isPointInShape(pt_6: vec2<f32>, shape: i32, shapeCoordinates: vec4<f32>) -> f32 {
    var pt_7: vec2<f32>;
    var shape_1: i32;
    var shapeCoordinates_1: vec4<f32>;
    var returnMe_1: f32 = 0f;

    pt_7 = pt_6;
    shape_1 = shape;
    shapeCoordinates_1 = shapeCoordinates;
    let _e63 = shape_1;
    if (_e63 == 0i) {
        {
            let _e66 = pt_7;
            let _e67 = shapeCoordinates_1;
            let _e69 = shapeCoordinates_1;
            let _e72 = shapeCoordinates_1;
            let _e76 = shapeCoordinates_1;
            let _e78 = shapeCoordinates_1;
            let _e83 = shapeCoordinates_1;
            let _e85 = shapeCoordinates_1;
            let _e91 = RotatedPointInTriangle(_e66, _e67.xy, (_e69.xy + vec2<f32>(0f, _e72.w)), (_e76.xy + vec2<f32>(_e78.z, 0f)), (_e83.xy + (_e85.zw / vec2(2f))));
            if _e91 {
                {
                    returnMe_1 = 1f;
                    let _e93 = pt_7;
                    let _e95 = shapeCoordinates_1;
                    let _e98 = pt_7;
                    let _e100 = shapeCoordinates_1;
                    if ((_e93.x > _e95.x) && (_e98.x < _e100.x)) {
                        {
                            let _e104 = pt_7;
                            let _e106 = shapeCoordinates_1;
                            let _e109 = global.resolution;
                            returnMe_1 = clamp(((_e104.x - _e106.x) / _e109.x), 0f, 1f);
                            let _e115 = returnMe_1;
                            returnMe_1 = pow(_e115, 0.5f);
                        }
                    } else {
                        let _e118 = pt_7;
                        let _e120 = shapeCoordinates_1;
                        let _e122 = shapeCoordinates_1;
                        let _e126 = pt_7;
                        let _e128 = shapeCoordinates_1;
                        let _e130 = shapeCoordinates_1;
                        if ((_e118.x > (_e120.x + _e122.z)) && (_e126.x < (_e128.x + _e130.z))) {
                            {
                                let _e135 = shapeCoordinates_1;
                                let _e137 = shapeCoordinates_1;
                                let _e140 = pt_7;
                                let _e143 = global.resolution;
                                returnMe_1 = clamp((((_e135.x + _e137.z) - _e140.x) / _e143.x), 0f, 1f);
                                let _e149 = returnMe_1;
                                returnMe_1 = pow(_e149, 0.5f);
                            }
                        }
                    }
                }
            } else {
                let _e152 = pt_7;
                let _e153 = shapeCoordinates_1;
                let _e155 = shapeCoordinates_1;
                let _e158 = shapeCoordinates_1;
                let _e161 = shapeCoordinates_1;
                let _e165 = shapeCoordinates_1;
                let _e167 = shapeCoordinates_1;
                let _e172 = shapeCoordinates_1;
                let _e174 = shapeCoordinates_1;
                let _e180 = RotatedPointInTriangle(_e152, (_e153.xy + _e155.zw), (_e158.xy + vec2<f32>(0f, _e161.w)), (_e165.xy + vec2<f32>(_e167.z, 0f)), (_e172.xy + (_e174.zw / vec2(2f))));
                if _e180 {
                    {
                        returnMe_1 = 1f;
                        let _e182 = pt_7;
                        let _e184 = shapeCoordinates_1;
                        let _e187 = pt_7;
                        let _e189 = shapeCoordinates_1;
                        if ((_e182.x > _e184.x) && (_e187.x < _e189.x)) {
                            {
                                let _e193 = pt_7;
                                let _e195 = shapeCoordinates_1;
                                let _e198 = global.resolution;
                                returnMe_1 = clamp(((_e193.x - _e195.x) / _e198.x), 0f, 1f);
                                let _e204 = returnMe_1;
                                returnMe_1 = pow(_e204, 0.5f);
                            }
                        } else {
                            let _e207 = pt_7;
                            let _e209 = shapeCoordinates_1;
                            let _e211 = shapeCoordinates_1;
                            let _e215 = pt_7;
                            let _e217 = shapeCoordinates_1;
                            let _e219 = shapeCoordinates_1;
                            if ((_e207.x > (_e209.x + _e211.z)) && (_e215.x < (_e217.x + _e219.z))) {
                                {
                                    let _e224 = shapeCoordinates_1;
                                    let _e226 = shapeCoordinates_1;
                                    let _e229 = pt_7;
                                    let _e232 = global.resolution;
                                    returnMe_1 = clamp((((_e224.x + _e226.z) - _e229.x) / _e232.x), 0f, 1f);
                                    let _e238 = returnMe_1;
                                    returnMe_1 = pow(_e238, 0.5f);
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        let _e241 = shape_1;
        if (_e241 == 1i) {
            {
                let _e244 = pt_7;
                let _e245 = shapeCoordinates_1;
                let _e247 = shapeCoordinates_1;
                let _e249 = shapeCoordinates_1;
                let _e253 = shapeCoordinates_1;
                let _e257 = shapeCoordinates_1;
                let _e259 = shapeCoordinates_1;
                let _e264 = shapeCoordinates_1;
                let _e266 = shapeCoordinates_1;
                let _e272 = RotatedPointInTriangle(_e244, _e245.xy, (_e247.xy + vec2<f32>((_e249.z / 2f), _e253.w)), (_e257.xy + vec2<f32>(_e259.z, 0f)), (_e264.xy + (_e266.zw / vec2(2f))));
                if _e272 {
                    {
                        returnMe_1 = 1f;
                    }
                }
            }
        } else {
            let _e274 = shape_1;
            if (_e274 == 2i) {
                {
                    let _e277 = pt_7;
                    let _e278 = shapeCoordinates_1;
                    let _e280 = shapeCoordinates_1;
                    returnMe_1 = distance(_e277, vec2<f32>((_e278.xy + (_e280.zw / vec2(2f)))));
                    let _e288 = returnMe_1;
                    let _e289 = shapeCoordinates_1;
                    let _e291 = shapeCoordinates_1;
                    if (_e288 < (min(_e289.z, _e291.w) / 2f)) {
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
                let _e299 = shape_1;
                if (_e299 == 3i) {
                    {
                        let _e302 = pt_7;
                        let _e303 = shapeCoordinates_1;
                        let _e306 = shapeCoordinates_1;
                        let _e312 = shapeCoordinates_1;
                        let _e314 = shapeCoordinates_1;
                        let _e318 = shapeCoordinates_1;
                        let _e322 = shapeCoordinates_1;
                        let _e324 = shapeCoordinates_1;
                        let _e326 = shapeCoordinates_1;
                        let _e332 = shapeCoordinates_1;
                        let _e334 = shapeCoordinates_1;
                        let _e340 = RotatedPointInTriangle(_e302, (_e303.xy + vec2<f32>(0f, (_e306.w / 2f))), (_e312.xy + vec2<f32>((_e314.z / 2f), _e318.w)), (_e322.xy + vec2<f32>(_e324.z, (_e326.w / 2f))), (_e332.xy + (_e334.zw / vec2(2f))));
                        if _e340 {
                            {
                                returnMe_1 = 1f;
                            }
                        } else {
                            let _e342 = pt_7;
                            let _e343 = shapeCoordinates_1;
                            let _e346 = shapeCoordinates_1;
                            let _e352 = shapeCoordinates_1;
                            let _e354 = shapeCoordinates_1;
                            let _e361 = shapeCoordinates_1;
                            let _e363 = shapeCoordinates_1;
                            let _e365 = shapeCoordinates_1;
                            let _e371 = shapeCoordinates_1;
                            let _e373 = shapeCoordinates_1;
                            let _e379 = RotatedPointInTriangle(_e342, (_e343.xy + vec2<f32>(0f, (_e346.w / 2f))), (_e352.xy + vec2<f32>((_e354.z / 2f), 0f)), (_e361.xy + vec2<f32>(_e363.z, (_e365.w / 2f))), (_e371.xy + (_e373.zw / vec2(2f))));
                            if _e379 {
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
    let _e381 = returnMe_1;
    return _e381;
}

fn hash(p: vec2<f32>, seed: f32) -> f32 {
    var p_1: vec2<f32>;
    var seed_1: f32;

    p_1 = p;
    seed_1 = seed;
    let _e60 = seed_1;
    let _e63 = p_1;
    p_1 = (50f * fract(((vec2((_e60 * 1.17921f)) + (_e63 * 0.3183099f)) + vec2<f32>(0.71f, 0.113f))));
    let _e77 = p_1;
    let _e79 = p_1;
    let _e82 = p_1;
    let _e84 = p_1;
    return (-1f + (2f * fract(((_e77.x * _e79.y) * (_e82.x + _e84.y)))));
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
    let _e59 = p_3;
    i = floor(_e59);
    let _e62 = p_3;
    f = fract(_e62);
    let _e65 = f;
    let _e66 = f;
    let _e70 = f;
    u = ((_e65 * _e66) * (vec2(3f) - (2f * _e70)));
    let _e76 = i;
    let _e81 = seed_3;
    let _e83 = hash((_e76 + vec2<f32>(0f, 0f)), floor(_e81));
    let _e84 = i;
    let _e89 = seed_3;
    let _e91 = hash((_e84 + vec2<f32>(1f, 0f)), floor(_e89));
    let _e92 = u;
    let _e95 = i;
    let _e100 = seed_3;
    let _e102 = hash((_e95 + vec2<f32>(0f, 1f)), floor(_e100));
    let _e103 = i;
    let _e108 = seed_3;
    let _e110 = hash((_e103 + vec2<f32>(1f, 1f)), floor(_e108));
    let _e111 = u;
    let _e114 = u;
    v1_4 = mix(mix(_e83, _e91, _e92.x), mix(_e102, _e110, _e111.x), _e114.y);
    let _e118 = i;
    let _e123 = seed_3;
    let _e125 = hash((_e118 + vec2<f32>(0f, 0f)), ceil(_e123));
    let _e126 = i;
    let _e131 = seed_3;
    let _e133 = hash((_e126 + vec2<f32>(1f, 0f)), ceil(_e131));
    let _e134 = u;
    let _e137 = i;
    let _e142 = seed_3;
    let _e144 = hash((_e137 + vec2<f32>(0f, 1f)), ceil(_e142));
    let _e145 = i;
    let _e150 = seed_3;
    let _e152 = hash((_e145 + vec2<f32>(1f, 1f)), ceil(_e150));
    let _e153 = u;
    let _e156 = u;
    v2_4 = mix(mix(_e125, _e133, _e134.x), mix(_e144, _e152, _e153.x), _e156.y);
    let _e160 = v1_4;
    let _e161 = v2_4;
    let _e162 = seed_3;
    return mix(_e160, _e161, fract(_e162));
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
    let _e65 = o1_1;
    let _e66 = o2_1;
    let _e67 = o1_1;
    let _e69 = n_1;
    let _e70 = i1_1;
    let _e73 = i2_1;
    let _e74 = i1_1;
    return (_e65 + (((_e66 - _e67) * (_e69 - _e70)) / (_e73 - _e74)));
}

fn rand(co: vec2<f32>) -> f32 {
    var co_1: vec2<f32>;

    co_1 = co;
    let _e57 = co_1;
    return fract((sin(dot(_e57.xy, vec2<f32>(12.9898f, 78.233f))) * 43758.547f));
}

fn rand4_(co_2: vec4<f32>) -> vec4<f32> {
    var co_3: vec4<f32>;
    var returnMe_2: vec4<f32> = vec4(0f);

    co_3 = co_2;
    let _e61 = co_3;
    let _e63 = rand(_e61.xy);
    returnMe_2.x = _e63;
    let _e65 = co_3;
    let _e67 = rand(_e65.yz);
    returnMe_2.y = _e67;
    let _e69 = co_3;
    let _e71 = rand(_e69.zw);
    returnMe_2.z = _e71;
    let _e73 = co_3;
    let _e75 = rand(_e73.xz);
    returnMe_2.w = _e75;
    let _e76 = returnMe_2;
    return _e76;
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
    let _e64 = pType_1;
    pt_8 = _e64;
    let _e66 = pt_8;
    if (_e66 <= 2i) {
        {
            let _e70 = gl_FragCoord_1;
            let _e72 = global.resolution;
            p_4 = (_e70.xy / _e72);
            let _e75 = p_4;
            let _e76 = global.resolution;
            let _e78 = global.resolution;
            uv = (_e75 * vec2<f32>((_e76.x / _e78.y), 1f));
            let _e87 = global.glitchSmoothness;
            if (_e87 < 1f) {
                {
                    let _e90 = global.noiseSeed;
                    let _e91 = uv;
                    let _e93 = global.glitchScale;
                    let _e100 = global.glitchScale;
                    let _e107 = rand((_e90 * (floor((_e91 * (1f + (_e93 * 50f)))) / vec2((1f + (_e100 * 50f))))));
                    blockness = _e107;
                    let _e109 = blockness;
                    let _e110 = global.glitchSmoothness;
                    if (_e109 > _e110) {
                        let _e112 = uv;
                        let _e114 = global.glitchScale;
                        let _e121 = global.glitchScale;
                        uv = (floor((_e112 * (1f + (_e114 * 20f)))) / vec2((1f + (_e121 * 20f))));
                    }
                }
            }
            {
                let _e127 = uv;
                let _e128 = global.glitchScale;
                uv = (_e127 * _e128);
                let _e140 = uv;
                let _e141 = global.patternSeed;
                let _e142 = noise(_e140, _e141);
                f_1 = (0.5f * _e142);
                let _e144 = m;
                let _e145 = uv;
                uv = (_e144 * _e145);
                let _e147 = f_1;
                let _e149 = uv;
                let _e150 = global.patternSeed;
                let _e151 = noise(_e149, _e150);
                f_1 = (_e147 + (0.25f * _e151));
                let _e154 = m;
                let _e155 = uv;
                uv = (_e154 * _e155);
                let _e157 = f_1;
                let _e159 = uv;
                let _e160 = global.patternSeed;
                let _e161 = noise(_e159, _e160);
                f_1 = (_e157 + (0.125f * _e161));
                let _e164 = m;
                let _e165 = uv;
                uv = (_e164 * _e165);
                let _e167 = f_1;
                let _e169 = uv;
                let _e170 = global.patternSeed;
                let _e171 = noise(_e169, _e170);
                f_1 = (_e167 + (0.0625f * _e171));
                let _e174 = m;
                let _e175 = uv;
                uv = (_e174 * _e175);
            }
            let _e177 = f_1;
            f_1 = pow(_e177, 0.5f);
            let _e180 = isf_FragNormCoord_1;
            d = distance(_e180, vec2(0.5f));
            let _e185 = d;
            let _e186 = global.glitchRadius;
            if (_e185 > _e186) {
                {
                    let _e188 = f_1;
                    let _e189 = d;
                    let _e190 = global.glitchRadius;
                    f_1 = (_e188 + (_e189 - _e190));
                }
            }
            let _e193 = pt_8;
            if (_e193 == 0i) {
                {
                    let _e197 = f_1;
                    pt_8 = (5i + i32((min(_e197, 1f) * 3.99f)));
                }
            } else {
                let _e204 = pt_8;
                if (_e204 == 1i) {
                    {
                        let _e208 = f_1;
                        pt_8 = (3i + i32((min(_e208, 1f) * 3.99f)));
                    }
                } else {
                    let _e215 = pt_8;
                    if (_e215 == 2i) {
                        {
                            let _e219 = f_1;
                            pt_8 = (4i + i32((_e219 * 5.99f)));
                        }
                    }
                }
            }
        }
    }
    let _e224 = coord_1;
    flickCoord = _e224;
    let _e226 = pt_8;
    if (_e226 == 5i) {
        {
            let _e229 = coord_1;
            let _e231 = coord_1;
            let _e234 = global.patternSeed;
            let _e239 = global.patternSeed;
            flickCoord = vec2<f32>(_e229.x, (floor((_e231.y / (1f + _e234))) * (1f + _e239)));
        }
    } else {
        let _e243 = pt_8;
        if (_e243 == 6i) {
            {
                let _e246 = coord_1;
                let _e249 = global.patternSeed;
                let _e254 = global.patternSeed;
                let _e257 = coord_1;
                flickCoord = vec2<f32>((floor((_e246.x / (1f + _e249))) * (1f + _e254)), _e257.y);
            }
        } else {
            let _e260 = pt_8;
            if (_e260 > 6i) {
                {
                    let _e263 = coord_1;
                    let _e265 = global.patternSeed;
                    let _e271 = global.patternSeed;
                    flickCoord = (floor((_e263 / vec2((1f + _e265)))) * (1f + _e271));
                }
            }
        }
    }
    let _e274 = global.flickerLevel;
    if (_e274 > 0f) {
        let _e277 = global.time;
        let _e278 = flickCoord;
        let _e280 = global.patternSeed;
        let _e283 = rand(((_e277 * _e278) + vec2(_e280)));
        local = _e283;
    } else {
        local = 0f;
    }
    let _e286 = local;
    flickRand = _e286;
    let _e290 = flickRand;
    let _e291 = global.flickerLevel;
    if (_e290 < _e291) {
        {
            doFlick = true;
        }
    }
    let _e294 = pt_8;
    if (_e294 == 3i) {
        {
            returnMe_3 = vec4(0f);
        }
    } else {
        let _e299 = pt_8;
        if (_e299 == 4i) {
            {
                let _e302 = doFlick;
                if _e302 {
                    {
                        let _e303 = isf_FragNormCoord_1;
                        let _e305 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e303, 0f);
                        returnMe_3 = _e305;
                    }
                } else {
                    {
                        let _e306 = global.resolution;
                        let _e307 = global.patternSeed;
                        let _e308 = coord_1;
                        let _e310 = rand((_e307 * _e308));
                        let _e311 = global.patternSeed;
                        let _e312 = coord_1;
                        let _e315 = rand((_e311 * _e312.yx));
                        modCoord = (_e306 * vec2<f32>(_e310, _e315));
                        let _e319 = modCoord;
                        let _e321 = global.patternSeed;
                        modCoord = (_e319 * max((1f + _e321), 1.00001f));
                        let _e327 = modCoord;
                        let _e329 = modCoord;
                        let _e331 = (_e327.x + _e329.y);
                        if ((_e331 - (floor((_e331 / 4f)) * 4f)) < 2f) {
                            local_1 = 1f;
                        } else {
                            local_1 = 0f;
                        }
                        let _e342 = local_1;
                        returnMe_3.x = _e342;
                        let _e344 = modCoord;
                        let _e346 = modCoord;
                        let _e348 = (_e344.x + _e346.y);
                        if ((_e348 - (floor((_e348 / 5f)) * 5f)) < 2f) {
                            local_2 = 1f;
                        } else {
                            local_2 = 0f;
                        }
                        let _e359 = local_2;
                        returnMe_3.y = _e359;
                        let _e361 = modCoord;
                        let _e363 = modCoord;
                        let _e365 = (_e361.x + _e363.y);
                        if ((_e365 - (floor((_e365 / 7f)) * 7f)) < 2f) {
                            local_3 = 1f;
                        } else {
                            local_3 = 0f;
                        }
                        let _e376 = local_3;
                        returnMe_3.z = _e376;
                        returnMe_3.w = 1f;
                    }
                }
            }
        } else {
            let _e379 = pt_8;
            if (_e379 == 5i) {
                {
                    let _e382 = global.patternSeed;
                    let _e383 = coord_1;
                    let _e386 = rand(vec2<f32>(_e382, _e383.y));
                    if (_e386 < 0.25f) {
                        let _e389 = coord_1;
                        local_4 = _e389;
                    } else {
                        let _e390 = coord_1;
                        local_4 = _e390.yx;
                    }
                    let _e393 = local_4;
                    modCoord_1 = _e393;
                    let _e395 = modCoord_1;
                    let _e397 = global.patternSeed;
                    modCoord_1 = (_e395 * (1f + _e397));
                    let _e400 = doFlick;
                    if _e400 {
                        {
                            let _e401 = modCoord_1;
                            let _e405 = global.resolution;
                            let _e408 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, (vec2<f32>(_e401.x, 0f) / _e405), 0f);
                            let _e409 = modCoord_1;
                            let _e411 = global.resolution;
                            let _e414 = global.resolution;
                            let _e417 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, (vec2<f32>(_e409.x, _e411.y) / _e414), 0f);
                            let _e418 = modCoord_1;
                            let _e420 = global.resolution;
                            returnMe_3 = mix(_e408, _e417, vec4((_e418.y / _e420.y)));
                        }
                    } else {
                        {
                            let _e426 = modCoord_1;
                            if ((_e426.x - (floor((_e426.x / 4f)) * 4f)) < 1f) {
                                local_5 = 1f;
                            } else {
                                local_5 = 0f;
                            }
                            let _e438 = local_5;
                            returnMe_3.x = _e438;
                            let _e440 = modCoord_1;
                            if ((_e440.x - (floor((_e440.x / 5f)) * 5f)) < 1f) {
                                local_6 = 1f;
                            } else {
                                local_6 = 0f;
                            }
                            let _e452 = local_6;
                            returnMe_3.y = _e452;
                            let _e454 = modCoord_1;
                            if ((_e454.x - (floor((_e454.x / 7f)) * 7f)) < 1f) {
                                local_7 = 1f;
                            } else {
                                local_7 = 0f;
                            }
                            let _e466 = local_7;
                            returnMe_3.z = _e466;
                            returnMe_3.w = 1f;
                        }
                    }
                }
            } else {
                let _e469 = pt_8;
                if (_e469 == 6i) {
                    {
                        let _e472 = coord_1;
                        modCoord_2 = _e472;
                        let _e475 = modCoord_2;
                        let _e478 = global.patternSeed;
                        modCoord_2.x = (_e475.x * (1f + _e478));
                        let _e481 = modCoord_2;
                        let _e483 = global.patternSeed;
                        modCoord_2 = (_e481 * (1f + _e483));
                        let _e486 = modCoord_2;
                        let _e487 = global.resolution;
                        modCoord_2 = (_e486 - (floor((_e486 / _e487)) * _e487));
                        let _e492 = doFlick;
                        if _e492 {
                            {
                                let _e493 = modCoord_2;
                                let _e497 = global.resolution;
                                let _e500 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, (vec2<f32>(_e493.x, 0f) / _e497), 0f);
                                let _e501 = modCoord_2;
                                let _e503 = global.resolution;
                                let _e506 = global.resolution;
                                let _e509 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, (vec2<f32>(_e501.x, _e503.y) / _e506), 0f);
                                let _e510 = modCoord_2;
                                let _e512 = global.resolution;
                                returnMe_3 = mix(_e500, _e509, vec4((_e510.y / _e512.y)));
                            }
                        } else {
                            {
                                let _e518 = modCoord_2;
                                if ((_e518.x - (floor((_e518.x / 4f)) * 4f)) < 1f) {
                                    local_8 = 1f;
                                } else {
                                    local_8 = 0f;
                                }
                                let _e530 = local_8;
                                returnMe_3.x = _e530;
                                let _e532 = modCoord_2;
                                if ((_e532.x - (floor((_e532.x / 5f)) * 5f)) < 1f) {
                                    local_9 = 1f;
                                } else {
                                    local_9 = 0f;
                                }
                                let _e544 = local_9;
                                returnMe_3.y = _e544;
                                let _e546 = modCoord_2;
                                if ((_e546.x - (floor((_e546.x / 7f)) * 7f)) < 1f) {
                                    local_10 = 1f;
                                } else {
                                    local_10 = 0f;
                                }
                                let _e558 = local_10;
                                returnMe_3.z = _e558;
                                returnMe_3.w = 1f;
                            }
                        }
                    }
                } else {
                    let _e561 = pt_8;
                    if (_e561 == 7i) {
                        {
                            let _e564 = coord_1;
                            modCoord_3 = _e564;
                            let _e567 = modCoord_3;
                            let _e570 = global.patternSeed;
                            modCoord_3.y = (_e567.y * (1f + _e570));
                            let _e573 = modCoord_3;
                            let _e574 = global.resolution;
                            modCoord_3 = (_e573 - (floor((_e573 / _e574)) * _e574));
                            let _e579 = doFlick;
                            if _e579 {
                                {
                                    let _e581 = modCoord_3;
                                    let _e584 = global.resolution;
                                    let _e587 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, (vec2<f32>(0f, _e581.y) / _e584), 0f);
                                    let _e588 = global.resolution;
                                    let _e590 = modCoord_3;
                                    let _e593 = global.resolution;
                                    let _e596 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, (vec2<f32>(_e588.x, _e590.y) / _e593), 0f);
                                    let _e597 = modCoord_3;
                                    let _e599 = global.resolution;
                                    returnMe_3 = mix(_e587, _e596, vec4((_e597.x / _e599.x)));
                                }
                            } else {
                                {
                                    let _e605 = modCoord_3;
                                    if ((_e605.y - (floor((_e605.y / 4f)) * 4f)) < 1f) {
                                        local_11 = 1f;
                                    } else {
                                        local_11 = 0f;
                                    }
                                    let _e617 = local_11;
                                    returnMe_3.x = _e617;
                                    let _e619 = modCoord_3;
                                    if ((_e619.y - (floor((_e619.y / 5f)) * 5f)) < 1f) {
                                        local_12 = 1f;
                                    } else {
                                        local_12 = 0f;
                                    }
                                    let _e631 = local_12;
                                    returnMe_3.y = _e631;
                                    let _e633 = modCoord_3;
                                    if ((_e633.y - (floor((_e633.y / 7f)) * 7f)) < 1f) {
                                        local_13 = 1f;
                                    } else {
                                        local_13 = 0f;
                                    }
                                    let _e645 = local_13;
                                    returnMe_3.z = _e645;
                                    returnMe_3.w = 1f;
                                }
                            }
                        }
                    } else {
                        let _e648 = pt_8;
                        if (_e648 == 8i) {
                            {
                                let _e651 = coord_1;
                                let _e653 = global.patternSeed;
                                let _e655 = (_e651 * (1f + _e653));
                                let _e657 = vec2(10f);
                                modCoord_4 = (_e655 - (floor((_e655 / _e657)) * _e657));
                                let _e663 = doFlick;
                                if _e663 {
                                    {
                                        let _e664 = modCoord_4;
                                        if (_e664.x < 5f) {
                                            let _e668 = modCoord_4;
                                            let _e672 = global.resolution;
                                            let _e675 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, (vec2<f32>(_e668.x, 0f) / _e672), 0f);
                                            local_14 = _e675;
                                        } else {
                                            let _e676 = modCoord_4;
                                            let _e678 = global.resolution;
                                            let _e681 = global.resolution;
                                            let _e684 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, (vec2<f32>(_e676.x, _e678.y) / _e681), 0f);
                                            local_14 = _e684;
                                        }
                                        let _e686 = local_14;
                                        returnMe_3 = _e686;
                                    }
                                } else {
                                    {
                                        let _e687 = modCoord_4;
                                        if (_e687.x < 5f) {
                                            local_15 = vec4(1f);
                                        } else {
                                            local_15 = vec4<f32>(0f, 0f, 0f, 1f);
                                        }
                                        let _e699 = local_15;
                                        returnMe_3 = _e699;
                                    }
                                }
                            }
                        } else {
                            {
                                let _e700 = coord_1;
                                let _e702 = global.patternSeed;
                                let _e704 = (_e700 * (1f + _e702));
                                let _e706 = vec2(80f);
                                modCoord_5 = (_e704 - (floor((_e704 / _e706)) * _e706));
                                returnMe_3 = vec4<f32>(0.9f, 0.9f, 0.9f, 1f);
                                let _e717 = doFlick;
                                if _e717 {
                                    let _e718 = coord_1;
                                    let _e720 = global.patternSeed;
                                    let _e726 = global.patternSeed;
                                    let _e729 = global.resolution;
                                    let _e732 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, ((floor((_e718 / vec2((1f + _e720)))) * (1f + _e726)) / _e729), 0f);
                                    returnMe_3 = _e732;
                                } else {
                                    let _e733 = modCoord_5;
                                    let _e737 = modCoord_5;
                                    if ((_e733.x < 40f) && (_e737.y < 40f)) {
                                        returnMe_3 = vec4<f32>(0.5f, 0.5f, 0.5f, 1f);
                                    } else {
                                        let _e747 = modCoord_5;
                                        let _e751 = modCoord_5;
                                        if ((_e747.x > 40f) && (_e751.y > 40f)) {
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
    let _e761 = returnMe_3;
    return _e761;
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

    let _e56 = gl_FragCoord_1;
    let _e58 = global.resolution;
    p_5 = (_e56.xy / _e58);
    let _e61 = isf_FragNormCoord_1;
    let _e63 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e61, 0f);
    returnMe_4 = _e63;
    let _e69 = global.rowGlitchLevel;
    if (_e69 > 0f) {
        {
            let _e72 = p_5;
            let _e74 = global.patternSeed;
            let _e78 = global.noiseSeed;
            let _e82 = rand(vec2<f32>(((_e72.y + _e74) + 0.239f), (_e78 + 0.821f)));
            rowGlitch = _e82;
            let _e84 = rowGlitch;
            let _e85 = global.rowGlitchLevel;
            if (_e84 < _e85) {
                {
                    let _e87 = p_5;
                    let _e90 = global.patternSeed;
                    let _e96 = global.noiseSeed;
                    let _e101 = rand(vec2<f32>(((_e87.y + (1.287f * _e90)) + 2.239f), ((6.12f * _e96) + 3.477f)));
                    rowGlitch = _e101;
                    let _e102 = p_5;
                    let _e104 = rowGlitch;
                    if (_e102.x < _e104) {
                        {
                            let _e106 = rowGlitch;
                            let _e107 = p_5;
                            let _e111 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e106, _e107.y), 0f);
                            returnMe_4 = _e111;
                            let _e112 = rowGlitch;
                            let _e114 = global.rowGlitchSeed;
                            let _e116 = p_5;
                            let _e119 = global.rowGlitchSeed;
                            let _e121 = p_5;
                            let _e124 = rowGlitch;
                            let _e128 = global.rowGlitchSeed;
                            let _e130 = rand4_(vec4<f32>(_e112, ((1.28f * _e114) + _e116.y), (_e119 + floor(((10f * _e121.x) / _e124))), _e128));
                            bgColor = _e130;
                            let _e132 = returnMe_4;
                            let _e133 = bgColor;
                            let _e135 = global.rowFlickerLevel;
                            let _e136 = global.time;
                            let _e137 = p_5;
                            let _e140 = rowGlitch;
                            let _e141 = global.rowGlitchSeed;
                            let _e144 = rand(vec2<f32>((_e136 + _e137.y), (_e140 + _e141)));
                            returnMe_4 = mix(_e132, _e133, vec4((1f - (_e135 * _e144))));
                            didRowGlitch = true;
                        }
                    }
                }
            }
        }
    }
    {
        let _e150 = p_5;
        let _e151 = global.resolution;
        let _e153 = global.resolution;
        uv_1 = (_e150 * vec2<f32>((_e151.x / _e153.y), 1f));
        f_2 = 0f;
        let _e161 = global.glitchSmoothness;
        if (_e161 < 1f) {
            {
                let _e164 = global.noiseSeed;
                let _e165 = uv_1;
                let _e167 = global.glitchScale;
                let _e174 = global.glitchScale;
                let _e181 = rand((_e164 * (floor((_e165 * (1f + (_e167 * 100f)))) / vec2((1f + (_e174 * 100f))))));
                blockness_1 = _e181;
                let _e183 = blockness_1;
                let _e184 = global.glitchSmoothness;
                if (_e183 > _e184) {
                    let _e186 = uv_1;
                    let _e188 = global.glitchScale;
                    let _e195 = global.glitchScale;
                    uv_1 = (floor((_e186 * (1f + (_e188 * 25f)))) / vec2((1f + (_e195 * 25f))));
                }
            }
        }
        {
            let _e201 = uv_1;
            let _e202 = global.glitchScale;
            uv_1 = (_e201 * _e202);
            let _e214 = uv_1;
            let _e215 = global.noiseSeed;
            let _e216 = noise(_e214, _e215);
            f_2 = (0.5f * _e216);
            let _e218 = m_1;
            let _e219 = uv_1;
            uv_1 = (_e218 * _e219);
            let _e221 = f_2;
            let _e223 = uv_1;
            let _e224 = global.noiseSeed;
            let _e225 = noise(_e223, _e224);
            f_2 = (_e221 + (0.25f * _e225));
            let _e228 = m_1;
            let _e229 = uv_1;
            uv_1 = (_e228 * _e229);
            let _e231 = f_2;
            let _e233 = uv_1;
            let _e234 = global.noiseSeed;
            let _e235 = noise(_e233, _e234);
            f_2 = (_e231 + (0.125f * _e235));
            let _e238 = m_1;
            let _e239 = uv_1;
            uv_1 = (_e238 * _e239);
            let _e241 = f_2;
            let _e243 = uv_1;
            let _e244 = global.noiseSeed;
            let _e245 = noise(_e243, _e244);
            f_2 = (_e241 + (0.0625f * _e245));
            let _e248 = m_1;
            let _e249 = uv_1;
            uv_1 = (_e248 * _e249);
        }
        let _e252 = f_2;
        let _e254 = global.glitchBrightnessCurve;
        let _e257 = global.glitchBrightness;
        f_2 = (1f - (pow(_e252, (5f - _e254)) * _e257));
        let _e260 = isf_FragNormCoord_1;
        d_1 = distance(_e260, vec2(0.5f));
        let _e265 = d_1;
        let _e266 = global.glitchRadius;
        if (_e265 > _e266) {
            {
                let _e268 = f_2;
                let _e269 = d_1;
                let _e270 = global.glitchRadius;
                f_2 = (_e268 + (_e269 - _e270));
            }
        }
        let _e273 = f_2;
        if (_e273 > 1f) {
            local_16 = 1f;
        } else {
            let _e277 = f_2;
            local_16 = _e277;
        }
        let _e279 = local_16;
        f_2 = _e279;
        let _e280 = global.hardEdge_on;
        if (_e280 != 0i) {
            {
                let _e283 = f_2;
                if (_e283 > 0.9f) {
                    local_17 = 1f;
                } else {
                    local_17 = 0f;
                }
                let _e289 = local_17;
                f_2 = _e289;
            }
        }
        let _e290 = f_2;
        if (_e290 < 1f) {
            {
                let _e293 = gl_FragCoord_1;
                let _e295 = global.patternStyle;
                let _e296 = global.patternSeed;
                let _e299 = patternForType(_e293.xy, _e295, vec2<f32>(_e296, 0.2941f));
                bgColor_1 = _e299;
                let _e301 = returnMe_4;
                let _e302 = bgColor_1;
                let _e304 = f_2;
                returnMe_4 = mix(_e301, _e302, vec4((1f - _e304)));
            }
        }
    }
    {
        let _e308 = p_5;
        let _e309 = global.resolution;
        let _e311 = global.resolution;
        uv_2 = (_e308 * vec2<f32>((_e309.x / _e311.y), 1f));
        f_2 = 0f;
        {
            let _e319 = uv_2;
            let _e320 = global.tintScale;
            uv_2 = (_e319 * _e320);
            let _e332 = uv_2;
            let _e333 = global.tintSeed;
            let _e334 = noise(_e332, _e333);
            f_2 = (0.5f * _e334);
            let _e336 = m_2;
            let _e337 = uv_2;
            uv_2 = (_e336 * _e337);
            let _e339 = f_2;
            let _e341 = uv_2;
            let _e342 = global.tintSeed;
            let _e343 = noise(_e341, _e342);
            f_2 = (_e339 + (0.25f * _e343));
            let _e346 = m_2;
            let _e347 = uv_2;
            uv_2 = (_e346 * _e347);
            let _e349 = f_2;
            let _e351 = uv_2;
            let _e352 = global.tintSeed;
            let _e353 = noise(_e351, _e352);
            f_2 = (_e349 + (0.125f * _e353));
            let _e356 = m_2;
            let _e357 = uv_2;
            uv_2 = (_e356 * _e357);
            let _e359 = f_2;
            let _e361 = uv_2;
            let _e362 = global.tintSeed;
            let _e363 = noise(_e361, _e362);
            f_2 = (_e359 + (0.0625f * _e363));
            let _e366 = m_2;
            let _e367 = uv_2;
            uv_2 = (_e366 * _e367);
        }
        let _e370 = f_2;
        let _e372 = global.tintBrightnessCurve;
        let _e375 = global.tintBrightness;
        f_2 = (1f - (pow(_e370, (5f - _e372)) * _e375));
        let _e378 = isf_FragNormCoord_1;
        d_2 = distance(_e378, vec2(0.5f));
        let _e383 = d_2;
        let _e384 = global.tintRadius;
        if (_e383 > _e384) {
            {
                let _e386 = f_2;
                let _e387 = d_2;
                let _e388 = global.tintRadius;
                f_2 = (_e386 + (_e387 - _e388));
            }
        }
        let _e391 = f_2;
        if (_e391 > 1f) {
            local_18 = 1f;
        } else {
            let _e395 = f_2;
            local_18 = _e395;
        }
        let _e397 = local_18;
        f_2 = _e397;
        let _e398 = f_2;
        if (_e398 < 1f) {
            {
                let _e401 = global.tintColor1_;
                let _e402 = global.tintColor2_;
                let _e403 = f_2;
                let _e404 = global.tintSeed;
                let _e407 = rand(vec2<f32>(_e404, 0.231f));
                bgColor_2 = mix(_e401, _e402, vec4((_e403 + _e407)));
                let _e412 = global.hardEdge_on;
                if (_e412 != 0i) {
                    {
                        let _e415 = f_2;
                        if (_e415 > 0.9f) {
                            local_19 = 1f;
                        } else {
                            local_19 = 0f;
                        }
                        let _e421 = local_19;
                        f_2 = _e421;
                    }
                }
                let _e422 = returnMe_4;
                let _e423 = bgColor_2;
                let _e424 = returnMe_4;
                let _e427 = f_2;
                returnMe_4 = mix(_e422, (_e423 * _e424), vec4((1f - _e427)));
            }
        }
    }
    let _e431 = global.alphaMode;
    if (_e431 == 1i) {
        let _e435 = returnMe_4;
        let _e437 = f_2;
        returnMe_4.w = (_e435.w * _e437);
    } else {
        let _e439 = global.alphaMode;
        if (_e439 == 2i) {
            let _e443 = f_2;
            returnMe_4.w = _e443;
        }
    }
    let _e444 = returnMe_4;
    isf_FragColor = _e444;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e65 = isf_FragColor;
    return FragmentOutput(_e65);
}
