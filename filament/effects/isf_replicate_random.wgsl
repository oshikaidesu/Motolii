// Replicate Random: ISF-Files' Replicate Random.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@randomSeed:randomSeed,0,1,0,0.001
// --track@repetitions:repetitions,1,15,5,0.001
// --check@randomizeOpacity_on:randomizeOpacity,0

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    passindex: f32,
    randomSeed: f32,
    repetitions: f32,
    randomizeOpacity_on: i32,
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

fn rand(co: vec2<f32>) -> f32 {
    var co_1: vec2<f32>;

    co_1 = co;
    let _e20 = co_1;
    return fract((sin(dot(_e20.xy, vec2<f32>(12.9898f, 78.233f))) * 43758.547f));
}

fn paddedZoomedPosition(loc: vec2<f32>, zl: f32, c: vec2<f32>, p: f32) -> vec2<f32> {
    var loc_1: vec2<f32>;
    var zl_1: f32;
    var c_1: vec2<f32>;
    var p_1: f32;
    var returnMe: vec2<f32>;
    var zoomMult: f32;
    var modifiedCenter: vec2<f32>;
    var modifiedPadding: f32;

    loc_1 = loc;
    zl_1 = zl;
    c_1 = c;
    p_1 = p;
    let _e26 = loc_1;
    returnMe = _e26;
    let _e29 = zl_1;
    zoomMult = (1f / _e29);
    let _e34 = p_1;
    let _e37 = c_1;
    let _e39 = global.resolution;
    let _e42 = p_1;
    modifiedCenter = ((((2f * (1f + _e34)) * _e37) / _e39) - vec2((1f + _e42)));
    let _e47 = p_1;
    modifiedPadding = _e47;
    let _e50 = returnMe;
    let _e52 = zoomMult;
    let _e54 = p_1;
    let _e58 = modifiedCenter;
    returnMe.x = (((_e50.x * _e52) + (_e54 / 2f)) - _e58.x);
    let _e62 = returnMe;
    let _e64 = zoomMult;
    let _e66 = p_1;
    let _e70 = modifiedCenter;
    returnMe.y = (((_e62.y * _e64) + (_e66 / 2f)) - _e70.y);
    let _e74 = returnMe;
    let _e77 = modifiedPadding;
    let _e78 = (1f + _e77);
    let _e83 = p_1;
    returnMe.x = ((_e74.x - (floor((_e74.x / _e78)) * _e78)) - (_e83 / 2f));
    let _e88 = returnMe;
    let _e91 = modifiedPadding;
    let _e92 = (1f + _e91);
    let _e97 = p_1;
    returnMe.y = ((_e88.y - (floor((_e88.y / _e92)) * _e92)) - (_e97 / 2f));
    let _e101 = returnMe;
    return _e101;
}

fn randomPaddedZoomedPositionWithSeed(loc_2: vec2<f32>, seed: vec2<f32>) -> vec2<f32> {
    var loc_3: vec2<f32>;
    var seed_1: vec2<f32>;
    var minZoomLevel: f32;
    var zSeed: vec2<f32>;
    var cSeed1_: vec2<f32>;
    var cSeed2_: vec2<f32>;
    var modZoom: f32;
    var modCenter: vec2<f32>;
    var modPad: f32;
    var local: f32;
    var returnMe_1: vec2<f32>;

    loc_3 = loc_2;
    seed_1 = seed;
    let _e23 = global.resolution;
    minZoomLevel = (4f / _e23.x);
    let _e27 = seed_1;
    zSeed = ((_e27 * vec2<f32>(0.128f, 9.21f)) + vec2<f32>(1.42f, 2.17f));
    let _e37 = seed_1;
    cSeed1_ = ((_e37 * vec2<f32>(0.436f, 0.931f)) + vec2<f32>(2.76f, 3.779f));
    let _e47 = seed_1;
    cSeed2_ = ((_e47 * vec2<f32>(2.831f, 2.173f)) + vec2<f32>(1.73f, 6.256f));
    let _e58 = zSeed;
    let _e59 = rand(_e58);
    modZoom = (0.9f * _e59);
    let _e62 = global.resolution;
    let _e63 = cSeed1_;
    let _e64 = rand(_e63);
    let _e65 = cSeed2_;
    let _e66 = rand(_e65);
    modCenter = (_e62 * vec2<f32>(_e64, _e66));
    let _e71 = seed_1;
    let _e72 = rand(_e71);
    modPad = (0.5f * _e72);
    let _e75 = modZoom;
    let _e76 = minZoomLevel;
    if (_e75 < _e76) {
        let _e78 = minZoomLevel;
        local = _e78;
    } else {
        let _e79 = modZoom;
        local = _e79;
    }
    let _e81 = local;
    modZoom = _e81;
    let _e82 = loc_3;
    let _e83 = modZoom;
    let _e84 = modCenter;
    let _e85 = modPad;
    let _e86 = paddedZoomedPosition(_e82, _e83, _e84, _e85);
    returnMe_1 = _e86;
    let _e88 = returnMe_1;
    return _e88;
}

fn main_1() {
    var inputPixelColor: vec4<f32> = vec4(0f);
    var depth: i32;
    var loc_4: vec2<f32>;
    var i: i32 = 0i;
    var tmpSeed: vec2<f32>;
    var local_1: f32;
    var modOpacity: f32;
    var tmpColor: vec4<f32>;

    let _e21 = global.repetitions;
    depth = i32(_e21);
    let _e24 = isf_FragNormCoord_1;
    loc_4 = _e24;
    loop {
        let _e28 = i;
        if !((_e28 < 15i)) {
            break;
        }
        {
            let _e35 = i;
            let _e36 = depth;
            if (_e35 >= _e36) {
                break;
            }
            let _e39 = i;
            let _e42 = global.randomSeed;
            let _e47 = i;
            let _e50 = global.randomSeed;
            tmpSeed = vec2<f32>((((1.12f + f32(_e39)) * _e42) + 1.37f), (((1.92f + f32(_e47)) * _e50) + 1.37f));
            let _e56 = global.randomizeOpacity_on;
            if (_e56 != 0i) {
                let _e61 = tmpSeed;
                let _e62 = rand(_e61);
                local_1 = (0.25f + (0.75f * _e62));
            } else {
                local_1 = 1f;
            }
            let _e67 = local_1;
            modOpacity = _e67;
            let _e69 = isf_FragNormCoord_1;
            let _e70 = tmpSeed;
            let _e71 = randomPaddedZoomedPositionWithSeed(_e69, _e70);
            loc_4 = _e71;
            let _e72 = loc_4;
            let _e76 = loc_4;
            let _e81 = loc_4;
            let _e86 = loc_4;
            if ((((_e72.x < 0f) || (_e76.y < 0f)) || (_e81.x > 1f)) || (_e86.y > 1f)) {
                {
                }
            } else {
                {
                    let _e91 = loc_4;
                    let _e93 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e91, 0f);
                    tmpColor = _e93;
                    let _e95 = inputPixelColor;
                    let _e97 = inputPixelColor;
                    let _e99 = tmpColor;
                    let _e101 = tmpColor;
                    let _e104 = modOpacity;
                    let _e106 = (_e97.xyz + ((_e99.xyz * _e101.w) * _e104));
                    inputPixelColor.x = _e106.x;
                    inputPixelColor.y = _e106.y;
                    inputPixelColor.z = _e106.z;
                    let _e114 = inputPixelColor;
                    let _e116 = tmpColor;
                    let _e118 = modOpacity;
                    inputPixelColor.w = (_e114.w + (_e116.w * _e118));
                    let _e121 = inputPixelColor;
                    if (_e121.w > 0.99f) {
                        {
                            break;
                        }
                    }
                }
            }
        }
        continuing {
            let _e32 = i;
            i = (_e32 + 1i);
        }
    }
    let _e125 = inputPixelColor;
    isf_FragColor = _e125;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e25 = isf_FragColor;
    return FragmentOutput(_e25);
}
