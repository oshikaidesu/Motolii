// Vignette: three.js's VignetteShader.js (MIT, Copyright (c) 2010-2026 three.js authors, github.com/mrdoob/three.js), its GLSL made WGSL by naga (filament/effects/from_three.py)
// --track@offset:offset,0.0,4.0,1.0,0.001
// --track@darkness:darkness,0.0,4.0,1.0,0.001

struct Knobs {
    offset: f32,
    darkness: f32,
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
    var texel: vec4<f32>;
    var uv: vec2<f32>;

    let _e8 = vUv_1;
    let _e9 = textureSample(tDiffuse_picture, tDiffuse_sampler, _e8);
    texel = _e9;
    let _e11 = vUv_1;
    let _e15 = global.offset;
    uv = ((_e11 - vec2(0.5f)) * vec2(_e15));
    let _e19 = texel;
    let _e22 = global.darkness;
    let _e25 = uv;
    let _e26 = uv;
    let _e29 = mix(_e19.xyz, vec3((1f - _e22)), vec3(dot(_e25, _e26)));
    let _e30 = texel;
    three_FragColor = vec4<f32>(_e29.x, _e29.y, _e29.z, _e30.w);
    return;
}

@fragment 
fn main(@location(0) vUv: vec2<f32>) -> FragmentOutput {
    vUv_1 = vUv;
    main_1();
    let _e15 = three_FragColor;
    return FragmentOutput(_e15);
}
