// Boxinator: ISF-Files' Boxinator.fs (MIT, Copyright (c) 2018 Vidvox, github.com/Vidvox/ISF-Files; by mojovideotech), its GLSL made WGSL by naga (filament/effects/from_isf.py)
// --label:Stylize
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
    passindex: f32,
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
var isf_tex_inputImage: texture_2d<f32>;
@group(0) @binding(1) 
var isf_smp_inputImage: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> isf_FragNormCoord_1: vec2<f32>;
var<private> isf_FragColor: vec4<f32>;
var<private> gl_FragCoord_1: vec4<f32>;

fn permute(x: vec4<f32>) -> vec4<f32> {
    var x_1: vec4<f32>;

    x_1 = x;
    let _e26 = x_1;
    let _e32 = x_1;
    let _e33 = (((_e26 * 34f) + vec4(1f)) * _e32);
    let _e35 = vec4(289f);
    return (_e33 - (floor((_e33 / _e35)) * _e35));
}

fn taylorInvSqrt(r: vec4<f32>) -> vec4<f32> {
    var r_1: vec4<f32>;

    r_1 = r;
    let _e28 = r_1;
    return (vec4(1.7928429f) - (0.85373473f * _e28));
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
    let _e40 = v_1;
    let _e41 = v_1;
    let _e42 = C;
    i = floor((_e40 + vec3(dot(_e41, _e42.yyy))));
    let _e49 = v_1;
    let _e50 = i;
    let _e52 = i;
    let _e53 = C;
    x0_ = ((_e49 - _e50) + vec3(dot(_e52, _e53.xxx)));
    let _e59 = x0_;
    let _e61 = x0_;
    g = step(_e59.yzx, _e61.xyz);
    let _e66 = g;
    l = (vec3(1f) - _e66);
    let _e70 = g;
    let _e72 = l;
    i1_ = min(_e70.xyz, _e72.zxy);
    let _e76 = g;
    let _e78 = l;
    i2_ = max(_e76.xyz, _e78.zxy);
    let _e82 = x0_;
    let _e83 = i1_;
    let _e86 = C;
    x1_ = ((_e82 - _e83) + (1f * _e86.xxx));
    let _e91 = x0_;
    let _e92 = i2_;
    let _e95 = C;
    x2_ = ((_e91 - _e92) + (2f * _e95.xxx));
    let _e100 = x0_;
    let _e105 = C;
    x3_ = ((_e100 - vec3(1f)) + (3f * _e105.xxx));
    let _e110 = i;
    let _e112 = vec3(289f);
    i = (_e110 - (floor((_e110 / _e112)) * _e112));
    let _e117 = i;
    let _e120 = i1_;
    let _e122 = i2_;
    let _e128 = permute((vec4(_e117.z) + vec4<f32>(0f, _e120.z, _e122.z, 1f)));
    let _e129 = i;
    let _e134 = i1_;
    let _e136 = i2_;
    let _e141 = permute(((_e128 + vec4(_e129.y)) + vec4<f32>(0f, _e134.y, _e136.y, 1f)));
    let _e142 = i;
    let _e147 = i1_;
    let _e149 = i2_;
    let _e154 = permute(((_e141 + vec4(_e142.x)) + vec4<f32>(0f, _e147.x, _e149.x, 1f)));
    p = _e154;
    let _e160 = n;
    let _e161 = D;
    let _e164 = D;
    ns = ((_e160 * _e161.wyz) - _e164.xzx);
    let _e168 = p;
    let _e170 = p;
    let _e171 = ns;
    let _e174 = ns;
    j = (_e168 - (49f * floor(((_e170 * _e171.z) * _e174.z))));
    let _e181 = j;
    let _e182 = ns;
    x_2 = floor((_e181 * _e182.z));
    let _e187 = j;
    let _e189 = x_2;
    y = floor((_e187 - (7f * _e189)));
    let _e194 = x_2;
    let _e195 = ns;
    let _e198 = ns;
    x_3 = ((_e194 * _e195.x) + _e198.yyyy);
    let _e202 = y;
    let _e203 = ns;
    let _e206 = ns;
    y_1 = ((_e202 * _e203.x) + _e206.yyyy);
    let _e211 = x_3;
    let _e215 = y_1;
    h = ((vec4(1f) - abs(_e211)) - abs(_e215));
    let _e219 = x_3;
    let _e220 = _e219.xy;
    let _e221 = y_1;
    let _e222 = _e221.xy;
    b0_ = vec4<f32>(_e220.x, _e220.y, _e222.x, _e222.y);
    let _e229 = x_3;
    let _e230 = _e229.zw;
    let _e231 = y_1;
    let _e232 = _e231.zw;
    b1_ = vec4<f32>(_e230.x, _e230.y, _e232.x, _e232.y);
    let _e239 = b0_;
    s0_ = ((floor(_e239) * 2f) + vec4(1f));
    let _e247 = b1_;
    s1_ = ((floor(_e247) * 2f) + vec4(1f));
    let _e255 = h;
    sh = -(step(_e255, vec4(0f)));
    let _e261 = b0_;
    let _e263 = s0_;
    let _e265 = sh;
    a0_ = (_e261.xzyw + (_e263.xzyw * _e265.xxyy));
    let _e270 = b1_;
    let _e272 = s1_;
    let _e274 = sh;
    a1_ = (_e270.xzyw + (_e272.xzyw * _e274.zzww));
    let _e279 = a0_;
    let _e280 = _e279.xy;
    let _e281 = h;
    p0_ = vec3<f32>(_e280.x, _e280.y, _e281.x);
    let _e287 = a0_;
    let _e288 = _e287.zw;
    let _e289 = h;
    p1_ = vec3<f32>(_e288.x, _e288.y, _e289.y);
    let _e295 = a1_;
    let _e296 = _e295.xy;
    let _e297 = h;
    p2_ = vec3<f32>(_e296.x, _e296.y, _e297.z);
    let _e303 = a1_;
    let _e304 = _e303.zw;
    let _e305 = h;
    p3_ = vec3<f32>(_e304.x, _e304.y, _e305.w);
    let _e311 = p0_;
    let _e312 = p0_;
    let _e314 = p1_;
    let _e315 = p1_;
    let _e317 = p2_;
    let _e318 = p2_;
    let _e320 = p3_;
    let _e321 = p3_;
    let _e324 = taylorInvSqrt(vec4<f32>(dot(_e311, _e312), dot(_e314, _e315), dot(_e317, _e318), dot(_e320, _e321)));
    norm = _e324;
    let _e326 = p0_;
    let _e327 = norm;
    p0_ = (_e326 * _e327.x);
    let _e330 = p1_;
    let _e331 = norm;
    p1_ = (_e330 * _e331.y);
    let _e334 = p2_;
    let _e335 = norm;
    p2_ = (_e334 * _e335.z);
    let _e338 = p3_;
    let _e339 = norm;
    p3_ = (_e338 * _e339.w);
    let _e343 = x0_;
    let _e344 = x0_;
    let _e346 = x1_;
    let _e347 = x1_;
    let _e349 = x2_;
    let _e350 = x2_;
    let _e352 = x3_;
    let _e353 = x3_;
    m = max((vec4(0.6f) - vec4<f32>(dot(_e343, _e344), dot(_e346, _e347), dot(_e349, _e350), dot(_e352, _e353))), vec4(0f));
    let _e362 = m;
    let _e363 = m;
    m = (_e362 * _e363);
    let _e366 = m;
    let _e367 = m;
    let _e369 = p0_;
    let _e370 = x0_;
    let _e372 = p1_;
    let _e373 = x1_;
    let _e375 = p2_;
    let _e376 = x2_;
    let _e378 = p3_;
    let _e379 = x3_;
    return (42f * dot((_e366 * _e367), vec4<f32>(dot(_e369, _e370), dot(_e372, _e373), dot(_e375, _e376), dot(_e378, _e379))));
}

fn hash(h_1: f32) -> f32 {
    var h_2: f32;

    h_2 = h_1;
    let _e26 = h_2;
    return fract((sin(_e26) * 43758.547f));
}

fn tile(cell: vec2<f32>, size: vec2<f32>) -> vec2<f32> {
    var cell_1: vec2<f32>;
    var size_1: vec2<f32>;

    cell_1 = cell;
    size_1 = size;
    let _e28 = cell_1;
    let _e29 = size_1;
    return fract((_e28 * _e29));
}

fn box(a: vec2<f32>, b: vec2<f32>) -> f32 {
    var a_1: vec2<f32>;
    var b_1: vec2<f32>;
    var o: vec2<f32>;

    a_1 = a;
    b_1 = b;
    let _e28 = b_1;
    let _e29 = a_1;
    o = step(_e28, _e29);
    let _e32 = o;
    let _e34 = o;
    return (_e32.x * _e34.y);
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

    let _e24 = global.time;
    let _e25 = global.rate;
    T = (_e24 * _e25);
    let _e29 = gl_FragCoord_1;
    let _e31 = global.resolution;
    uv = (_e29.xy / _e31.xy);
    let _e35 = global.grid;
    g_1 = floor(_e35.xy);
    let _e39 = g_1;
    let _e41 = g_1;
    C_1 = (_e39.x * _e41.y);
    let _e46 = uv;
    let _e48 = g_1;
    let _e53 = g_1;
    let _e55 = uv;
    let _e57 = g_1;
    let _e63 = g_1;
    I = (((1f + floor((_e46.x * _e48.x))) + (_e53.y * floor((_e55.y * _e57.y)))) + _e63.x);
    let _e67 = uv;
    let _e68 = g_1;
    let _e69 = tile(_e67, _e68);
    st = _e69;
    let _e71 = I;
    let _e72 = C_1;
    let _e74 = st;
    let _e75 = global.edge;
    let _e76 = g_1;
    let _e80 = box(_e74, vec2<f32>((_e75 * _e76.xy)));
    S = ((_e71 / _e72) * _e80);
    let _e83 = S;
    let _e84 = S;
    let _e85 = hash(_e84);
    let _e86 = global.randomize;
    S = mix(_e83, _e85, _e86);
    let _e88 = S;
    let _e89 = T;
    color = vec3((_e88 * _e89));
    let _e93 = color;
    let _e94 = uv;
    let _e97 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e94.xy, 0f);
    let _e99 = global.blend;
    let _e102 = snoise((_e93 + (_e97.xyz * _e99)));
    n_1 = _e102;
    let _e104 = n_1;
    let _e105 = n_1;
    let _e106 = n_1;
    let _e107 = vec3<f32>(_e104, _e105, _e106);
    let _e113 = uv;
    let _e116 = textureSampleLevel(isf_tex_inputImage, isf_smp_inputImage, _e113.xy, 0f);
    let _e121 = global.gamma;
    isf_FragColor = sqrt((max((vec4<f32>(_e107.x, _e107.y, _e107.z, 1f) + _e116), vec4(0f)) + vec4(_e121)));
    return;
}

@fragment 
fn main(@location(0) isf_FragNormCoord: vec2<f32>, @builtin(position) gl_FragCoord: vec4<f32>) -> FragmentOutput {
    isf_FragNormCoord_1 = isf_FragNormCoord;
    gl_FragCoord_1 = gl_FragCoord;
    main_1();
    let _e33 = isf_FragColor;
    return FragmentOutput(_e33);
}
