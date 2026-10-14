// Sepia: three.js's SepiaShader.js (MIT, Copyright (c) 2010-2026 three.js authors, github.com/mrdoob/three.js), its GLSL made WGSL by naga (filament/effects/from_three.py)
// --track@amount:amount,0.0,4.0,1.0,0.001

struct Knobs {
    amount: f32,
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

fn main_1() {
    var color: vec4<f32>;
    var c: vec3<f32>;

    let _e6 = vUv_1;
    let _e7 = textureSample(tDiffuse_picture, tDiffuse_sampler, _e6);
    color = _e7;
    let _e9 = color;
    c = _e9.xyz;
    let _e13 = c;
    let _e16 = global.amount;
    let _e20 = global.amount;
    let _e23 = global.amount;
    color.x = dot(_e13, vec3<f32>((1f - (0.607f * _e16)), (0.769f * _e20), (0.189f * _e23)));
    let _e28 = c;
    let _e30 = global.amount;
    let _e34 = global.amount;
    let _e38 = global.amount;
    color.y = dot(_e28, vec3<f32>((0.349f * _e30), (1f - (0.314f * _e34)), (0.168f * _e38)));
    let _e43 = c;
    let _e45 = global.amount;
    let _e48 = global.amount;
    let _e52 = global.amount;
    color.z = dot(_e43, vec3<f32>((0.272f * _e45), (0.534f * _e48), (1f - (0.869f * _e52))));
    let _e59 = color;
    let _e61 = min(vec3(1f), _e59.xyz);
    let _e62 = color;
    three_FragColor = vec4<f32>(_e61.x, _e61.y, _e61.z, _e62.w);
    return;
}

@fragment 
fn main(@location(0) vUv: vec2<f32>) -> FragmentOutput {
    vUv_1 = vUv;
    main_1();
    let _e13 = three_FragColor;
    return FragmentOutput(_e13);
}
