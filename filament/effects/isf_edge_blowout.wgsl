// Edge Blowout: ISF-Files' Edge Blowout.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Light
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
    passindex: f32,
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
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
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

    let _e29 = global.leftEdge;
    let _e30 = global.rightEdge;
    if (_e29 < _e30) {
        let _e32 = global.leftEdge;
        local = _e32;
    } else {
        let _e33 = global.rightEdge;
        local = _e33;
    }
    let _e35 = local;
    realLeftEdge = _e35;
    let _e37 = global.leftEdge;
    let _e38 = global.rightEdge;
    if (_e37 > _e38) {
        let _e40 = global.leftEdge;
        local_1 = _e40;
    } else {
        let _e41 = global.rightEdge;
        local_1 = _e41;
    }
    let _e43 = local_1;
    realRightEdge = _e43;
    let _e45 = global.bottomEdge;
    let _e46 = global.topEdge;
    if (_e45 < _e46) {
        let _e48 = global.bottomEdge;
        local_2 = _e48;
    } else {
        let _e49 = global.topEdge;
        local_2 = _e49;
    }
    let _e51 = local_2;
    realBottomEdge = _e51;
    let _e53 = global.bottomEdge;
    let _e54 = global.topEdge;
    if (_e53 > _e54) {
        let _e56 = global.bottomEdge;
        local_3 = _e56;
    } else {
        let _e57 = global.topEdge;
        local_3 = _e57;
    }
    let _e59 = local_3;
    realTopEdge = _e59;
    let _e61 = isf_FragNormCoord_1;
    sampleCoord = _e61.xy;
    let _e70 = sampleCoord;
    let _e72 = realLeftEdge;
    if (_e70.x < _e72) {
        {
            region.x = 0f;
        }
    } else {
        let _e76 = sampleCoord;
        let _e78 = realRightEdge;
        if (_e76.x > _e78) {
            {
                region.x = 2f;
            }
        } else {
            {
                region.x = 1f;
            }
        }
    }
    let _e84 = sampleCoord;
    let _e86 = realBottomEdge;
    if (_e84.y < _e86) {
        {
            region.y = 0f;
        }
    } else {
        let _e90 = sampleCoord;
        let _e92 = realTopEdge;
        if (_e90.y > _e92) {
            {
                region.y = 2f;
            }
        } else {
            {
                region.y = 1f;
            }
        }
    }
    let _e98 = region;
    let _e102 = region;
    let _e107 = global.insideBleed_on;
    if (((_e98.x == 1f) && (_e102.y == 1f)) && ((_e107 != 0i) == true)) {
        {
            insideBox = true;
        }
    } else {
        let _e114 = global.outsideBleed_on;
        if ((_e114 != 0i) == true) {
            {
                let _e119 = region;
                let _e123 = region;
                if ((_e119.x == 0f) && (_e123.y == 0f)) {
                    {
                        let _e128 = global.doHorizontal_on;
                        let _e131 = global.doVertical_on;
                        if ((_e128 != 0i) && (_e131 != 0i)) {
                            {
                                insideBox = true;
                                let _e136 = realLeftEdge;
                                realRightEdge = _e136;
                                realLeftEdge = 0f;
                                let _e138 = realBottomEdge;
                                realTopEdge = _e138;
                                realBottomEdge = 0f;
                            }
                        }
                    }
                } else {
                    let _e140 = region;
                    let _e144 = region;
                    if ((_e140.x == 1f) && (_e144.y == 0f)) {
                        {
                            let _e149 = global.doHorizontal_on;
                            let _e152 = global.doVertical_on;
                            if ((_e149 != 0i) && (_e152 != 0i)) {
                                {
                                    insideBox = true;
                                    let _e157 = realBottomEdge;
                                    realTopEdge = _e157;
                                    realBottomEdge = 0f;
                                }
                            } else {
                                let _e159 = global.doVertical_on;
                                if (_e159 != 0i) {
                                    {
                                        insideBox = true;
                                        let _e163 = realBottomEdge;
                                        realTopEdge = _e163;
                                        realBottomEdge = 0f;
                                    }
                                }
                            }
                        }
                    } else {
                        let _e165 = region;
                        let _e169 = region;
                        if ((_e165.x == 2f) && (_e169.y == 0f)) {
                            {
                                let _e174 = global.doHorizontal_on;
                                let _e177 = global.doVertical_on;
                                if ((_e174 != 0i) && (_e177 != 0i)) {
                                    {
                                        insideBox = true;
                                        let _e182 = realRightEdge;
                                        realLeftEdge = _e182;
                                        realRightEdge = 1f;
                                        let _e184 = realBottomEdge;
                                        realTopEdge = _e184;
                                        realBottomEdge = 0f;
                                    }
                                }
                            }
                        } else {
                            let _e186 = region;
                            let _e190 = region;
                            if ((_e186.x == 0f) && (_e190.y == 1f)) {
                                {
                                    let _e195 = global.doHorizontal_on;
                                    if (_e195 != 0i) {
                                        {
                                            insideBox = true;
                                            let _e199 = realLeftEdge;
                                            realRightEdge = _e199;
                                            realLeftEdge = 0f;
                                        }
                                    }
                                }
                            } else {
                                let _e201 = region;
                                let _e205 = region;
                                if ((_e201.x == 2f) && (_e205.y == 1f)) {
                                    {
                                        let _e210 = global.doHorizontal_on;
                                        if (_e210 != 0i) {
                                            {
                                                insideBox = true;
                                                let _e214 = realRightEdge;
                                                realLeftEdge = _e214;
                                                realRightEdge = 1f;
                                            }
                                        }
                                    }
                                } else {
                                    let _e216 = region;
                                    let _e220 = region;
                                    if ((_e216.x == 0f) && (_e220.y == 2f)) {
                                        {
                                            let _e225 = global.doHorizontal_on;
                                            let _e228 = global.doVertical_on;
                                            if ((_e225 != 0i) && (_e228 != 0i)) {
                                                {
                                                    insideBox = true;
                                                    let _e233 = realLeftEdge;
                                                    realRightEdge = _e233;
                                                    realLeftEdge = 0f;
                                                    let _e235 = realTopEdge;
                                                    realBottomEdge = _e235;
                                                    realTopEdge = 1f;
                                                }
                                            }
                                        }
                                    } else {
                                        let _e237 = region;
                                        let _e241 = region;
                                        if ((_e237.x == 1f) && (_e241.y == 2f)) {
                                            {
                                                let _e246 = global.doVertical_on;
                                                if (_e246 != 0i) {
                                                    {
                                                        insideBox = true;
                                                        let _e250 = realTopEdge;
                                                        realBottomEdge = _e250;
                                                        realTopEdge = 1f;
                                                    }
                                                }
                                            }
                                        } else {
                                            let _e252 = region;
                                            let _e256 = region;
                                            if ((_e252.x == 2f) && (_e256.y == 2f)) {
                                                {
                                                    let _e261 = global.doHorizontal_on;
                                                    let _e264 = global.doVertical_on;
                                                    if ((_e261 != 0i) && (_e264 != 0i)) {
                                                        {
                                                            insideBox = true;
                                                            let _e269 = realRightEdge;
                                                            realLeftEdge = _e269;
                                                            realRightEdge = 1f;
                                                            let _e271 = realTopEdge;
                                                            realBottomEdge = _e271;
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
    let _e273 = insideBox;
    if _e273 {
        {
            let _e274 = global.doHorizontal_on;
            let _e277 = global.doVertical_on;
            if ((_e274 != 0i) && (_e277 != 0i)) {
                {
                    let _e281 = sampleCoord;
                    let _e283 = realLeftEdge;
                    leftDistance = (_e281.x - _e283);
                    let _e286 = realRightEdge;
                    let _e287 = sampleCoord;
                    rightDistance = (_e286 - _e287.x);
                    let _e291 = sampleCoord;
                    let _e293 = realBottomEdge;
                    bottomDistance = (_e291.y - _e293);
                    let _e296 = realTopEdge;
                    let _e297 = sampleCoord;
                    topDistance = (_e296 - _e297.y);
                    let _e301 = leftDistance;
                    let _e302 = rightDistance;
                    let _e304 = bottomDistance;
                    let _e306 = topDistance;
                    totalDistance = (((_e301 + _e302) + _e304) + _e306);
                    let _e309 = realLeftEdge;
                    let _e310 = sampleCoord;
                    let _e314 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e309, _e310.y), 0f);
                    leftPixel = _e314;
                    let _e316 = realRightEdge;
                    let _e317 = sampleCoord;
                    let _e321 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e316, _e317.y), 0f);
                    rightPixel = _e321;
                    let _e323 = sampleCoord;
                    let _e325 = realBottomEdge;
                    let _e328 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e323.x, _e325), 0f);
                    bottomPixel = _e328;
                    let _e330 = sampleCoord;
                    let _e332 = realTopEdge;
                    let _e335 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e330.x, _e332), 0f);
                    topPixel = _e335;
                    let _e337 = rightDistance;
                    let _e338 = leftPixel;
                    let _e340 = leftDistance;
                    let _e341 = rightPixel;
                    let _e344 = topDistance;
                    let _e345 = bottomPixel;
                    let _e348 = bottomDistance;
                    let _e349 = topPixel;
                    let _e352 = totalDistance;
                    outputPixelColor = (((((_e337 * _e338) + (_e340 * _e341)) + (_e344 * _e345)) + (_e348 * _e349)) / vec4(_e352));
                }
            } else {
                let _e355 = global.doHorizontal_on;
                if (_e355 != 0i) {
                    {
                        let _e358 = sampleCoord;
                        let _e360 = realLeftEdge;
                        leftDistance_1 = (_e358.x - _e360);
                        let _e363 = realRightEdge;
                        let _e364 = sampleCoord;
                        rightDistance_1 = (_e363 - _e364.x);
                        let _e368 = leftDistance_1;
                        let _e369 = rightDistance_1;
                        totalDistance_1 = (_e368 + _e369);
                        let _e372 = realLeftEdge;
                        let _e373 = sampleCoord;
                        let _e377 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e372, _e373.y), 0f);
                        leftPixel_1 = _e377;
                        let _e379 = realRightEdge;
                        let _e380 = sampleCoord;
                        let _e384 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e379, _e380.y), 0f);
                        rightPixel_1 = _e384;
                        let _e386 = rightDistance_1;
                        let _e387 = leftPixel_1;
                        let _e389 = leftDistance_1;
                        let _e390 = rightPixel_1;
                        let _e393 = totalDistance_1;
                        outputPixelColor = (((_e386 * _e387) + (_e389 * _e390)) / vec4(_e393));
                    }
                } else {
                    let _e396 = global.doVertical_on;
                    if (_e396 != 0i) {
                        {
                            let _e399 = sampleCoord;
                            let _e401 = realBottomEdge;
                            bottomDistance_1 = (_e399.y - _e401);
                            let _e404 = realTopEdge;
                            let _e405 = sampleCoord;
                            topDistance_1 = (_e404 - _e405.y);
                            let _e409 = bottomDistance_1;
                            let _e410 = topDistance_1;
                            totalDistance_2 = (_e409 + _e410);
                            let _e413 = sampleCoord;
                            let _e415 = realBottomEdge;
                            let _e418 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e413.x, _e415), 0f);
                            bottomPixel_1 = _e418;
                            let _e420 = sampleCoord;
                            let _e422 = realTopEdge;
                            let _e425 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, vec2<f32>(_e420.x, _e422), 0f);
                            topPixel_1 = _e425;
                            let _e427 = topDistance_1;
                            let _e428 = bottomPixel_1;
                            let _e430 = bottomDistance_1;
                            let _e431 = topPixel_1;
                            let _e434 = totalDistance_2;
                            outputPixelColor = (((_e427 * _e428) + (_e430 * _e431)) / vec4(_e434));
                        }
                    } else {
                        {
                            let _e437 = sampleCoord;
                            let _e439 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e437, 0f);
                            outputPixelColor = _e439;
                        }
                    }
                }
            }
        }
    } else {
        {
            let _e440 = sampleCoord;
            let _e442 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e440, 0f);
            outputPixelColor = _e442;
        }
    }
    let _e443 = outputPixelColor;
    isf_FragColor = _e443;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e35 = isf_FragColor;
    return FragmentOutput(_e35);
}
