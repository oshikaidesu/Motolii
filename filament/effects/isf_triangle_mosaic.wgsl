// Triangle Mosaic: ISF-Files' Triangle Mosaic.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@cellSize:Cell Size,0.005,0.25,0.025,0.001
// --track@jitter:Jitter,0.0,1.0,0.5,0.001
// --track@seed:Seed,0.0,100.0,0.0,0.001
// --track@outlineWidth:Outline Width,0.0,5.0,1.0,0.001
// --track@outlineColor_x:Outline Color r,0,1,0.0,0.001
// --track@outlineColor_y:Outline Color g,0,1,0.0,0.001
// --track@outlineColor_z:Outline Color b,0,1,0.0,0.001
// --track@outlineColor_w:Outline Color a,0,1,1.0,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    cellSize: f32,
    jitter: f32,
    seed: f32,
    outlineWidth: f32,
    outlineColor: vec4<f32>,
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

fn hash2_(p: vec2<f32>) -> vec2<f32> {
    var p_1: vec2<f32>;

    p_1 = p;
    let _e22 = p_1;
    let _e27 = p_1;
    p_1 = vec2<f32>(dot(_e22, vec2<f32>(127.1f, 311.7f)), dot(_e27, vec2<f32>(269.5f, 183.3f)));
    let _e33 = p_1;
    return fract((sin(_e33) * 43758.547f));
}

fn segDist(p_2: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    var p_3: vec2<f32>;
    var a_1: vec2<f32>;
    var b_1: vec2<f32>;
    var ab: vec2<f32>;
    var ap: vec2<f32>;
    var t: f32;

    p_3 = p_2;
    a_1 = a;
    b_1 = b;
    let _e26 = b_1;
    let _e27 = a_1;
    ab = (_e26 - _e27);
    let _e30 = p_3;
    let _e31 = a_1;
    ap = (_e30 - _e31);
    let _e34 = ap;
    let _e35 = ab;
    let _e37 = ab;
    let _e38 = ab;
    t = clamp((dot(_e34, _e35) / max(dot(_e37, _e38), 0.000001f)), 0f, 1f);
    let _e47 = ap;
    let _e48 = ab;
    let _e49 = t;
    return length((_e47 - (_e48 * _e49)));
}

fn main_1() {
    var uv: vec2<f32>;
    var aspect: f32;
    var cellUV: vec2<f32>;
    var grid: vec2<f32>;
    var cell: vec2<f32>;
    var local: vec2<f32>;
    var j00_: vec2<f32>;
    var j10_: vec2<f32>;
    var j01_: vec2<f32>;
    var j11_: vec2<f32>;
    var p00_: vec2<f32>;
    var p10_: vec2<f32>;
    var p01_: vec2<f32>;
    var p11_: vec2<f32>;
    var edge: vec2<f32>;
    var toPt: vec2<f32>;
    var side: f32;
    var va: vec2<f32>;
    var vb: vec2<f32>;
    var vc: vec2<f32>;
    var triCenter: vec2<f32>;
    var sampleUV: vec2<f32>;
    var fillColor: vec4<f32>;
    var finalColor: vec3<f32>;
    var d: f32;
    var dPixels: f32;
    var edgeAA: f32;

    let _e20 = isf_FragNormCoord_1;
    uv = _e20;
    let _e22 = global.resolution;
    let _e24 = global.resolution;
    aspect = (_e22.x / _e24.y);
    let _e28 = global.cellSize;
    let _e29 = global.cellSize;
    let _e30 = aspect;
    cellUV = vec2<f32>(_e28, (_e29 * _e30));
    let _e34 = uv;
    let _e35 = cellUV;
    grid = (_e34 / _e35);
    let _e38 = grid;
    cell = floor(_e38);
    let _e41 = grid;
    local = fract(_e41);
    let _e44 = cell;
    let _e49 = global.seed;
    let _e52 = hash2_(((_e44 + vec2<f32>(0f, 0f)) + vec2(_e49)));
    let _e56 = global.jitter;
    j00_ = ((_e52 - vec2(0.5f)) * _e56);
    let _e59 = cell;
    let _e64 = global.seed;
    let _e67 = hash2_(((_e59 + vec2<f32>(1f, 0f)) + vec2(_e64)));
    let _e71 = global.jitter;
    j10_ = ((_e67 - vec2(0.5f)) * _e71);
    let _e74 = cell;
    let _e79 = global.seed;
    let _e82 = hash2_(((_e74 + vec2<f32>(0f, 1f)) + vec2(_e79)));
    let _e86 = global.jitter;
    j01_ = ((_e82 - vec2(0.5f)) * _e86);
    let _e89 = cell;
    let _e94 = global.seed;
    let _e97 = hash2_(((_e89 + vec2<f32>(1f, 1f)) + vec2(_e94)));
    let _e101 = global.jitter;
    j11_ = ((_e97 - vec2(0.5f)) * _e101);
    let _e107 = j00_;
    p00_ = (vec2<f32>(0f, 0f) + _e107);
    let _e113 = j10_;
    p10_ = (vec2<f32>(1f, 0f) + _e113);
    let _e119 = j01_;
    p01_ = (vec2<f32>(0f, 1f) + _e119);
    let _e125 = j11_;
    p11_ = (vec2<f32>(1f, 1f) + _e125);
    let _e128 = p01_;
    let _e129 = p10_;
    edge = (_e128 - _e129);
    let _e132 = local;
    let _e133 = p10_;
    toPt = (_e132 - _e133);
    let _e136 = edge;
    let _e138 = toPt;
    let _e141 = edge;
    let _e143 = toPt;
    side = ((_e136.x * _e138.y) - (_e141.y * _e143.x));
    let _e151 = side;
    if (_e151 > 0f) {
        {
            let _e154 = p00_;
            va = _e154;
            let _e155 = p10_;
            vb = _e155;
            let _e156 = p01_;
            vc = _e156;
        }
    } else {
        {
            let _e157 = p10_;
            va = _e157;
            let _e158 = p11_;
            vb = _e158;
            let _e159 = p01_;
            vc = _e159;
        }
    }
    let _e160 = va;
    let _e161 = vb;
    let _e163 = vc;
    triCenter = (((_e160 + _e161) + _e163) / vec2(3f));
    let _e169 = cell;
    let _e170 = triCenter;
    let _e172 = cellUV;
    sampleUV = ((_e169 + _e170) * _e172);
    let _e175 = sampleUV;
    sampleUV = clamp(_e175, vec2(0f), vec2(1f));
    let _e181 = sampleUV;
    let _e183 = textureSampleLevel(isf_picture, isf_sampler, _e181, 0f);
    fillColor = _e183;
    let _e185 = fillColor;
    finalColor = _e185.xyz;
    let _e188 = global.outlineWidth;
    if (_e188 > 0f) {
        {
            let _e191 = local;
            let _e192 = va;
            let _e193 = vb;
            let _e194 = segDist(_e191, _e192, _e193);
            let _e195 = local;
            let _e196 = vb;
            let _e197 = vc;
            let _e198 = segDist(_e195, _e196, _e197);
            let _e200 = local;
            let _e201 = vc;
            let _e202 = va;
            let _e203 = segDist(_e200, _e201, _e202);
            d = min(min(_e194, _e198), _e203);
            let _e206 = d;
            let _e207 = cellUV;
            let _e210 = global.resolution;
            dPixels = ((_e206 * _e207.x) * _e210.x);
            let _e215 = global.outlineWidth;
            let _e218 = global.outlineWidth;
            let _e223 = dPixels;
            edgeAA = (1f - smoothstep((_e215 * 0.5f), ((_e218 * 0.5f) + 1f), _e223));
            let _e227 = finalColor;
            let _e228 = global.outlineColor;
            let _e230 = edgeAA;
            let _e231 = global.outlineColor;
            finalColor = mix(_e227, _e228.xyz, vec3((_e230 * _e231.w)));
        }
    }
    let _e236 = finalColor;
    let _e237 = fillColor;
    isf_FragColor = vec4<f32>(_e236.x, _e236.y, _e236.z, _e237.w);
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e27 = isf_FragColor;
    return FragmentOutput(_e27);
}
