// Film: three.js's FilmShader.js (MIT, Copyright (c) 2010-2026 three.js authors, github.com/mrdoob/three.js), its GLSL made WGSL by naga (filament/effects/from_three.py)
// --label:Film
// --track@intensity:intensity,0.0,2.0,0.5,0.001
// --check@grayscale_on:grayscale,0

struct Knobs {
    intensity: f32,
    grayscale_on: i32,
    time: f32,
}

struct IncidentLight {
    color: vec3<f32>,
    direction: vec3<f32>,
    visible: bool,
}

struct ReflectedLight {
    directDiffuse: vec3<f32>,
    directSpecular: vec3<f32>,
    indirectDiffuse: vec3<f32>,
    indirectSpecular: vec3<f32>,
}

struct FragmentOutput {
    @location(0) three_FragColor: vec4<f32>,
}

@group(0) @binding(0) 
var tDiffuse_picture: texture_2d<f32>;
@group(0) @binding(1) 
var tDiffuse_sampler: sampler;
@group(0) @binding(2) 
var<uniform> global: Knobs;
var<private> vUv_1: vec2<f32>;
var<private> three_FragColor: vec4<f32>;

fn pow2_(x: f32) -> f32 {
    var x_1: f32;

    x_1 = x;
    let _e12 = x_1;
    let _e13 = x_1;
    return (_e12 * _e13);
}

fn pow2_1(x_2: vec3<f32>) -> vec3<f32> {
    var x_3: vec3<f32>;

    x_3 = x_2;
    let _e12 = x_3;
    let _e13 = x_3;
    return (_e12 * _e13);
}

fn pow3_(x_4: f32) -> f32 {
    var x_5: f32;

    x_5 = x_4;
    let _e12 = x_5;
    let _e13 = x_5;
    let _e15 = x_5;
    return ((_e12 * _e13) * _e15);
}

fn pow4_(x_6: f32) -> f32 {
    var x_7: f32;
    var x2_: f32;

    x_7 = x_6;
    let _e12 = x_7;
    let _e13 = x_7;
    x2_ = (_e12 * _e13);
    let _e16 = x2_;
    let _e17 = x2_;
    return (_e16 * _e17);
}

fn max3_(v: vec3<f32>) -> f32 {
    var v_1: vec3<f32>;

    v_1 = v;
    let _e12 = v_1;
    let _e14 = v_1;
    let _e17 = v_1;
    return max(max(_e12.x, _e14.y), _e17.z);
}

fn average(v_2: vec3<f32>) -> f32 {
    var v_3: vec3<f32>;

    v_3 = v_2;
    let _e12 = v_3;
    return dot(_e12, vec3(0.3333333f));
}

fn rand(uv: vec2<f32>) -> f32 {
    var uv_1: vec2<f32>;
    var a: f32 = 12.9898f;
    var b: f32 = 78.233f;
    var c: f32 = 43758.547f;
    var dt: f32;
    var sn: f32;

    uv_1 = uv;
    let _e18 = uv_1;
    let _e20 = a;
    let _e21 = b;
    dt = dot(_e18.xy, vec2<f32>(_e20, _e21));
    let _e25 = dt;
    sn = (_e25 - (floor((_e25 / 3.1415927f)) * 3.1415927f));
    let _e32 = sn;
    let _e34 = c;
    return fract((sin(_e32) * _e34));
}

fn precisionSafeLength(v_4: vec3<f32>) -> f32 {
    var v_5: vec3<f32>;
    var maxComponent: f32;

    v_5 = v_4;
    let _e12 = v_5;
    let _e14 = max3_(abs(_e12));
    maxComponent = _e14;
    let _e16 = v_5;
    let _e17 = maxComponent;
    let _e21 = maxComponent;
    return (length((_e16 / vec3(_e17))) * _e21);
}

fn transformDirection(dir: vec3<f32>, matrix: mat4x4<f32>) -> vec3<f32> {
    var dir_1: vec3<f32>;
    var matrix_1: mat4x4<f32>;

    dir_1 = dir;
    matrix_1 = matrix;
    let _e14 = matrix_1;
    let _e15 = dir_1;
    return normalize((_e14 * vec4<f32>(_e15.x, _e15.y, _e15.z, 0f)).xyz);
}

fn transformNormalByInverseViewMatrix(normal: vec3<f32>, viewMatrix: mat4x4<f32>) -> vec3<f32> {
    var normal_1: vec3<f32>;
    var viewMatrix_1: mat4x4<f32>;

    normal_1 = normal;
    viewMatrix_1 = viewMatrix;
    let _e14 = normal_1;
    let _e20 = viewMatrix_1;
    return normalize((vec4<f32>(_e14.x, _e14.y, _e14.z, 0f) * _e20).xyz);
}

fn transformDirectionByInverseViewMatrix(dir_2: vec3<f32>, viewMatrix_2: mat4x4<f32>) -> vec3<f32> {
    var dir_3: vec3<f32>;
    var viewMatrix_3: mat4x4<f32>;

    dir_3 = dir_2;
    viewMatrix_3 = viewMatrix_2;
    let _e14 = dir_3;
    let _e20 = viewMatrix_3;
    return normalize((vec4<f32>(_e14.x, _e14.y, _e14.z, 0f) * _e20).xyz);
}

fn isPerspectiveMatrix(m: mat4x4<f32>) -> bool {
    var m_1: mat4x4<f32>;

    m_1 = m;
    let _e16 = m_1[2][3];
    return (_e16 == -1f);
}

fn equirectUv(dir_4: vec3<f32>) -> vec2<f32> {
    var dir_5: vec3<f32>;
    var u: f32;
    var v_6: f32;

    dir_5 = dir_4;
    let _e12 = dir_5;
    let _e14 = dir_5;
    u = ((atan2(_e12.z, _e14.x) * 0.15915494f) + 0.5f);
    let _e22 = dir_5;
    v_6 = ((asin(clamp(_e22.y, -1f, 1f)) * 0.31830987f) + 0.5f);
    let _e34 = u;
    let _e35 = v_6;
    return vec2<f32>(_e34, _e35);
}

fn BRDF_Lambert(diffuseColor: vec3<f32>) -> vec3<f32> {
    var diffuseColor_1: vec3<f32>;

    diffuseColor_1 = diffuseColor;
    let _e13 = diffuseColor_1;
    return (0.31830987f * _e13);
}

fn F_Schlick(f0_: vec3<f32>, f90_: f32, dotVH: f32) -> vec3<f32> {
    var f0_1: vec3<f32>;
    var f90_1: f32;
    var dotVH_1: f32;
    var fresnel: f32;

    f0_1 = f0_;
    f90_1 = f90_;
    dotVH_1 = dotVH;
    let _e18 = dotVH_1;
    let _e22 = dotVH_1;
    fresnel = exp2((((-5.55473f * _e18) - 6.98316f) * _e22));
    let _e26 = f0_1;
    let _e28 = fresnel;
    let _e31 = f90_1;
    let _e32 = fresnel;
    return ((_e26 * (1f - _e28)) + vec3((_e31 * _e32)));
}

fn F_Schlick_1(f0_2: f32, f90_2: f32, dotVH_2: f32) -> f32 {
    var f0_3: f32;
    var f90_3: f32;
    var dotVH_3: f32;
    var fresnel_1: f32;

    f0_3 = f0_2;
    f90_3 = f90_2;
    dotVH_3 = dotVH_2;
    let _e18 = dotVH_3;
    let _e22 = dotVH_3;
    fresnel_1 = exp2((((-5.55473f * _e18) - 6.98316f) * _e22));
    let _e26 = f0_3;
    let _e28 = fresnel_1;
    let _e31 = f90_3;
    let _e32 = fresnel_1;
    return ((_e26 * (1f - _e28)) + (_e31 * _e32));
}

fn luminance(rgb: vec3<f32>) -> f32 {
    var rgb_1: vec3<f32>;

    rgb_1 = rgb;
    let _e16 = rgb_1;
    return dot(vec3<f32>(0.2126f, 0.7152f, 0.0722f), _e16);
}

fn main_1() {
    var base: vec4<f32>;
    var noise: f32;
    var color: vec3<f32>;

    let _e10 = vUv_1;
    let _e11 = textureSample(tDiffuse_picture, tDiffuse_sampler, _e10);
    base = _e11;
    let _e13 = vUv_1;
    let _e14 = global.time;
    let _e18 = rand(fract((_e13 + vec2(_e14))));
    noise = _e18;
    let _e20 = base;
    let _e22 = base;
    let _e25 = noise;
    color = (_e20.xyz + (_e22.xyz * clamp((0.1f + _e25), 0f, 1f)));
    let _e33 = base;
    let _e35 = color;
    let _e36 = global.intensity;
    color = mix(_e33.xyz, _e35, vec3(_e36));
    let _e39 = global.grayscale_on;
    if (_e39 != 0i) {
        {
            let _e42 = color;
            let _e43 = luminance(_e42);
            color = vec3(_e43);
        }
    }
    let _e45 = color;
    let _e46 = base;
    three_FragColor = vec4<f32>(_e45.x, _e45.y, _e45.z, _e46.w);
    return;
}

@fragment 
fn main(@location(0) vUv: vec2<f32>) -> FragmentOutput {
    vUv_1 = vUv;
    main_1();
    let _e17 = three_FragColor;
    return FragmentOutput(_e17);
}
