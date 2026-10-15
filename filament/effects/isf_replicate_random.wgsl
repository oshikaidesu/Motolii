// Replicate Random: ISF-Files' Replicate Random.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@randomSeed:randomSeed,0,1,0,0.001
// --track@repetitions:repetitions,1,15,5,0.001
// --check@randomizeOpacity_on:randomizeOpacity,0

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    randomSeed: f32,
    repetitions: f32,
    randomizeOpacity_on: i32,
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

fn rand(co: vec2<f32>) -> f32 {
    var co_1: vec2<f32>;

    co_1 = co;
    let _e18 = co_1;
    return fract((sin(dot(_e18.xy, vec2<f32>(12.9898f, 78.233f))) * 43758.547f));
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
    let _e24 = loc_1;
    returnMe = _e24;
    let _e27 = zl_1;
    zoomMult = (1f / _e27);
    let _e32 = p_1;
    let _e35 = c_1;
    let _e37 = global.resolution;
    let _e40 = p_1;
    modifiedCenter = ((((2f * (1f + _e32)) * _e35) / _e37) - vec2((1f + _e40)));
    let _e45 = p_1;
    modifiedPadding = _e45;
    let _e48 = returnMe;
    let _e50 = zoomMult;
    let _e52 = p_1;
    let _e56 = modifiedCenter;
    returnMe.x = (((_e48.x * _e50) + (_e52 / 2f)) - _e56.x);
    let _e60 = returnMe;
    let _e62 = zoomMult;
    let _e64 = p_1;
    let _e68 = modifiedCenter;
    returnMe.y = (((_e60.y * _e62) + (_e64 / 2f)) - _e68.y);
    let _e72 = returnMe;
    let _e75 = modifiedPadding;
    let _e76 = (1f + _e75);
    let _e81 = p_1;
    returnMe.x = ((_e72.x - (floor((_e72.x / _e76)) * _e76)) - (_e81 / 2f));
    let _e86 = returnMe;
    let _e89 = modifiedPadding;
    let _e90 = (1f + _e89);
    let _e95 = p_1;
    returnMe.y = ((_e86.y - (floor((_e86.y / _e90)) * _e90)) - (_e95 / 2f));
    let _e99 = returnMe;
    return _e99;
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
    let _e21 = global.resolution;
    minZoomLevel = (4f / _e21.x);
    let _e25 = seed_1;
    zSeed = ((_e25 * vec2<f32>(0.128f, 9.21f)) + vec2<f32>(1.42f, 2.17f));
    let _e35 = seed_1;
    cSeed1_ = ((_e35 * vec2<f32>(0.436f, 0.931f)) + vec2<f32>(2.76f, 3.779f));
    let _e45 = seed_1;
    cSeed2_ = ((_e45 * vec2<f32>(2.831f, 2.173f)) + vec2<f32>(1.73f, 6.256f));
    let _e56 = zSeed;
    let _e57 = rand(_e56);
    modZoom = (0.9f * _e57);
    let _e60 = global.resolution;
    let _e61 = cSeed1_;
    let _e62 = rand(_e61);
    let _e63 = cSeed2_;
    let _e64 = rand(_e63);
    modCenter = (_e60 * vec2<f32>(_e62, _e64));
    let _e69 = seed_1;
    let _e70 = rand(_e69);
    modPad = (0.5f * _e70);
    let _e73 = modZoom;
    let _e74 = minZoomLevel;
    if (_e73 < _e74) {
        let _e76 = minZoomLevel;
        local = _e76;
    } else {
        let _e77 = modZoom;
        local = _e77;
    }
    let _e79 = local;
    modZoom = _e79;
    let _e80 = loc_3;
    let _e81 = modZoom;
    let _e82 = modCenter;
    let _e83 = modPad;
    let _e84 = paddedZoomedPosition(_e80, _e81, _e82, _e83);
    returnMe_1 = _e84;
    let _e86 = returnMe_1;
    return _e86;
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

    let _e19 = global.repetitions;
    depth = i32(_e19);
    let _e22 = isf_FragNormCoord_1;
    loc_4 = _e22;
    loop {
        let _e26 = i;
        if !((_e26 < 15i)) {
            break;
        }
        {
            let _e33 = i;
            let _e34 = depth;
            if (_e33 >= _e34) {
                break;
            }
            let _e37 = i;
            let _e40 = global.randomSeed;
            let _e45 = i;
            let _e48 = global.randomSeed;
            tmpSeed = vec2<f32>((((1.12f + f32(_e37)) * _e40) + 1.37f), (((1.92f + f32(_e45)) * _e48) + 1.37f));
            let _e54 = global.randomizeOpacity_on;
            if (_e54 != 0i) {
                let _e59 = tmpSeed;
                let _e60 = rand(_e59);
                local_1 = (0.25f + (0.75f * _e60));
            } else {
                local_1 = 1f;
            }
            let _e65 = local_1;
            modOpacity = _e65;
            let _e67 = isf_FragNormCoord_1;
            let _e68 = tmpSeed;
            let _e69 = randomPaddedZoomedPositionWithSeed(_e67, _e68);
            loc_4 = _e69;
            let _e70 = loc_4;
            let _e74 = loc_4;
            let _e79 = loc_4;
            let _e84 = loc_4;
            if ((((_e70.x < 0f) || (_e74.y < 0f)) || (_e79.x > 1f)) || (_e84.y > 1f)) {
                {
                }
            } else {
                {
                    let _e89 = loc_4;
                    let _e91 = textureSampleLevel(isf_picture, isf_sampler, _e89, 0f);
                    tmpColor = _e91;
                    let _e93 = inputPixelColor;
                    let _e95 = inputPixelColor;
                    let _e97 = tmpColor;
                    let _e99 = tmpColor;
                    let _e102 = modOpacity;
                    let _e104 = (_e95.xyz + ((_e97.xyz * _e99.w) * _e102));
                    inputPixelColor.x = _e104.x;
                    inputPixelColor.y = _e104.y;
                    inputPixelColor.z = _e104.z;
                    let _e112 = inputPixelColor;
                    let _e114 = tmpColor;
                    let _e116 = modOpacity;
                    inputPixelColor.w = (_e112.w + (_e114.w * _e116));
                    let _e119 = inputPixelColor;
                    if (_e119.w > 0.99f) {
                        {
                            break;
                        }
                    }
                }
            }
        }
        continuing {
            let _e30 = i;
            i = (_e30 + 1i);
        }
    }
    let _e123 = inputPixelColor;
    isf_FragColor = _e123;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e23 = isf_FragColor;
    return FragmentOutput(_e23);
}
