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
    passindex: f32,
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
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;

fn mod289_(x: vec3<f32>) -> vec3<f32> {
    var x_1: vec3<f32>;

    x_1 = x;
    let _e31 = x_1;
    let _e32 = x_1;
    return (_e31 - (floor((_e32 * 0.0034602077f)) * 289f));
}

fn mod289_1(x_2: vec2<f32>) -> vec2<f32> {
    var x_3: vec2<f32>;

    x_3 = x_2;
    let _e31 = x_3;
    let _e32 = x_3;
    return (_e31 - (floor((_e32 * 0.0034602077f)) * 289f));
}

fn permute(x_4: vec3<f32>) -> vec3<f32> {
    var x_5: vec3<f32>;

    x_5 = x_4;
    let _e31 = x_5;
    let _e37 = x_5;
    let _e39 = mod289_((((_e31 * 34f) + vec3(1f)) * _e37));
    return _e39;
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
    let _e31 = v_1;
    let _e32 = v_1;
    i = floor((_e31 + vec2(dot(_e32, vec2<f32>(0.36602542f, 0.36602542f)))));
    let _e44 = v_1;
    let _e45 = i;
    let _e47 = i;
    x0_ = ((_e44 - _e45) + vec2(dot(_e47, vec2<f32>(0.21132487f, 0.21132487f))));
    let _e59 = x0_;
    let _e61 = x0_;
    if (_e59.x > _e61.y) {
        local = vec2<f32>(1f, 0f);
    } else {
        local = vec2<f32>(0f, 1f);
    }
    let _e71 = local;
    i1_ = _e71;
    let _e72 = x0_;
    x12_ = (_e72.xyxy + vec4<f32>(0.21132487f, 0.21132487f, -0.57735026f, -0.57735026f));
    let _e82 = x12_;
    let _e84 = x12_;
    let _e86 = i1_;
    let _e87 = (_e84.xy - _e86);
    x12_.x = _e87.x;
    x12_.y = _e87.y;
    let _e92 = i;
    let _e93 = mod289_1(_e92);
    i = _e93;
    let _e94 = i;
    let _e97 = i1_;
    let _e103 = permute((vec3(_e94.y) + vec3<f32>(0f, _e97.y, 1f)));
    let _e104 = i;
    let _e109 = i1_;
    let _e114 = permute(((_e103 + vec3(_e104.x)) + vec3<f32>(0f, _e109.x, 1f)));
    p = _e114;
    let _e117 = x0_;
    let _e118 = x0_;
    let _e120 = x12_;
    let _e122 = x12_;
    let _e125 = x12_;
    let _e127 = x12_;
    m = max((vec3(0.5f) - vec3<f32>(dot(_e117, _e118), dot(_e120.xy, _e122.xy), dot(_e125.zw, _e127.zw))), vec3(0f));
    let _e137 = m;
    let _e138 = m;
    m = (_e137 * _e138);
    let _e140 = m;
    let _e141 = m;
    m = (_e140 * _e141);
    let _e144 = p;
    x_6 = ((2f * fract((_e144 * vec3<f32>(0.024390243f, 0.024390243f, 0.024390243f)))) - vec3(1f));
    let _e158 = x_6;
    h = (abs(_e158) - vec3(0.5f));
    let _e164 = x_6;
    ox = floor((_e164 + vec3(0.5f)));
    let _e170 = x_6;
    let _e171 = ox;
    a0_ = (_e170 - _e171);
    let _e174 = m;
    let _e177 = a0_;
    let _e178 = a0_;
    let _e180 = h;
    let _e181 = h;
    m = (_e174 * (vec3(1.7928429f) - (0.85373473f * ((_e177 * _e178) + (_e180 * _e181)))));
    let _e190 = a0_;
    let _e192 = x0_;
    let _e195 = h;
    let _e197 = x0_;
    g.x = ((_e190.x * _e192.x) + (_e195.x * _e197.y));
    let _e201 = g;
    let _e203 = a0_;
    let _e205 = x12_;
    let _e208 = h;
    let _e210 = x12_;
    let _e213 = ((_e203.yz * _e205.xz) + (_e208.yz * _e210.yw));
    g.y = _e213.x;
    g.z = _e213.y;
    let _e219 = m;
    let _e220 = g;
    return (130f * dot(_e219, _e220));
}

fn pattern(pt: vec2<f32>) -> vec2<f32> {
    var pt_1: vec2<f32>;
    var s: f32 = 0f;
    var c: f32 = 1f;
    var tex: vec2<f32>;
    var point: vec2<f32>;
    var d: f32;

    pt_1 = pt;
    let _e36 = pt_1;
    let _e37 = global.resolution;
    tex = (_e36 * _e37);
    let _e40 = c;
    let _e41 = tex;
    let _e44 = s;
    let _e45 = tex;
    let _e49 = s;
    let _e50 = tex;
    let _e53 = c;
    let _e54 = tex;
    let _e60 = global.scanLineThickness;
    point = (vec2<f32>(((_e40 * _e41.x) - (_e44 * _e45.y)), ((_e49 * _e50.x) + (_e53 * _e54.y))) * (1f / _e60));
    let _e64 = point;
    d = _e64.y;
    let _e67 = d;
    let _e68 = global.scanLineOffset;
    let _e71 = pt_1;
    let _e77 = d;
    let _e78 = global.scanLineOffset;
    let _e81 = pt_1;
    return vec2<f32>(sin(((_e67 + (_e68 * tau)) + cos((_e71.x * tau)))), cos(((_e77 + (_e78 * tau)) + sin((_e81.y * tau)))));
}

fn rand(co: vec2<f32>) -> f32 {
    var co_1: vec2<f32>;

    co_1 = co;
    let _e32 = co_1;
    return fract((sin(dot(_e32.xy, vec2<f32>(12.9898f, 78.233f))) * 43758.547f));
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

    let _e30 = isf_FragNormCoord_1;
    p_1 = _e30;
    let _e32 = global.time;
    let _e33 = global.speed;
    ty = (_e32 * _e33);
    let _e36 = p_1;
    let _e38 = ty;
    yt = (_e36.y - _e38);
    let _e41 = yt;
    let _e46 = snoise(vec2<f32>((_e41 * 3f), 0f));
    offset = (_e46 * 0.2f);
    let _e50 = offset;
    let _e51 = global.distortion1_;
    let _e55 = global.distortion1_;
    offset = (pow((_e50 * _e51), 3f) / max(_e55, 0.001f));
    let _e59 = offset;
    let _e60 = yt;
    let _e65 = snoise(vec2<f32>((_e60 * 50f), 0f));
    let _e66 = global.distortion2_;
    offset = (_e59 + ((_e65 * _e66) * 0.001f));
    let _e71 = p_1;
    let _e73 = offset;
    let _e76 = p_1;
    let _e78 = global.scroll;
    adjusted = vec2<f32>(fract((_e71.x + _e73)), fract((_e76.y - _e78)));
    let _e83 = adjusted;
    let _e85 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e83, 0f);
    result = _e85;
    let _e87 = adjusted;
    let _e88 = pattern(_e87);
    pat = _e88;
    let _e90 = global.scanLineIntensity;
    let _e92 = pat;
    let _e96 = pat;
    shift = ((_e90 * vec3<f32>((0.3f * _e92.x), (0.59f * _e96.y), 0.11f)) / vec3(2f));
    let _e106 = result;
    let _e109 = global.scanLineIntensity;
    let _e113 = result;
    let _e116 = shift;
    let _e118 = adjusted;
    let _e119 = global.time;
    let _e121 = rand((_e118 * _e119));
    let _e124 = global.noiseLevel;
    let _e127 = ((((1f + (_e109 / 2f)) * _e113.xyz) + _e116) + vec3(((_e121 - 0.5f) * _e124)));
    result.x = _e127.x;
    result.y = _e127.y;
    result.z = _e127.z;
    let _e134 = result;
    isf_FragColor = _e134;
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    main_1();
    let _e39 = isf_FragColor;
    return FragmentOutput(_e39);
}
