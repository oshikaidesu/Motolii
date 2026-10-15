// Edge Blowout: ISF-Files' Edge Blowout.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@leftEdge:Left Edge,0,1,0.25,0.001
// --track@rightEdge:Right Edge,0,1,0.75,0.001
// --track@bottomEdge:Bottom Edge,0,1,0.25,0.001
// --track@topEdge:Top Edge,0,1,0.75,0.001
// --check@doHorizontal_on:Horizontal Bleed,1
// --check@doVertical_on:Vertical Bleed,1
// --check@insideBleed_on:Inside Bleed,1
// --check@outsideBleed_on:Outside Bleed,1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    leftEdge: f32,
    rightEdge: f32,
    bottomEdge: f32,
    topEdge: f32,
    doHorizontal_on: i32,
    doVertical_on: i32,
    insideBleed_on: i32,
    outsideBleed_on: i32,
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
    var outputPixelColor: vec4<f32>;
    var local: f32;
    var realLeftEdge: f32;
    var local_1: f32;
    var realRightEdge: f32;
    var local_2: f32;
    var realBottomEdge: f32;
    var local_3: f32;
    var realTopEdge: f32;
    var sampleCoord: vec2<f32>;
    var insideBox: bool = false;
    var region: vec2<f32> = vec2<f32>(0f, 0f);
    var leftDistance: f32;
    var rightDistance: f32;
    var bottomDistance: f32;
    var topDistance: f32;
    var totalDistance: f32;
    var leftPixel: vec4<f32>;
    var rightPixel: vec4<f32>;
    var bottomPixel: vec4<f32>;
    var topPixel: vec4<f32>;
    var leftDistance_1: f32;
    var rightDistance_1: f32;
    var totalDistance_1: f32;
    var leftPixel_1: vec4<f32>;
    var rightPixel_1: vec4<f32>;
    var bottomDistance_1: f32;
    var topDistance_1: f32;
    var totalDistance_2: f32;
    var bottomPixel_1: vec4<f32>;
    var topPixel_1: vec4<f32>;

    let _e27 = global.leftEdge;
    let _e28 = global.rightEdge;
    if (_e27 < _e28) {
        let _e30 = global.leftEdge;
        local = _e30;
    } else {
        let _e31 = global.rightEdge;
        local = _e31;
    }
    let _e33 = local;
    realLeftEdge = _e33;
    let _e35 = global.leftEdge;
    let _e36 = global.rightEdge;
    if (_e35 > _e36) {
        let _e38 = global.leftEdge;
        local_1 = _e38;
    } else {
        let _e39 = global.rightEdge;
        local_1 = _e39;
    }
    let _e41 = local_1;
    realRightEdge = _e41;
    let _e43 = global.bottomEdge;
    let _e44 = global.topEdge;
    if (_e43 < _e44) {
        let _e46 = global.bottomEdge;
        local_2 = _e46;
    } else {
        let _e47 = global.topEdge;
        local_2 = _e47;
    }
    let _e49 = local_2;
    realBottomEdge = _e49;
    let _e51 = global.bottomEdge;
    let _e52 = global.topEdge;
    if (_e51 > _e52) {
        let _e54 = global.bottomEdge;
        local_3 = _e54;
    } else {
        let _e55 = global.topEdge;
        local_3 = _e55;
    }
    let _e57 = local_3;
    realTopEdge = _e57;
    let _e59 = isf_FragNormCoord_1;
    sampleCoord = _e59.xy;
    let _e68 = sampleCoord;
    let _e70 = realLeftEdge;
    if (_e68.x < _e70) {
        {
            region.x = 0f;
        }
    } else {
        let _e74 = sampleCoord;
        let _e76 = realRightEdge;
        if (_e74.x > _e76) {
            {
                region.x = 2f;
            }
        } else {
            {
                region.x = 1f;
            }
        }
    }
    let _e82 = sampleCoord;
    let _e84 = realBottomEdge;
    if (_e82.y < _e84) {
        {
            region.y = 0f;
        }
    } else {
        let _e88 = sampleCoord;
        let _e90 = realTopEdge;
        if (_e88.y > _e90) {
            {
                region.y = 2f;
            }
        } else {
            {
                region.y = 1f;
            }
        }
    }
    let _e96 = region;
    let _e100 = region;
    let _e105 = global.insideBleed_on;
    if (((_e96.x == 1f) && (_e100.y == 1f)) && ((_e105 != 0i) == true)) {
        {
            insideBox = true;
        }
    } else {
        let _e112 = global.outsideBleed_on;
        if ((_e112 != 0i) == true) {
            {
                let _e117 = region;
                let _e121 = region;
                if ((_e117.x == 0f) && (_e121.y == 0f)) {
                    {
                        let _e126 = global.doHorizontal_on;
                        let _e129 = global.doVertical_on;
                        if ((_e126 != 0i) && (_e129 != 0i)) {
                            {
                                insideBox = true;
                                let _e134 = realLeftEdge;
                                realRightEdge = _e134;
                                realLeftEdge = 0f;
                                let _e136 = realBottomEdge;
                                realTopEdge = _e136;
                                realBottomEdge = 0f;
                            }
                        }
                    }
                } else {
                    let _e138 = region;
                    let _e142 = region;
                    if ((_e138.x == 1f) && (_e142.y == 0f)) {
                        {
                            let _e147 = global.doHorizontal_on;
                            let _e150 = global.doVertical_on;
                            if ((_e147 != 0i) && (_e150 != 0i)) {
                                {
                                    insideBox = true;
                                    let _e155 = realBottomEdge;
                                    realTopEdge = _e155;
                                    realBottomEdge = 0f;
                                }
                            } else {
                                let _e157 = global.doVertical_on;
                                if (_e157 != 0i) {
                                    {
                                        insideBox = true;
                                        let _e161 = realBottomEdge;
                                        realTopEdge = _e161;
                                        realBottomEdge = 0f;
                                    }
                                }
                            }
                        }
                    } else {
                        let _e163 = region;
                        let _e167 = region;
                        if ((_e163.x == 2f) && (_e167.y == 0f)) {
                            {
                                let _e172 = global.doHorizontal_on;
                                let _e175 = global.doVertical_on;
                                if ((_e172 != 0i) && (_e175 != 0i)) {
                                    {
                                        insideBox = true;
                                        let _e180 = realRightEdge;
                                        realLeftEdge = _e180;
                                        realRightEdge = 1f;
                                        let _e182 = realBottomEdge;
                                        realTopEdge = _e182;
                                        realBottomEdge = 0f;
                                    }
                                }
                            }
                        } else {
                            let _e184 = region;
                            let _e188 = region;
                            if ((_e184.x == 0f) && (_e188.y == 1f)) {
                                {
                                    let _e193 = global.doHorizontal_on;
                                    if (_e193 != 0i) {
                                        {
                                            insideBox = true;
                                            let _e197 = realLeftEdge;
                                            realRightEdge = _e197;
                                            realLeftEdge = 0f;
                                        }
                                    }
                                }
                            } else {
                                let _e199 = region;
                                let _e203 = region;
                                if ((_e199.x == 2f) && (_e203.y == 1f)) {
                                    {
                                        let _e208 = global.doHorizontal_on;
                                        if (_e208 != 0i) {
                                            {
                                                insideBox = true;
                                                let _e212 = realRightEdge;
                                                realLeftEdge = _e212;
                                                realRightEdge = 1f;
                                            }
                                        }
                                    }
                                } else {
                                    let _e214 = region;
                                    let _e218 = region;
                                    if ((_e214.x == 0f) && (_e218.y == 2f)) {
                                        {
                                            let _e223 = global.doHorizontal_on;
                                            let _e226 = global.doVertical_on;
                                            if ((_e223 != 0i) && (_e226 != 0i)) {
                                                {
                                                    insideBox = true;
                                                    let _e231 = realLeftEdge;
                                                    realRightEdge = _e231;
                                                    realLeftEdge = 0f;
                                                    let _e233 = realTopEdge;
                                                    realBottomEdge = _e233;
                                                    realTopEdge = 1f;
                                                }
                                            }
                                        }
                                    } else {
                                        let _e235 = region;
                                        let _e239 = region;
                                        if ((_e235.x == 1f) && (_e239.y == 2f)) {
                                            {
                                                let _e244 = global.doVertical_on;
                                                if (_e244 != 0i) {
                                                    {
                                                        insideBox = true;
                                                        let _e248 = realTopEdge;
                                                        realBottomEdge = _e248;
                                                        realTopEdge = 1f;
                                                    }
                                                }
                                            }
                                        } else {
                                            let _e250 = region;
                                            let _e254 = region;
                                            if ((_e250.x == 2f) && (_e254.y == 2f)) {
                                                {
                                                    let _e259 = global.doHorizontal_on;
                                                    let _e262 = global.doVertical_on;
                                                    if ((_e259 != 0i) && (_e262 != 0i)) {
                                                        {
                                                            insideBox = true;
                                                            let _e267 = realRightEdge;
                                                            realLeftEdge = _e267;
                                                            realRightEdge = 1f;
                                                            let _e269 = realTopEdge;
                                                            realBottomEdge = _e269;
                                                            realTopEdge = 1f;
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
                }
            }
        }
    }
    let _e271 = insideBox;
    if _e271 {
        {
            let _e272 = global.doHorizontal_on;
            let _e275 = global.doVertical_on;
            if ((_e272 != 0i) && (_e275 != 0i)) {
                {
                    let _e279 = sampleCoord;
                    let _e281 = realLeftEdge;
                    leftDistance = (_e279.x - _e281);
                    let _e284 = realRightEdge;
                    let _e285 = sampleCoord;
                    rightDistance = (_e284 - _e285.x);
                    let _e289 = sampleCoord;
                    let _e291 = realBottomEdge;
                    bottomDistance = (_e289.y - _e291);
                    let _e294 = realTopEdge;
                    let _e295 = sampleCoord;
                    topDistance = (_e294 - _e295.y);
                    let _e299 = leftDistance;
                    let _e300 = rightDistance;
                    let _e302 = bottomDistance;
                    let _e304 = topDistance;
                    totalDistance = (((_e299 + _e300) + _e302) + _e304);
                    let _e307 = realLeftEdge;
                    let _e308 = sampleCoord;
                    let _e312 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e307, _e308.y), 0f);
                    leftPixel = _e312;
                    let _e314 = realRightEdge;
                    let _e315 = sampleCoord;
                    let _e319 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e314, _e315.y), 0f);
                    rightPixel = _e319;
                    let _e321 = sampleCoord;
                    let _e323 = realBottomEdge;
                    let _e326 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e321.x, _e323), 0f);
                    bottomPixel = _e326;
                    let _e328 = sampleCoord;
                    let _e330 = realTopEdge;
                    let _e333 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e328.x, _e330), 0f);
                    topPixel = _e333;
                    let _e335 = rightDistance;
                    let _e336 = leftPixel;
                    let _e338 = leftDistance;
                    let _e339 = rightPixel;
                    let _e342 = topDistance;
                    let _e343 = bottomPixel;
                    let _e346 = bottomDistance;
                    let _e347 = topPixel;
                    let _e350 = totalDistance;
                    outputPixelColor = (((((_e335 * _e336) + (_e338 * _e339)) + (_e342 * _e343)) + (_e346 * _e347)) / vec4(_e350));
                }
            } else {
                let _e353 = global.doHorizontal_on;
                if (_e353 != 0i) {
                    {
                        let _e356 = sampleCoord;
                        let _e358 = realLeftEdge;
                        leftDistance_1 = (_e356.x - _e358);
                        let _e361 = realRightEdge;
                        let _e362 = sampleCoord;
                        rightDistance_1 = (_e361 - _e362.x);
                        let _e366 = leftDistance_1;
                        let _e367 = rightDistance_1;
                        totalDistance_1 = (_e366 + _e367);
                        let _e370 = realLeftEdge;
                        let _e371 = sampleCoord;
                        let _e375 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e370, _e371.y), 0f);
                        leftPixel_1 = _e375;
                        let _e377 = realRightEdge;
                        let _e378 = sampleCoord;
                        let _e382 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e377, _e378.y), 0f);
                        rightPixel_1 = _e382;
                        let _e384 = rightDistance_1;
                        let _e385 = leftPixel_1;
                        let _e387 = leftDistance_1;
                        let _e388 = rightPixel_1;
                        let _e391 = totalDistance_1;
                        outputPixelColor = (((_e384 * _e385) + (_e387 * _e388)) / vec4(_e391));
                    }
                } else {
                    let _e394 = global.doVertical_on;
                    if (_e394 != 0i) {
                        {
                            let _e397 = sampleCoord;
                            let _e399 = realBottomEdge;
                            bottomDistance_1 = (_e397.y - _e399);
                            let _e402 = realTopEdge;
                            let _e403 = sampleCoord;
                            topDistance_1 = (_e402 - _e403.y);
                            let _e407 = bottomDistance_1;
                            let _e408 = topDistance_1;
                            totalDistance_2 = (_e407 + _e408);
                            let _e411 = sampleCoord;
                            let _e413 = realBottomEdge;
                            let _e416 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e411.x, _e413), 0f);
                            bottomPixel_1 = _e416;
                            let _e418 = sampleCoord;
                            let _e420 = realTopEdge;
                            let _e423 = textureSampleLevel(isf_picture, isf_sampler, vec2<f32>(_e418.x, _e420), 0f);
                            topPixel_1 = _e423;
                            let _e425 = topDistance_1;
                            let _e426 = bottomPixel_1;
                            let _e428 = bottomDistance_1;
                            let _e429 = topPixel_1;
                            let _e432 = totalDistance_2;
                            outputPixelColor = (((_e425 * _e426) + (_e428 * _e429)) / vec4(_e432));
                        }
                    } else {
                        {
                            let _e435 = sampleCoord;
                            let _e437 = textureSampleLevel(isf_picture, isf_sampler, _e435, 0f);
                            outputPixelColor = _e437;
                        }
                    }
                }
            }
        }
    } else {
        {
            let _e438 = sampleCoord;
            let _e440 = textureSampleLevel(isf_picture, isf_sampler, _e438, 0f);
            outputPixelColor = _e440;
        }
    }
    let _e441 = outputPixelColor;
    isf_FragColor = _e441;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e33 = isf_FragColor;
    return FragmentOutput(_e33);
}
