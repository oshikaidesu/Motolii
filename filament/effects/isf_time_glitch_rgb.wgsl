// Time Glitch RGB: ISF-Files' Time Glitch RGB.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@inputDelay_x:Buffer r,0,1,0.25,0.001
// --track@inputDelay_y:Buffer g,0,1,0.5,0.001
// --track@inputDelay_z:Buffer b,0,1,0.75,0.001
// --track@inputDelay_w:Buffer a,0,1,0.5,0.001
// --track@inputRate:Buffer Lag,1.0,20.0,4.0,0.001
// --track@glitch_size:Size,0.0,0.5,0.1,0.001
// --track@glitch_horizontal:Horizontal Amount,0.0,1.0,0.2,0.001
// --track@glitch_vertical:Vertical Amount,0.0,1.0,0.0,0.001
// --check@randomize_size_on:Randomize Size,1
// --check@randomize_position_on:Randomize Position,0
// --check@randomize_zoom_on:Randomize Zoom,0
// --pass@lastRow:1,0,a1,a1
// --pass@buffer8:1,0,s1,s1
// --pass@buffer7:1,0,s1,s1
// --pass@buffer6:1,0,s1,s1
// --pass@buffer5:1,0,s1,s1
// --pass@buffer4:1,0,s1,s1
// --pass@buffer3:1,0,s1,s1
// --pass@buffer2:1,0,s1,s1
// --pass@buffer1:1,0,s1,s1
// --pass@:0,0,s1,s1

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    inputDelay: vec4<f32>,
    inputRate: f32,
    glitch_size: f32,
    glitch_horizontal: f32,
    glitch_vertical: f32,
    randomize_size_on: i32,
    randomize_position_on: i32,
    randomize_zoom_on: i32,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

@group(0) @binding(0) 
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
@group(0) @binding(2) 
var isf_tex_lastRow: texture_2d<f32>;
@group(0) @binding(3) 
var isf_smp_lastRow: sampler;
@group(0) @binding(4) 
var isf_tex_buffer8_: texture_2d<f32>;
@group(0) @binding(5) 
var isf_smp_buffer8_: sampler;
@group(0) @binding(6) 
var isf_tex_buffer7_: texture_2d<f32>;
@group(0) @binding(7) 
var isf_smp_buffer7_: sampler;
@group(0) @binding(8) 
var isf_tex_buffer6_: texture_2d<f32>;
@group(0) @binding(9) 
var isf_smp_buffer6_: sampler;
@group(0) @binding(10) 
var isf_tex_buffer5_: texture_2d<f32>;
@group(0) @binding(11) 
var isf_smp_buffer5_: sampler;
@group(0) @binding(12) 
var isf_tex_buffer4_: texture_2d<f32>;
@group(0) @binding(13) 
var isf_smp_buffer4_: sampler;
@group(0) @binding(14) 
var isf_tex_buffer3_: texture_2d<f32>;
@group(0) @binding(15) 
var isf_smp_buffer3_: sampler;
@group(0) @binding(16) 
var isf_tex_buffer2_: texture_2d<f32>;
@group(0) @binding(17) 
var isf_smp_buffer2_: sampler;
@group(0) @binding(18) 
var isf_tex_buffer1_: texture_2d<f32>;
@group(0) @binding(19) 
var isf_smp_buffer1_: sampler;
@group(0) @binding(20) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;

fn rand(co: vec2<f32>) -> f32 {
    var co_1: vec2<f32>;

    co_1 = co;
    let _e48 = co_1;
    return fract((sin(dot(_e48.xy, vec2<f32>(12.9898f, 78.233f))) * 43758.547f));
}

fn main_1() {
    var srcPixel: vec4<f32>;
    var local: f32;
    var lastRow: vec4<f32>;
    var lastRow_1: vec4<f32>;
    var lastRow_2: vec4<f32>;
    var lastRow_3: vec4<f32>;
    var lastRow_4: vec4<f32>;
    var lastRow_5: vec4<f32>;
    var lastRow_6: vec4<f32>;
    var lastRow_7: vec4<f32>;
    var tex: vec2<f32>;
    var color: vec4<f32> = vec4(0f);
    var randomDelayShift: f32 = 0f;
    var xy: vec2<f32>;
    var random: vec2<f32>;
    var local_glitch_size: f32;
    var random_offset: f32 = 0f;
    var pixelBuffer: vec4<f32>;
    var level: f32;
    var level_1: f32;
    var level_2: f32;

    let _e46 = global.passindex;
    if (i32(_e46) == 0i) {
        {
            let _e52 = global.resolution;
            let _e55 = textureSampleLevel(isf_tex_lastRow, isf_smp_lastRow, (vec2(0.5f) / _e52), 0f);
            srcPixel = _e55;
            let _e57 = global.inputRate;
            if (_e57 == 0f) {
                {
                    srcPixel.x = 0f;
                    srcPixel.y = 0f;
                }
            } else {
                let _e64 = global.inputRate;
                if (_e64 <= 1f) {
                    {
                        let _e68 = srcPixel;
                        if (_e68.x > 0.5f) {
                            local = 0f;
                        } else {
                            local = 1f;
                        }
                        let _e75 = local;
                        srcPixel.x = _e75;
                        srcPixel.y = 0f;
                    }
                } else {
                    {
                        let _e79 = srcPixel;
                        let _e82 = global.inputRate;
                        let _e85 = srcPixel;
                        srcPixel.x = ((_e79.x + (1f / _e82)) + _e85.y);
                        let _e88 = srcPixel;
                        if (_e88.x > 1f) {
                            {
                                let _e93 = srcPixel;
                                srcPixel.y = (_e93.x - (floor((_e93.x / 1f)) * 1f));
                                srcPixel.x = 0f;
                            }
                        }
                    }
                }
            }
            let _e102 = srcPixel;
            isf_FragColor = _e102;
        }
    }
    let _e103 = global.passindex;
    if (i32(_e103) == 1i) {
        {
            let _e109 = global.resolution;
            let _e112 = textureSampleLevel(isf_tex_lastRow, isf_smp_lastRow, (vec2(0.5f) / _e109), 0f);
            lastRow = _e112;
            let _e114 = lastRow;
            if (_e114.x == 0f) {
                {
                    let _e118 = isf_FragNormCoord_1;
                    let _e120 = textureSampleLevel(isf_tex_buffer7_, isf_smp_buffer7_, _e118, 0f);
                    isf_FragColor = _e120;
                    return;
                }
            } else {
                {
                    let _e121 = isf_FragNormCoord_1;
                    let _e123 = textureSampleLevel(isf_tex_buffer8_, isf_smp_buffer8_, _e121, 0f);
                    isf_FragColor = _e123;
                    return;
                }
            }
        }
    } else {
        let _e124 = global.passindex;
        if (i32(_e124) == 2i) {
            {
                let _e130 = global.resolution;
                let _e133 = textureSampleLevel(isf_tex_lastRow, isf_smp_lastRow, (vec2(0.5f) / _e130), 0f);
                lastRow_1 = _e133;
                let _e135 = lastRow_1;
                if (_e135.x == 0f) {
                    {
                        let _e139 = isf_FragNormCoord_1;
                        let _e141 = textureSampleLevel(isf_tex_buffer6_, isf_smp_buffer6_, _e139, 0f);
                        isf_FragColor = _e141;
                        return;
                    }
                } else {
                    {
                        let _e142 = isf_FragNormCoord_1;
                        let _e144 = textureSampleLevel(isf_tex_buffer7_, isf_smp_buffer7_, _e142, 0f);
                        isf_FragColor = _e144;
                        return;
                    }
                }
            }
        } else {
            let _e145 = global.passindex;
            if (i32(_e145) == 3i) {
                {
                    let _e151 = global.resolution;
                    let _e154 = textureSampleLevel(isf_tex_lastRow, isf_smp_lastRow, (vec2(0.5f) / _e151), 0f);
                    lastRow_2 = _e154;
                    let _e156 = lastRow_2;
                    if (_e156.x == 0f) {
                        {
                            let _e160 = isf_FragNormCoord_1;
                            let _e162 = textureSampleLevel(isf_tex_buffer5_, isf_smp_buffer5_, _e160, 0f);
                            isf_FragColor = _e162;
                            return;
                        }
                    } else {
                        {
                            let _e163 = isf_FragNormCoord_1;
                            let _e165 = textureSampleLevel(isf_tex_buffer6_, isf_smp_buffer6_, _e163, 0f);
                            isf_FragColor = _e165;
                            return;
                        }
                    }
                }
            } else {
                let _e166 = global.passindex;
                if (i32(_e166) == 4i) {
                    {
                        let _e172 = global.resolution;
                        let _e175 = textureSampleLevel(isf_tex_lastRow, isf_smp_lastRow, (vec2(0.5f) / _e172), 0f);
                        lastRow_3 = _e175;
                        let _e177 = lastRow_3;
                        if (_e177.x == 0f) {
                            {
                                let _e181 = isf_FragNormCoord_1;
                                let _e183 = textureSampleLevel(isf_tex_buffer4_, isf_smp_buffer4_, _e181, 0f);
                                isf_FragColor = _e183;
                                return;
                            }
                        } else {
                            {
                                let _e184 = isf_FragNormCoord_1;
                                let _e186 = textureSampleLevel(isf_tex_buffer5_, isf_smp_buffer5_, _e184, 0f);
                                isf_FragColor = _e186;
                                return;
                            }
                        }
                    }
                } else {
                    let _e187 = global.passindex;
                    if (i32(_e187) == 5i) {
                        {
                            let _e193 = global.resolution;
                            let _e196 = textureSampleLevel(isf_tex_lastRow, isf_smp_lastRow, (vec2(0.5f) / _e193), 0f);
                            lastRow_4 = _e196;
                            let _e198 = lastRow_4;
                            if (_e198.x == 0f) {
                                {
                                    let _e202 = isf_FragNormCoord_1;
                                    let _e204 = textureSampleLevel(isf_tex_buffer3_, isf_smp_buffer3_, _e202, 0f);
                                    isf_FragColor = _e204;
                                    return;
                                }
                            } else {
                                {
                                    let _e205 = isf_FragNormCoord_1;
                                    let _e207 = textureSampleLevel(isf_tex_buffer4_, isf_smp_buffer4_, _e205, 0f);
                                    isf_FragColor = _e207;
                                    return;
                                }
                            }
                        }
                    } else {
                        let _e208 = global.passindex;
                        if (i32(_e208) == 6i) {
                            {
                                let _e214 = global.resolution;
                                let _e217 = textureSampleLevel(isf_tex_lastRow, isf_smp_lastRow, (vec2(0.5f) / _e214), 0f);
                                lastRow_5 = _e217;
                                let _e219 = lastRow_5;
                                if (_e219.x == 0f) {
                                    {
                                        let _e223 = isf_FragNormCoord_1;
                                        let _e225 = textureSampleLevel(isf_tex_buffer2_, isf_smp_buffer2_, _e223, 0f);
                                        isf_FragColor = _e225;
                                        return;
                                    }
                                } else {
                                    {
                                        let _e226 = isf_FragNormCoord_1;
                                        let _e228 = textureSampleLevel(isf_tex_buffer3_, isf_smp_buffer3_, _e226, 0f);
                                        isf_FragColor = _e228;
                                        return;
                                    }
                                }
                            }
                        } else {
                            let _e229 = global.passindex;
                            if (i32(_e229) == 7i) {
                                {
                                    let _e235 = global.resolution;
                                    let _e238 = textureSampleLevel(isf_tex_lastRow, isf_smp_lastRow, (vec2(0.5f) / _e235), 0f);
                                    lastRow_6 = _e238;
                                    let _e240 = lastRow_6;
                                    if (_e240.x == 0f) {
                                        {
                                            let _e244 = isf_FragNormCoord_1;
                                            let _e246 = textureSampleLevel(isf_tex_buffer1_, isf_smp_buffer1_, _e244, 0f);
                                            isf_FragColor = _e246;
                                            return;
                                        }
                                    } else {
                                        {
                                            let _e247 = isf_FragNormCoord_1;
                                            let _e249 = textureSampleLevel(isf_tex_buffer2_, isf_smp_buffer2_, _e247, 0f);
                                            isf_FragColor = _e249;
                                            return;
                                        }
                                    }
                                }
                            } else {
                                let _e250 = global.passindex;
                                if (i32(_e250) == 8i) {
                                    {
                                        let _e256 = global.resolution;
                                        let _e259 = textureSampleLevel(isf_tex_lastRow, isf_smp_lastRow, (vec2(0.5f) / _e256), 0f);
                                        lastRow_7 = _e259;
                                        let _e261 = lastRow_7;
                                        if (_e261.x == 0f) {
                                            {
                                                let _e265 = isf_FragNormCoord_1;
                                                let _e267 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e265, 0f);
                                                isf_FragColor = _e267;
                                                return;
                                            }
                                        } else {
                                            {
                                                let _e268 = isf_FragNormCoord_1;
                                                let _e270 = textureSampleLevel(isf_tex_buffer1_, isf_smp_buffer1_, _e268, 0f);
                                                isf_FragColor = _e270;
                                                return;
                                            }
                                        }
                                    }
                                } else {
                                    let _e271 = global.passindex;
                                    if (i32(_e271) == 9i) {
                                        {
                                            let _e275 = isf_FragNormCoord_1;
                                            tex = _e275;
                                            let _e286 = isf_FragNormCoord_1.x;
                                            xy.x = _e286;
                                            let _e290 = isf_FragNormCoord_1.y;
                                            xy.y = _e290;
                                            let _e292 = global.glitch_size;
                                            local_glitch_size = _e292;
                                            let _e296 = global.randomize_size_on;
                                            if (_e296 != 0i) {
                                                {
                                                    let _e299 = global.time;
                                                    let _e300 = global.time;
                                                    let _e302 = rand(vec2<f32>(_e299, _e300));
                                                    random_offset = (_e302 - (floor((_e302 / 1f)) * 1f));
                                                    let _e308 = random_offset;
                                                    let _e309 = global.glitch_size;
                                                    local_glitch_size = (_e308 * _e309);
                                                }
                                            }
                                            let _e311 = local_glitch_size;
                                            if (_e311 > 0f) {
                                                {
                                                    let _e315 = random_offset;
                                                    let _e316 = xy;
                                                    let _e318 = local_glitch_size;
                                                    let _e322 = local_glitch_size;
                                                    let _e324 = global.time;
                                                    let _e326 = rand(vec2<f32>((floor((_e315 + (_e316.y / _e318))) * _e322), _e324));
                                                    random.x = _e326;
                                                    let _e328 = random_offset;
                                                    let _e329 = xy;
                                                    let _e331 = local_glitch_size;
                                                    let _e335 = local_glitch_size;
                                                    let _e337 = global.time;
                                                    let _e339 = rand(vec2<f32>((floor((_e328 + (_e329.x / _e331))) * _e335), _e337));
                                                    random.y = _e339;
                                                }
                                            } else {
                                                {
                                                    let _e341 = xy;
                                                    let _e343 = global.time;
                                                    let _e345 = rand(vec2<f32>(_e341.x, _e343));
                                                    random.x = _e345;
                                                    let _e347 = xy;
                                                    let _e349 = global.time;
                                                    let _e351 = rand(vec2<f32>(_e347.y, _e349));
                                                    random.y = _e351;
                                                }
                                            }
                                            let _e352 = random;
                                            let _e354 = global.glitch_horizontal;
                                            let _e356 = random;
                                            let _e358 = global.glitch_vertical;
                                            if ((_e352.x < _e354) && (_e356.y < _e358)) {
                                                {
                                                    let _e361 = random;
                                                    let _e363 = random;
                                                    randomDelayShift = clamp((_e361.x + _e363.y), 0f, 2f);
                                                }
                                            } else {
                                                let _e369 = random;
                                                let _e371 = global.glitch_horizontal;
                                                if (_e369.x < _e371) {
                                                    {
                                                        let _e373 = random;
                                                        let _e375 = random;
                                                        randomDelayShift = clamp((_e373.x + _e375.y), 0f, 2f);
                                                    }
                                                } else {
                                                    let _e381 = random;
                                                    let _e383 = global.glitch_vertical;
                                                    if (_e381.y < _e383) {
                                                        {
                                                            let _e385 = random;
                                                            let _e387 = random;
                                                            randomDelayShift = clamp((_e385.x + _e387.y), 0f, 2f);
                                                        }
                                                    }
                                                }
                                            }
                                            let _e393 = randomDelayShift;
                                            let _e394 = global.inputDelay;
                                            pixelBuffer = ((_e393 * _e394) * 9f);
                                            let _e399 = global.randomize_zoom_on;
                                            if (_e399 != 0i) {
                                                {
                                                    let _e402 = random;
                                                    let _e404 = global.glitch_horizontal;
                                                    let _e406 = random;
                                                    let _e408 = global.glitch_vertical;
                                                    if ((_e402.x < _e404) && (_e406.y < _e408)) {
                                                        {
                                                            let _e411 = random;
                                                            let _e413 = random;
                                                            level = (((_e411.x + _e413.y) / 3f) + 0.9f);
                                                            let _e421 = tex;
                                                            let _e426 = level;
                                                            tex = (((_e421 - vec2(0.5f)) * (1f / _e426)) + vec2(0.5f));
                                                        }
                                                    } else {
                                                        let _e432 = random;
                                                        let _e434 = global.glitch_horizontal;
                                                        if (_e432.x < _e434) {
                                                            {
                                                                let _e436 = random;
                                                                level_1 = ((_e436.x / 2f) + 0.95f);
                                                                let _e443 = tex;
                                                                let _e448 = level_1;
                                                                tex = (((_e443 - vec2(0.5f)) * (1f / _e448)) + vec2(0.5f));
                                                            }
                                                        } else {
                                                            let _e454 = random;
                                                            let _e456 = global.glitch_vertical;
                                                            if (_e454.y < _e456) {
                                                                {
                                                                    let _e458 = random;
                                                                    level_2 = ((_e458.y / 2f) + 0.95f);
                                                                    let _e465 = tex;
                                                                    let _e470 = level_2;
                                                                    tex = (((_e465 - vec2(0.5f)) * (1f / _e470)) + vec2(0.5f));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            let _e476 = global.randomize_position_on;
                                            if (_e476 != 0i) {
                                                {
                                                    let _e479 = random;
                                                    let _e481 = global.glitch_horizontal;
                                                    let _e483 = random;
                                                    let _e485 = global.glitch_vertical;
                                                    if ((_e479.x < _e481) && (_e483.y < _e485)) {
                                                        {
                                                            let _e489 = tex;
                                                            let _e491 = global.inputDelay;
                                                            let _e493 = random;
                                                            let _e496 = (_e489.x + (_e491.x * _e493.x));
                                                            tex.x = (_e496 - (floor((_e496 / 1f)) * 1f));
                                                            let _e503 = tex;
                                                            let _e505 = global.inputDelay;
                                                            let _e507 = random;
                                                            let _e510 = (_e503.y + (_e505.x * _e507.y));
                                                            tex.y = (_e510 - (floor((_e510 / 1f)) * 1f));
                                                        }
                                                    } else {
                                                        let _e516 = random;
                                                        let _e518 = global.glitch_horizontal;
                                                        if (_e516.x < _e518) {
                                                            {
                                                                let _e521 = tex;
                                                                let _e523 = global.inputDelay;
                                                                let _e525 = random;
                                                                let _e528 = (_e521.y + (_e523.x * _e525.x));
                                                                tex.y = (_e528 - (floor((_e528 / 1f)) * 1f));
                                                            }
                                                        } else {
                                                            let _e534 = random;
                                                            let _e536 = global.glitch_vertical;
                                                            if (_e534.y < _e536) {
                                                                {
                                                                    let _e539 = tex;
                                                                    let _e541 = global.inputDelay;
                                                                    let _e543 = random;
                                                                    let _e546 = (_e539.x + (_e541.x * _e543.y));
                                                                    tex.x = (_e546 - (floor((_e546 / 1f)) * 1f));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            let _e552 = pixelBuffer;
                                            if (_e552.x < 1f) {
                                                {
                                                    let _e557 = tex;
                                                    let _e559 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e557, 0f);
                                                    color.x = _e559.x;
                                                }
                                            } else {
                                                let _e561 = pixelBuffer;
                                                if (_e561.x < 2f) {
                                                    {
                                                        let _e566 = tex;
                                                        let _e568 = textureSampleLevel(isf_tex_buffer1_, isf_smp_buffer1_, _e566, 0f);
                                                        color.x = _e568.x;
                                                    }
                                                } else {
                                                    let _e570 = pixelBuffer;
                                                    if (_e570.x < 3f) {
                                                        {
                                                            let _e575 = tex;
                                                            let _e577 = textureSampleLevel(isf_tex_buffer2_, isf_smp_buffer2_, _e575, 0f);
                                                            color.x = _e577.x;
                                                        }
                                                    } else {
                                                        let _e579 = pixelBuffer;
                                                        if (_e579.x < 4f) {
                                                            {
                                                                let _e584 = tex;
                                                                let _e586 = textureSampleLevel(isf_tex_buffer3_, isf_smp_buffer3_, _e584, 0f);
                                                                color.x = _e586.x;
                                                            }
                                                        } else {
                                                            let _e588 = pixelBuffer;
                                                            if (_e588.x < 5f) {
                                                                {
                                                                    let _e593 = tex;
                                                                    let _e595 = textureSampleLevel(isf_tex_buffer4_, isf_smp_buffer4_, _e593, 0f);
                                                                    color.x = _e595.x;
                                                                }
                                                            } else {
                                                                let _e597 = pixelBuffer;
                                                                if (_e597.x < 6f) {
                                                                    {
                                                                        let _e602 = tex;
                                                                        let _e604 = textureSampleLevel(isf_tex_buffer5_, isf_smp_buffer5_, _e602, 0f);
                                                                        color.x = _e604.x;
                                                                    }
                                                                } else {
                                                                    let _e606 = pixelBuffer;
                                                                    if (_e606.x < 7f) {
                                                                        {
                                                                            let _e611 = tex;
                                                                            let _e613 = textureSampleLevel(isf_tex_buffer6_, isf_smp_buffer6_, _e611, 0f);
                                                                            color.x = _e613.x;
                                                                        }
                                                                    } else {
                                                                        let _e615 = pixelBuffer;
                                                                        if (_e615.x < 8f) {
                                                                            {
                                                                                let _e620 = tex;
                                                                                let _e622 = textureSampleLevel(isf_tex_buffer7_, isf_smp_buffer7_, _e620, 0f);
                                                                                color.x = _e622.x;
                                                                            }
                                                                        } else {
                                                                            {
                                                                                let _e625 = tex;
                                                                                let _e627 = textureSampleLevel(isf_tex_buffer8_, isf_smp_buffer8_, _e625, 0f);
                                                                                color.x = _e627.x;
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            let _e629 = global.randomize_position_on;
                                            if (_e629 != 0i) {
                                                {
                                                    let _e632 = random;
                                                    let _e634 = global.glitch_horizontal;
                                                    let _e636 = random;
                                                    let _e638 = global.glitch_vertical;
                                                    if ((_e632.x < _e634) && (_e636.y < _e638)) {
                                                        {
                                                            let _e642 = tex;
                                                            let _e644 = random;
                                                            let _e646 = global.inputDelay;
                                                            let _e649 = (_e642.x + (_e644.x * _e646.y));
                                                            tex.x = (_e649 - (floor((_e649 / 1f)) * 1f));
                                                            let _e656 = tex;
                                                            let _e658 = random;
                                                            let _e660 = global.inputDelay;
                                                            let _e663 = (_e656.y + (_e658.y * _e660.y));
                                                            tex.y = (_e663 - (floor((_e663 / 1f)) * 1f));
                                                        }
                                                    } else {
                                                        let _e669 = random;
                                                        let _e671 = global.glitch_horizontal;
                                                        if (_e669.x < _e671) {
                                                            {
                                                                let _e674 = tex;
                                                                let _e676 = random;
                                                                let _e678 = global.inputDelay;
                                                                let _e681 = (_e674.y + (_e676.x * _e678.y));
                                                                tex.y = (_e681 - (floor((_e681 / 1f)) * 1f));
                                                            }
                                                        } else {
                                                            let _e687 = random;
                                                            let _e689 = global.glitch_vertical;
                                                            if (_e687.y < _e689) {
                                                                {
                                                                    let _e692 = tex;
                                                                    let _e694 = random;
                                                                    let _e696 = global.inputDelay;
                                                                    let _e699 = (_e692.x + (_e694.y * _e696.y));
                                                                    tex.x = (_e699 - (floor((_e699 / 1f)) * 1f));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            let _e705 = pixelBuffer;
                                            if (_e705.y < 1f) {
                                                {
                                                    let _e710 = tex;
                                                    let _e712 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e710, 0f);
                                                    color.y = _e712.y;
                                                }
                                            } else {
                                                let _e714 = pixelBuffer;
                                                if (_e714.y < 2f) {
                                                    {
                                                        let _e719 = tex;
                                                        let _e721 = textureSampleLevel(isf_tex_buffer1_, isf_smp_buffer1_, _e719, 0f);
                                                        color.y = _e721.y;
                                                    }
                                                } else {
                                                    let _e723 = pixelBuffer;
                                                    if (_e723.y < 3f) {
                                                        {
                                                            let _e728 = tex;
                                                            let _e730 = textureSampleLevel(isf_tex_buffer2_, isf_smp_buffer2_, _e728, 0f);
                                                            color.y = _e730.y;
                                                        }
                                                    } else {
                                                        let _e732 = pixelBuffer;
                                                        if (_e732.y < 4f) {
                                                            {
                                                                let _e737 = tex;
                                                                let _e739 = textureSampleLevel(isf_tex_buffer3_, isf_smp_buffer3_, _e737, 0f);
                                                                color.y = _e739.y;
                                                            }
                                                        } else {
                                                            let _e741 = pixelBuffer;
                                                            if (_e741.y < 5f) {
                                                                {
                                                                    let _e746 = tex;
                                                                    let _e748 = textureSampleLevel(isf_tex_buffer4_, isf_smp_buffer4_, _e746, 0f);
                                                                    color.y = _e748.y;
                                                                }
                                                            } else {
                                                                let _e750 = pixelBuffer;
                                                                if (_e750.y < 6f) {
                                                                    {
                                                                        let _e755 = tex;
                                                                        let _e757 = textureSampleLevel(isf_tex_buffer5_, isf_smp_buffer5_, _e755, 0f);
                                                                        color.y = _e757.y;
                                                                    }
                                                                } else {
                                                                    let _e759 = pixelBuffer;
                                                                    if (_e759.y < 7f) {
                                                                        {
                                                                            let _e764 = tex;
                                                                            let _e766 = textureSampleLevel(isf_tex_buffer6_, isf_smp_buffer6_, _e764, 0f);
                                                                            color.y = _e766.y;
                                                                        }
                                                                    } else {
                                                                        let _e768 = pixelBuffer;
                                                                        if (_e768.y < 8f) {
                                                                            {
                                                                                let _e773 = tex;
                                                                                let _e775 = textureSampleLevel(isf_tex_buffer7_, isf_smp_buffer7_, _e773, 0f);
                                                                                color.y = _e775.y;
                                                                            }
                                                                        } else {
                                                                            {
                                                                                let _e778 = tex;
                                                                                let _e780 = textureSampleLevel(isf_tex_buffer8_, isf_smp_buffer8_, _e778, 0f);
                                                                                color.y = _e780.y;
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            let _e782 = global.randomize_position_on;
                                            if (_e782 != 0i) {
                                                {
                                                    let _e785 = random;
                                                    let _e787 = global.glitch_horizontal;
                                                    let _e789 = random;
                                                    let _e791 = global.glitch_vertical;
                                                    if ((_e785.x < _e787) && (_e789.y < _e791)) {
                                                        {
                                                            let _e795 = tex;
                                                            let _e797 = random;
                                                            let _e799 = global.inputDelay;
                                                            let _e802 = (_e795.x + (_e797.x * _e799.z));
                                                            tex.x = (_e802 - (floor((_e802 / 1f)) * 1f));
                                                            let _e809 = tex;
                                                            let _e811 = random;
                                                            let _e813 = global.inputDelay;
                                                            let _e816 = (_e809.y + (_e811.y * _e813.z));
                                                            tex.y = (_e816 - (floor((_e816 / 1f)) * 1f));
                                                        }
                                                    } else {
                                                        let _e822 = random;
                                                        let _e824 = global.glitch_horizontal;
                                                        if (_e822.x < _e824) {
                                                            {
                                                                let _e827 = tex;
                                                                let _e829 = random;
                                                                let _e831 = global.inputDelay;
                                                                let _e834 = (_e827.y + (_e829.x * _e831.z));
                                                                tex.y = (_e834 - (floor((_e834 / 1f)) * 1f));
                                                            }
                                                        } else {
                                                            let _e840 = random;
                                                            let _e842 = global.glitch_vertical;
                                                            if (_e840.y < _e842) {
                                                                {
                                                                    let _e845 = tex;
                                                                    let _e847 = random;
                                                                    let _e849 = global.inputDelay;
                                                                    let _e852 = (_e845.x + (_e847.y * _e849.z));
                                                                    tex.x = (_e852 - (floor((_e852 / 1f)) * 1f));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            let _e858 = pixelBuffer;
                                            if (_e858.z < 1f) {
                                                {
                                                    let _e863 = tex;
                                                    let _e865 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e863, 0f);
                                                    color.z = _e865.z;
                                                }
                                            } else {
                                                let _e867 = pixelBuffer;
                                                if (_e867.z < 2f) {
                                                    {
                                                        let _e872 = tex;
                                                        let _e874 = textureSampleLevel(isf_tex_buffer1_, isf_smp_buffer1_, _e872, 0f);
                                                        color.z = _e874.z;
                                                    }
                                                } else {
                                                    let _e876 = pixelBuffer;
                                                    if (_e876.z < 3f) {
                                                        {
                                                            let _e881 = tex;
                                                            let _e883 = textureSampleLevel(isf_tex_buffer2_, isf_smp_buffer2_, _e881, 0f);
                                                            color.z = _e883.z;
                                                        }
                                                    } else {
                                                        let _e885 = pixelBuffer;
                                                        if (_e885.z < 4f) {
                                                            {
                                                                let _e890 = tex;
                                                                let _e892 = textureSampleLevel(isf_tex_buffer3_, isf_smp_buffer3_, _e890, 0f);
                                                                color.z = _e892.z;
                                                            }
                                                        } else {
                                                            let _e894 = pixelBuffer;
                                                            if (_e894.z < 5f) {
                                                                {
                                                                    let _e899 = tex;
                                                                    let _e901 = textureSampleLevel(isf_tex_buffer4_, isf_smp_buffer4_, _e899, 0f);
                                                                    color.z = _e901.z;
                                                                }
                                                            } else {
                                                                let _e903 = pixelBuffer;
                                                                if (_e903.z < 6f) {
                                                                    {
                                                                        let _e908 = tex;
                                                                        let _e910 = textureSampleLevel(isf_tex_buffer5_, isf_smp_buffer5_, _e908, 0f);
                                                                        color.z = _e910.z;
                                                                    }
                                                                } else {
                                                                    let _e912 = pixelBuffer;
                                                                    if (_e912.z < 7f) {
                                                                        {
                                                                            let _e917 = tex;
                                                                            let _e919 = textureSampleLevel(isf_tex_buffer6_, isf_smp_buffer6_, _e917, 0f);
                                                                            color.z = _e919.z;
                                                                        }
                                                                    } else {
                                                                        let _e921 = pixelBuffer;
                                                                        if (_e921.z < 8f) {
                                                                            {
                                                                                let _e926 = tex;
                                                                                let _e928 = textureSampleLevel(isf_tex_buffer7_, isf_smp_buffer7_, _e926, 0f);
                                                                                color.z = _e928.z;
                                                                            }
                                                                        } else {
                                                                            {
                                                                                let _e931 = tex;
                                                                                let _e933 = textureSampleLevel(isf_tex_buffer8_, isf_smp_buffer8_, _e931, 0f);
                                                                                color.z = _e933.z;
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            let _e935 = global.randomize_position_on;
                                            if (_e935 != 0i) {
                                                {
                                                    let _e938 = random;
                                                    let _e940 = global.glitch_horizontal;
                                                    let _e942 = random;
                                                    let _e944 = global.glitch_vertical;
                                                    if ((_e938.x < _e940) && (_e942.y < _e944)) {
                                                        {
                                                            let _e948 = tex;
                                                            let _e950 = random;
                                                            let _e952 = global.inputDelay;
                                                            let _e955 = (_e948.x + (_e950.x * _e952.w));
                                                            tex.x = (_e955 - (floor((_e955 / 1f)) * 1f));
                                                            let _e962 = tex;
                                                            let _e964 = random;
                                                            let _e966 = global.inputDelay;
                                                            let _e969 = (_e962.y + (_e964.y * _e966.w));
                                                            tex.y = (_e969 - (floor((_e969 / 1f)) * 1f));
                                                        }
                                                    } else {
                                                        let _e975 = random;
                                                        let _e977 = global.glitch_horizontal;
                                                        if (_e975.x < _e977) {
                                                            {
                                                                let _e980 = tex;
                                                                let _e982 = random;
                                                                let _e984 = global.inputDelay;
                                                                let _e987 = (_e980.y + (_e982.x * _e984.w));
                                                                tex.y = (_e987 - (floor((_e987 / 1f)) * 1f));
                                                            }
                                                        } else {
                                                            let _e993 = random;
                                                            let _e995 = global.glitch_vertical;
                                                            if (_e993.y < _e995) {
                                                                {
                                                                    let _e998 = tex;
                                                                    let _e1000 = random;
                                                                    let _e1002 = global.inputDelay;
                                                                    let _e1005 = (_e998.x + (_e1000.y * _e1002.w));
                                                                    tex.x = (_e1005 - (floor((_e1005 / 1f)) * 1f));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            let _e1011 = pixelBuffer;
                                            if (_e1011.w < 1f) {
                                                {
                                                    let _e1016 = tex;
                                                    let _e1018 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e1016, 0f);
                                                    color.w = _e1018.w;
                                                }
                                            } else {
                                                let _e1020 = pixelBuffer;
                                                if (_e1020.w < 2f) {
                                                    {
                                                        let _e1025 = tex;
                                                        let _e1027 = textureSampleLevel(isf_tex_buffer1_, isf_smp_buffer1_, _e1025, 0f);
                                                        color.w = _e1027.w;
                                                    }
                                                } else {
                                                    let _e1029 = pixelBuffer;
                                                    if (_e1029.w < 3f) {
                                                        {
                                                            let _e1034 = tex;
                                                            let _e1036 = textureSampleLevel(isf_tex_buffer2_, isf_smp_buffer2_, _e1034, 0f);
                                                            color.w = _e1036.w;
                                                        }
                                                    } else {
                                                        let _e1038 = pixelBuffer;
                                                        if (_e1038.w < 4f) {
                                                            {
                                                                let _e1043 = tex;
                                                                let _e1045 = textureSampleLevel(isf_tex_buffer3_, isf_smp_buffer3_, _e1043, 0f);
                                                                color.w = _e1045.w;
                                                            }
                                                        } else {
                                                            let _e1047 = pixelBuffer;
                                                            if (_e1047.w < 5f) {
                                                                {
                                                                    let _e1052 = tex;
                                                                    let _e1054 = textureSampleLevel(isf_tex_buffer4_, isf_smp_buffer4_, _e1052, 0f);
                                                                    color.w = _e1054.w;
                                                                }
                                                            } else {
                                                                let _e1056 = pixelBuffer;
                                                                if (_e1056.w < 6f) {
                                                                    {
                                                                        let _e1061 = tex;
                                                                        let _e1063 = textureSampleLevel(isf_tex_buffer5_, isf_smp_buffer5_, _e1061, 0f);
                                                                        color.w = _e1063.w;
                                                                    }
                                                                } else {
                                                                    let _e1065 = pixelBuffer;
                                                                    if (_e1065.w < 7f) {
                                                                        {
                                                                            let _e1070 = tex;
                                                                            let _e1072 = textureSampleLevel(isf_tex_buffer6_, isf_smp_buffer6_, _e1070, 0f);
                                                                            color.w = _e1072.w;
                                                                        }
                                                                    } else {
                                                                        let _e1074 = pixelBuffer;
                                                                        if (_e1074.w < 8f) {
                                                                            {
                                                                                let _e1079 = tex;
                                                                                let _e1081 = textureSampleLevel(isf_tex_buffer7_, isf_smp_buffer7_, _e1079, 0f);
                                                                                color.w = _e1081.w;
                                                                            }
                                                                        } else {
                                                                            {
                                                                                let _e1084 = tex;
                                                                                let _e1086 = textureSampleLevel(isf_tex_buffer8_, isf_smp_buffer8_, _e1084, 0f);
                                                                                color.w = _e1086.w;
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            let _e1088 = color;
                                            isf_FragColor = _e1088;
                                            return;
                                        }
                                    } else {
                                        return;
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

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e71 = isf_FragColor;
    return FragmentOutput(_e71);
}
