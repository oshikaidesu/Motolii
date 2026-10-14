// BleachBypass: three.js's BleachBypassShader.js (MIT, Copyright (c) 2010-2026 three.js authors, github.com/mrdoob/three.js), its GLSL made WGSL by naga (filament/effects/from_three.py)
// --track@opacity:opacity,0.0,4.0,1.0,0.001

struct Knobs {
    opacity: f32,
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

fn luminance(rgb: vec3<f32>) -> f32 {
    var rgb_1: vec3<f32>;

    rgb_1 = rgb;
    let _e12 = rgb_1;
    return dot(vec3<f32>(0.2126f, 0.7152f, 0.0722f), _e12);
}

fn main_1() {
    var base: vec4<f32>;
    var lum: f32;
    var blend: vec3<f32>;
    var L: f32;
    var result1_: vec3<f32>;
    var result2_: vec3<f32>;
    var newColor: vec3<f32>;
    var A2_: f32;
    var mixRGB: vec3<f32>;

    let _e6 = vUv_1;
    let _e7 = textureSample(tDiffuse_picture, tDiffuse_sampler, _e6);
    base = _e7;
    let _e9 = base;
    let _e11 = luminance(_e9.xyz);
    lum = _e11;
    let _e13 = lum;
    blend = vec3(_e13);
    let _e19 = lum;
    L = min(1f, max(0f, (10f * (_e19 - 0.45f))));
    let _e27 = base;
    let _e30 = blend;
    result1_ = ((2f * _e27.xyz) * _e30);
    let _e36 = blend;
    let _e41 = base;
    result2_ = (vec3(1f) - ((2f * (vec3(1f) - _e36)) * (vec3(1f) - _e41.xyz)));
    let _e49 = result1_;
    let _e50 = result2_;
    let _e51 = L;
    newColor = mix(_e49, _e50, vec3(_e51));
    let _e55 = global.opacity;
    let _e56 = base;
    A2_ = (_e55 * _e56.w);
    let _e60 = A2_;
    let _e61 = newColor;
    mixRGB = (_e60 * _e61.xyz);
    let _e65 = mixRGB;
    let _e67 = A2_;
    let _e69 = base;
    mixRGB = (_e65 + ((1f - _e67) * _e69.xyz));
    let _e73 = mixRGB;
    let _e74 = base;
    three_FragColor = vec4<f32>(_e73.x, _e73.y, _e73.z, _e74.w);
    return;
}

@fragment 
fn main(@location(0) vUv: vec2<f32>) -> FragmentOutput {
    vUv_1 = vUv;
    main_1();
    let _e13 = three_FragColor;
    return FragmentOutput(_e13);
}
