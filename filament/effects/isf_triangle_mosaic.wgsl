// Triangle Mosaic: ISF-Files' Triangle Mosaic.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Stylize
// --alpha (on a thing: its cells are its shape, each as clear as what it gathered)
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
    passindex: f32,
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
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;

fn hash2_(p: vec2<f32>) -> vec2<f32> {
    var p_1: vec2<f32>;

    p_1 = p;
    let _e24 = p_1;
    let _e29 = p_1;
    p_1 = vec2<f32>(dot(_e24, vec2<f32>(127.1f, 311.7f)), dot(_e29, vec2<f32>(269.5f, 183.3f)));
    let _e35 = p_1;
    return fract((sin(_e35) * 43758.547f));
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
    let _e28 = b_1;
    let _e29 = a_1;
    ab = (_e28 - _e29);
    let _e32 = p_3;
    let _e33 = a_1;
    ap = (_e32 - _e33);
    let _e36 = ap;
    let _e37 = ab;
    let _e39 = ab;
    let _e40 = ab;
    t = clamp((dot(_e36, _e37) / max(dot(_e39, _e40), 0.000001f)), 0f, 1f);
    let _e49 = ap;
    let _e50 = ab;
    let _e51 = t;
    return length((_e49 - (_e50 * _e51)));
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

    let _e22 = isf_FragNormCoord_1;
    uv = _e22;
    let _e24 = global.resolution;
    let _e26 = global.resolution;
    aspect = (_e24.x / _e26.y);
    let _e30 = global.cellSize;
    let _e31 = global.cellSize;
    let _e32 = aspect;
    cellUV = vec2<f32>(_e30, (_e31 * _e32));
    let _e36 = uv;
    let _e37 = cellUV;
    grid = (_e36 / _e37);
    let _e40 = grid;
    cell = floor(_e40);
    let _e43 = grid;
    local = fract(_e43);
    let _e46 = cell;
    let _e51 = global.seed;
    let _e54 = hash2_(((_e46 + vec2<f32>(0f, 0f)) + vec2(_e51)));
    let _e58 = global.jitter;
    j00_ = ((_e54 - vec2(0.5f)) * _e58);
    let _e61 = cell;
    let _e66 = global.seed;
    let _e69 = hash2_(((_e61 + vec2<f32>(1f, 0f)) + vec2(_e66)));
    let _e73 = global.jitter;
    j10_ = ((_e69 - vec2(0.5f)) * _e73);
    let _e76 = cell;
    let _e81 = global.seed;
    let _e84 = hash2_(((_e76 + vec2<f32>(0f, 1f)) + vec2(_e81)));
    let _e88 = global.jitter;
    j01_ = ((_e84 - vec2(0.5f)) * _e88);
    let _e91 = cell;
    let _e96 = global.seed;
    let _e99 = hash2_(((_e91 + vec2<f32>(1f, 1f)) + vec2(_e96)));
    let _e103 = global.jitter;
    j11_ = ((_e99 - vec2(0.5f)) * _e103);
    let _e109 = j00_;
    p00_ = (vec2<f32>(0f, 0f) + _e109);
    let _e115 = j10_;
    p10_ = (vec2<f32>(1f, 0f) + _e115);
    let _e121 = j01_;
    p01_ = (vec2<f32>(0f, 1f) + _e121);
    let _e127 = j11_;
    p11_ = (vec2<f32>(1f, 1f) + _e127);
    let _e130 = p01_;
    let _e131 = p10_;
    edge = (_e130 - _e131);
    let _e134 = local;
    let _e135 = p10_;
    toPt = (_e134 - _e135);
    let _e138 = edge;
    let _e140 = toPt;
    let _e143 = edge;
    let _e145 = toPt;
    side = ((_e138.x * _e140.y) - (_e143.y * _e145.x));
    let _e153 = side;
    if (_e153 > 0f) {
        {
            let _e156 = p00_;
            va = _e156;
            let _e157 = p10_;
            vb = _e157;
            let _e158 = p01_;
            vc = _e158;
        }
    } else {
        {
            let _e159 = p10_;
            va = _e159;
            let _e160 = p11_;
            vb = _e160;
            let _e161 = p01_;
            vc = _e161;
        }
    }
    let _e162 = va;
    let _e163 = vb;
    let _e165 = vc;
    triCenter = (((_e162 + _e163) + _e165) / vec2(3f));
    let _e171 = cell;
    let _e172 = triCenter;
    let _e174 = cellUV;
    sampleUV = ((_e171 + _e172) * _e174);
    let _e177 = sampleUV;
    sampleUV = clamp(_e177, vec2(0f), vec2(1f));
    let _e183 = sampleUV;
    let _e185 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e183, 0f);
    fillColor = _e185;
    let _e187 = fillColor;
    finalColor = _e187.xyz;
    let _e190 = global.outlineWidth;
    if (_e190 > 0f) {
        {
            let _e193 = local;
            let _e194 = va;
            let _e195 = vb;
            let _e196 = segDist(_e193, _e194, _e195);
            let _e197 = local;
            let _e198 = vb;
            let _e199 = vc;
            let _e200 = segDist(_e197, _e198, _e199);
            let _e202 = local;
            let _e203 = vc;
            let _e204 = va;
            let _e205 = segDist(_e202, _e203, _e204);
            d = min(min(_e196, _e200), _e205);
            let _e208 = d;
            let _e209 = cellUV;
            let _e212 = global.resolution;
            dPixels = ((_e208 * _e209.x) * _e212.x);
            let _e217 = global.outlineWidth;
            let _e220 = global.outlineWidth;
            let _e225 = dPixels;
            edgeAA = (1f - smoothstep((_e217 * 0.5f), ((_e220 * 0.5f) + 1f), _e225));
            let _e229 = finalColor;
            let _e230 = global.outlineColor;
            let _e232 = edgeAA;
            let _e233 = global.outlineColor;
            finalColor = mix(_e229, _e230.xyz, vec3((_e232 * _e233.w)));
        }
    }
    let _e238 = finalColor;
    let _e239 = fillColor;
    isf_FragColor = vec4<f32>(_e238.x, _e238.y, _e238.z, _e239.w);
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e29 = isf_FragColor;
    return FragmentOutput(_e29);
}
