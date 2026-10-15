// Boxinator: ISF-Files' Boxinator.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by mojovideotech), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --track@rate:rate,0.0,10.0,2.5,0.001
// --track@edge:edge,0.0,0.01,0.001,0.001
// --track@blend:blend,-1.0,1.0,0.95,0.001
// --track@randomize:randomize,0.0,1.0,0.5,0.001
// --track@gamma:gamma,-0.5,0.2,-0.3,0.001
// --track@grid_x:grid x,1.5,900.0,64.0,0.001
// --track@grid_y:grid y,1.5,600.0,36.0,0.001

struct Knobs {
    time: f32,
    resolution: vec2<f32>,
    frame: f32,
    rate: f32,
    edge: f32,
    blend: f32,
    randomize: f32,
    gamma: f32,
    grid: vec2<f32>,
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
var<private> gl_FragCoord_1: vec4<f32>;

fn permute(x: vec4<f32>) -> vec4<f32> {
    var x_1: vec4<f32>;

    x_1 = x;
    let _e24 = x_1;
    let _e30 = x_1;
    let _e31 = (((_e24 * 34f) + vec4(1f)) * _e30);
    let _e33 = vec4(289f);
    return (_e31 - (floor((_e31 / _e33)) * _e33));
}

fn taylorInvSqrt(r: vec4<f32>) -> vec4<f32> {
    var r_1: vec4<f32>;

    r_1 = r;
    let _e26 = r_1;
    return (vec4(1.7928429f) - (0.85373473f * _e26));
}

fn snoise(v: vec3<f32>) -> f32 {
    var v_1: vec3<f32>;
    var C: vec2<f32> = vec2<f32>(0.16666667f, 0.33333334f);
    var D: vec4<f32> = vec4<f32>(0f, 0.5f, 1f, 2f);
    var i: vec3<f32>;
    var x0_: vec3<f32>;
    var g: vec3<f32>;
    var l: vec3<f32>;
    var i1_: vec3<f32>;
    var i2_: vec3<f32>;
    var x1_: vec3<f32>;
    var x2_: vec3<f32>;
    var x3_: vec3<f32>;
    var p: vec4<f32>;
    var n: f32 = 0.14285715f;
    var ns: vec3<f32>;
    var j: vec4<f32>;
    var x_2: vec4<f32>;
    var y: vec4<f32>;
    var x_3: vec4<f32>;
    var y_1: vec4<f32>;
    var h: vec4<f32>;
    var b0_: vec4<f32>;
    var b1_: vec4<f32>;
    var s0_: vec4<f32>;
    var s1_: vec4<f32>;
    var sh: vec4<f32>;
    var a0_: vec4<f32>;
    var a1_: vec4<f32>;
    var p0_: vec3<f32>;
    var p1_: vec3<f32>;
    var p2_: vec3<f32>;
    var p3_: vec3<f32>;
    var norm: vec4<f32>;
    var m: vec4<f32>;

    v_1 = v;
    let _e38 = v_1;
    let _e39 = v_1;
    let _e40 = C;
    i = floor((_e38 + vec3(dot(_e39, _e40.yyy))));
    let _e47 = v_1;
    let _e48 = i;
    let _e50 = i;
    let _e51 = C;
    x0_ = ((_e47 - _e48) + vec3(dot(_e50, _e51.xxx)));
    let _e57 = x0_;
    let _e59 = x0_;
    g = step(_e57.yzx, _e59.xyz);
    let _e64 = g;
    l = (vec3(1f) - _e64);
    let _e68 = g;
    let _e70 = l;
    i1_ = min(_e68.xyz, _e70.zxy);
    let _e74 = g;
    let _e76 = l;
    i2_ = max(_e74.xyz, _e76.zxy);
    let _e80 = x0_;
    let _e81 = i1_;
    let _e84 = C;
    x1_ = ((_e80 - _e81) + (1f * _e84.xxx));
    let _e89 = x0_;
    let _e90 = i2_;
    let _e93 = C;
    x2_ = ((_e89 - _e90) + (2f * _e93.xxx));
    let _e98 = x0_;
    let _e103 = C;
    x3_ = ((_e98 - vec3(1f)) + (3f * _e103.xxx));
    let _e108 = i;
    let _e110 = vec3(289f);
    i = (_e108 - (floor((_e108 / _e110)) * _e110));
    let _e115 = i;
    let _e118 = i1_;
    let _e120 = i2_;
    let _e126 = permute((vec4(_e115.z) + vec4<f32>(0f, _e118.z, _e120.z, 1f)));
    let _e127 = i;
    let _e132 = i1_;
    let _e134 = i2_;
    let _e139 = permute(((_e126 + vec4(_e127.y)) + vec4<f32>(0f, _e132.y, _e134.y, 1f)));
    let _e140 = i;
    let _e145 = i1_;
    let _e147 = i2_;
    let _e152 = permute(((_e139 + vec4(_e140.x)) + vec4<f32>(0f, _e145.x, _e147.x, 1f)));
    p = _e152;
    let _e158 = n;
    let _e159 = D;
    let _e162 = D;
    ns = ((_e158 * _e159.wyz) - _e162.xzx);
    let _e166 = p;
    let _e168 = p;
    let _e169 = ns;
    let _e172 = ns;
    j = (_e166 - (49f * floor(((_e168 * _e169.z) * _e172.z))));
    let _e179 = j;
    let _e180 = ns;
    x_2 = floor((_e179 * _e180.z));
    let _e185 = j;
    let _e187 = x_2;
    y = floor((_e185 - (7f * _e187)));
    let _e192 = x_2;
    let _e193 = ns;
    let _e196 = ns;
    x_3 = ((_e192 * _e193.x) + _e196.yyyy);
    let _e200 = y;
    let _e201 = ns;
    let _e204 = ns;
    y_1 = ((_e200 * _e201.x) + _e204.yyyy);
    let _e209 = x_3;
    let _e213 = y_1;
    h = ((vec4(1f) - abs(_e209)) - abs(_e213));
    let _e217 = x_3;
    let _e218 = _e217.xy;
    let _e219 = y_1;
    let _e220 = _e219.xy;
    b0_ = vec4<f32>(_e218.x, _e218.y, _e220.x, _e220.y);
    let _e227 = x_3;
    let _e228 = _e227.zw;
    let _e229 = y_1;
    let _e230 = _e229.zw;
    b1_ = vec4<f32>(_e228.x, _e228.y, _e230.x, _e230.y);
    let _e237 = b0_;
    s0_ = ((floor(_e237) * 2f) + vec4(1f));
    let _e245 = b1_;
    s1_ = ((floor(_e245) * 2f) + vec4(1f));
    let _e253 = h;
    sh = -(step(_e253, vec4(0f)));
    let _e259 = b0_;
    let _e261 = s0_;
    let _e263 = sh;
    a0_ = (_e259.xzyw + (_e261.xzyw * _e263.xxyy));
    let _e268 = b1_;
    let _e270 = s1_;
    let _e272 = sh;
    a1_ = (_e268.xzyw + (_e270.xzyw * _e272.zzww));
    let _e277 = a0_;
    let _e278 = _e277.xy;
    let _e279 = h;
    p0_ = vec3<f32>(_e278.x, _e278.y, _e279.x);
    let _e285 = a0_;
    let _e286 = _e285.zw;
    let _e287 = h;
    p1_ = vec3<f32>(_e286.x, _e286.y, _e287.y);
    let _e293 = a1_;
    let _e294 = _e293.xy;
    let _e295 = h;
    p2_ = vec3<f32>(_e294.x, _e294.y, _e295.z);
    let _e301 = a1_;
    let _e302 = _e301.zw;
    let _e303 = h;
    p3_ = vec3<f32>(_e302.x, _e302.y, _e303.w);
    let _e309 = p0_;
    let _e310 = p0_;
    let _e312 = p1_;
    let _e313 = p1_;
    let _e315 = p2_;
    let _e316 = p2_;
    let _e318 = p3_;
    let _e319 = p3_;
    let _e322 = taylorInvSqrt(vec4<f32>(dot(_e309, _e310), dot(_e312, _e313), dot(_e315, _e316), dot(_e318, _e319)));
    norm = _e322;
    let _e324 = p0_;
    let _e325 = norm;
    p0_ = (_e324 * _e325.x);
    let _e328 = p1_;
    let _e329 = norm;
    p1_ = (_e328 * _e329.y);
    let _e332 = p2_;
    let _e333 = norm;
    p2_ = (_e332 * _e333.z);
    let _e336 = p3_;
    let _e337 = norm;
    p3_ = (_e336 * _e337.w);
    let _e341 = x0_;
    let _e342 = x0_;
    let _e344 = x1_;
    let _e345 = x1_;
    let _e347 = x2_;
    let _e348 = x2_;
    let _e350 = x3_;
    let _e351 = x3_;
    m = max((vec4(0.6f) - vec4<f32>(dot(_e341, _e342), dot(_e344, _e345), dot(_e347, _e348), dot(_e350, _e351))), vec4(0f));
    let _e360 = m;
    let _e361 = m;
    m = (_e360 * _e361);
    let _e364 = m;
    let _e365 = m;
    let _e367 = p0_;
    let _e368 = x0_;
    let _e370 = p1_;
    let _e371 = x1_;
    let _e373 = p2_;
    let _e374 = x2_;
    let _e376 = p3_;
    let _e377 = x3_;
    return (42f * dot((_e364 * _e365), vec4<f32>(dot(_e367, _e368), dot(_e370, _e371), dot(_e373, _e374), dot(_e376, _e377))));
}

fn hash(h_1: f32) -> f32 {
    var h_2: f32;

    h_2 = h_1;
    let _e24 = h_2;
    return fract((sin(_e24) * 43758.547f));
}

fn tile(cell: vec2<f32>, size: vec2<f32>) -> vec2<f32> {
    var cell_1: vec2<f32>;
    var size_1: vec2<f32>;

    cell_1 = cell;
    size_1 = size;
    let _e26 = cell_1;
    let _e27 = size_1;
    return fract((_e26 * _e27));
}

fn box(a: vec2<f32>, b: vec2<f32>) -> f32 {
    var a_1: vec2<f32>;
    var b_1: vec2<f32>;
    var o: vec2<f32>;

    a_1 = a;
    b_1 = b;
    let _e26 = b_1;
    let _e27 = a_1;
    o = step(_e26, _e27);
    let _e30 = o;
    let _e32 = o;
    return (_e30.x * _e32.y);
}

fn main_1() {
    var T: f32;
    var uv: vec2<f32>;
    var g_1: vec2<f32>;
    var C_1: f32;
    var I: f32;
    var st: vec2<f32>;
    var S: f32;
    var color: vec3<f32>;
    var n_1: f32;

    let _e22 = global.time;
    let _e23 = global.rate;
    T = (_e22 * _e23);
    let _e27 = gl_FragCoord_1;
    let _e29 = global.resolution;
    uv = (_e27.xy / _e29.xy);
    let _e33 = global.grid;
    g_1 = floor(_e33.xy);
    let _e37 = g_1;
    let _e39 = g_1;
    C_1 = (_e37.x * _e39.y);
    let _e44 = uv;
    let _e46 = g_1;
    let _e51 = g_1;
    let _e53 = uv;
    let _e55 = g_1;
    let _e61 = g_1;
    I = (((1f + floor((_e44.x * _e46.x))) + (_e51.y * floor((_e53.y * _e55.y)))) + _e61.x);
    let _e65 = uv;
    let _e66 = g_1;
    let _e67 = tile(_e65, _e66);
    st = _e67;
    let _e69 = I;
    let _e70 = C_1;
    let _e72 = st;
    let _e73 = global.edge;
    let _e74 = g_1;
    let _e78 = box(_e72, vec2<f32>((_e73 * _e74.xy)));
    S = ((_e69 / _e70) * _e78);
    let _e81 = S;
    let _e82 = S;
    let _e83 = hash(_e82);
    let _e84 = global.randomize;
    S = mix(_e81, _e83, _e84);
    let _e86 = S;
    let _e87 = T;
    color = vec3((_e86 * _e87));
    let _e91 = color;
    let _e92 = uv;
    let _e95 = textureSampleLevel(isf_picture, isf_sampler, _e92.xy, 0f);
    let _e97 = global.blend;
    let _e100 = snoise((_e91 + (_e95.xyz * _e97)));
    n_1 = _e100;
    let _e102 = n_1;
    let _e103 = n_1;
    let _e104 = n_1;
    let _e105 = vec3<f32>(_e102, _e103, _e104);
    let _e111 = uv;
    let _e114 = textureSampleLevel(isf_picture, isf_sampler, _e111.xy, 0f);
    let _e119 = global.gamma;
    isf_FragColor = sqrt((max((vec4<f32>(_e105.x, _e105.y, _e105.z, 1f) + _e114), vec4(0f)) + vec4(_e119)));
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e31 = isf_FragColor;
    return FragmentOutput(_e31);
}
