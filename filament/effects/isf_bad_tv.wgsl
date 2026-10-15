// Bad TV: ISF-Files' Bad TV.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by VIDVOX), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@noiseLevel:noiseLevel,0.0,1.0,0.5,0.001
// --track@distortion1:distortion1,0.0,5.0,1.0,0.001
// --track@distortion2:distortion2,0.0,5.0,5.0,0.001
// --track@speed:speed,0.0,1.0,0.3,0.001
// --track@scroll:scroll,0.0,1.0,0.0,0.001
// --track@scanLineThickness:scanLineThickness,1.0,50.0,25.0,0.001
// --track@scanLineIntensity:scanLineIntensity,0.0,1.0,0.5,0.001
// --track@scanLineOffset:scanLineOffset,0.0,1.0,0.0,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    noiseLevel: f32,
    distortion1_: f32,
    distortion2_: f32,
    speed: f32,
    scroll: f32,
    scanLineThickness: f32,
    scanLineIntensity: f32,
    scanLineOffset: f32,
}

struct FragmentOutput {
    @location(0) isf_FragColor: vec4<f32>,
}

const C: vec4<f32> = vec4<f32>(0.21132487f, 0.36602542f, -0.57735026f, 0.024390243f);
const tau: f32 = 6.2831855f;

@group(0) @binding(0) 
var isf_picture: texture_2d<f32>;
@group(0) @binding(1) 
var isf_sampler: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;

fn mod289_(x: vec3<f32>) -> vec3<f32> {
    var x_1: vec3<f32>;

    x_1 = x;
    let _e29 = x_1;
    let _e30 = x_1;
    return (_e29 - (floor((_e30 * 0.0034602077f)) * 289f));
}

fn mod289_1(x_2: vec2<f32>) -> vec2<f32> {
    var x_3: vec2<f32>;

    x_3 = x_2;
    let _e29 = x_3;
    let _e30 = x_3;
    return (_e29 - (floor((_e30 * 0.0034602077f)) * 289f));
}

fn permute(x_4: vec3<f32>) -> vec3<f32> {
    var x_5: vec3<f32>;

    x_5 = x_4;
    let _e29 = x_5;
    let _e35 = x_5;
    let _e37 = mod289_((((_e29 * 34f) + vec3(1f)) * _e35));
    return _e37;
}

fn snoise(v: vec2<f32>) -> f32 {
    var v_1: vec2<f32>;
    var i: vec2<f32>;
    var x0_: vec2<f32>;
    var i1_: vec2<f32>;
    var local: vec2<f32>;
    var x12_: vec4<f32>;
    var p: vec3<f32>;
    var m: vec3<f32>;
    var x_6: vec3<f32>;
    var h: vec3<f32>;
    var ox: vec3<f32>;
    var a0_: vec3<f32>;
    var g: vec3<f32>;

    v_1 = v;
    let _e29 = v_1;
    let _e30 = v_1;
    i = floor((_e29 + vec2(dot(_e30, vec2<f32>(0.36602542f, 0.36602542f)))));
    let _e42 = v_1;
    let _e43 = i;
    let _e45 = i;
    x0_ = ((_e42 - _e43) + vec2(dot(_e45, vec2<f32>(0.21132487f, 0.21132487f))));
    let _e57 = x0_;
    let _e59 = x0_;
    if (_e57.x > _e59.y) {
        local = vec2<f32>(1f, 0f);
    } else {
        local = vec2<f32>(0f, 1f);
    }
    let _e69 = local;
    i1_ = _e69;
    let _e70 = x0_;
    x12_ = (_e70.xyxy + vec4<f32>(0.21132487f, 0.21132487f, -0.57735026f, -0.57735026f));
    let _e80 = x12_;
    let _e82 = x12_;
    let _e84 = i1_;
    let _e85 = (_e82.xy - _e84);
    x12_.x = _e85.x;
    x12_.y = _e85.y;
    let _e90 = i;
    let _e91 = mod289_1(_e90);
    i = _e91;
    let _e92 = i;
    let _e95 = i1_;
    let _e101 = permute((vec3(_e92.y) + vec3<f32>(0f, _e95.y, 1f)));
    let _e102 = i;
    let _e107 = i1_;
    let _e112 = permute(((_e101 + vec3(_e102.x)) + vec3<f32>(0f, _e107.x, 1f)));
    p = _e112;
    let _e115 = x0_;
    let _e116 = x0_;
    let _e118 = x12_;
    let _e120 = x12_;
    let _e123 = x12_;
    let _e125 = x12_;
    m = max((vec3(0.5f) - vec3<f32>(dot(_e115, _e116), dot(_e118.xy, _e120.xy), dot(_e123.zw, _e125.zw))), vec3(0f));
    let _e135 = m;
    let _e136 = m;
    m = (_e135 * _e136);
    let _e138 = m;
    let _e139 = m;
    m = (_e138 * _e139);
    let _e142 = p;
    x_6 = ((2f * fract((_e142 * vec3<f32>(0.024390243f, 0.024390243f, 0.024390243f)))) - vec3(1f));
    let _e156 = x_6;
    h = (abs(_e156) - vec3(0.5f));
    let _e162 = x_6;
    ox = floor((_e162 + vec3(0.5f)));
    let _e168 = x_6;
    let _e169 = ox;
    a0_ = (_e168 - _e169);
    let _e172 = m;
    let _e175 = a0_;
    let _e176 = a0_;
    let _e178 = h;
    let _e179 = h;
    m = (_e172 * (vec3(1.7928429f) - (0.85373473f * ((_e175 * _e176) + (_e178 * _e179)))));
    let _e188 = a0_;
    let _e190 = x0_;
    let _e193 = h;
    let _e195 = x0_;
    g.x = ((_e188.x * _e190.x) + (_e193.x * _e195.y));
    let _e199 = g;
    let _e201 = a0_;
    let _e203 = x12_;
    let _e206 = h;
    let _e208 = x12_;
    let _e211 = ((_e201.yz * _e203.xz) + (_e206.yz * _e208.yw));
    g.y = _e211.x;
    g.z = _e211.y;
    let _e217 = m;
    let _e218 = g;
    return (130f * dot(_e217, _e218));
}

fn pattern(pt: vec2<f32>) -> vec2<f32> {
    var pt_1: vec2<f32>;
    var s: f32 = 0f;
    var c: f32 = 1f;
    var tex: vec2<f32>;
    var point: vec2<f32>;
    var d: f32;

    pt_1 = pt;
    let _e34 = pt_1;
    let _e35 = global.resolution;
    tex = (_e34 * _e35);
    let _e38 = c;
    let _e39 = tex;
    let _e42 = s;
    let _e43 = tex;
    let _e47 = s;
    let _e48 = tex;
    let _e51 = c;
    let _e52 = tex;
    let _e58 = global.scanLineThickness;
    point = (vec2<f32>(((_e38 * _e39.x) - (_e42 * _e43.y)), ((_e47 * _e48.x) + (_e51 * _e52.y))) * (1f / _e58));
    let _e62 = point;
    d = _e62.y;
    let _e65 = d;
    let _e66 = global.scanLineOffset;
    let _e69 = pt_1;
    let _e75 = d;
    let _e76 = global.scanLineOffset;
    let _e79 = pt_1;
    return vec2<f32>(sin(((_e65 + (_e66 * tau)) + cos((_e69.x * tau)))), cos(((_e75 + (_e76 * tau)) + sin((_e79.y * tau)))));
}

fn rand(co: vec2<f32>) -> f32 {
    var co_1: vec2<f32>;

    co_1 = co;
    let _e30 = co_1;
    return fract((sin(dot(_e30.xy, vec2<f32>(12.9898f, 78.233f))) * 43758.547f));
}

fn main_1() {
    var p_1: vec2<f32>;
    var ty: f32;
    var yt: f32;
    var offset: f32;
    var adjusted: vec2<f32>;
    var result: vec4<f32>;
    var pat: vec2<f32>;
    var shift: vec3<f32>;

    let _e28 = isf_FragNormCoord_1;
    p_1 = _e28;
    let _e30 = global.time;
    let _e31 = global.speed;
    ty = (_e30 * _e31);
    let _e34 = p_1;
    let _e36 = ty;
    yt = (_e34.y - _e36);
    let _e39 = yt;
    let _e44 = snoise(vec2<f32>((_e39 * 3f), 0f));
    offset = (_e44 * 0.2f);
    let _e48 = offset;
    let _e49 = global.distortion1_;
    let _e53 = global.distortion1_;
    offset = (pow((_e48 * _e49), 3f) / max(_e53, 0.001f));
    let _e57 = offset;
    let _e58 = yt;
    let _e63 = snoise(vec2<f32>((_e58 * 50f), 0f));
    let _e64 = global.distortion2_;
    offset = (_e57 + ((_e63 * _e64) * 0.001f));
    let _e69 = p_1;
    let _e71 = offset;
    let _e74 = p_1;
    let _e76 = global.scroll;
    adjusted = vec2<f32>(fract((_e69.x + _e71)), fract((_e74.y - _e76)));
    let _e81 = adjusted;
    let _e83 = textureSampleLevel(isf_picture, isf_sampler, _e81, 0f);
    result = _e83;
    let _e85 = adjusted;
    let _e86 = pattern(_e85);
    pat = _e86;
    let _e88 = global.scanLineIntensity;
    let _e90 = pat;
    let _e94 = pat;
    shift = ((_e88 * vec3<f32>((0.3f * _e90.x), (0.59f * _e94.y), 0.11f)) / vec3(2f));
    let _e104 = result;
    let _e107 = global.scanLineIntensity;
    let _e111 = result;
    let _e114 = shift;
    let _e116 = adjusted;
    let _e117 = global.time;
    let _e119 = rand((_e116 * _e117));
    let _e122 = global.noiseLevel;
    let _e125 = ((((1f + (_e107 / 2f)) * _e111.xyz) + _e114) + vec3(((_e119 - 0.5f) * _e122)));
    result.x = _e125.x;
    result.y = _e125.y;
    result.z = _e125.z;
    let _e132 = result;
    isf_FragColor = _e132;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e37 = isf_FragColor;
    return FragmentOutput(_e37);
}
